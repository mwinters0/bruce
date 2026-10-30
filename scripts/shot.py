#!/usr/bin/env python3
"""An HTML page rendered headless to PNGs and a PDF, for review.

`bruce report digest` writes a page nobody can check by reading it, and its
layout fails at a phone's width and in print. This renders it in Chrome's
headless shell, run in a container so the host supplies no libraries and the
browser is the same on every machine, at three sizes under `--out`:

- `<stem>-1280.png`, a desktop's width;
- `<stem>-390.png`, a phone's: emulated as one (`mobile`, so the page's
  `<meta name=viewport>` sets its layout width, as on a phone; a page with none
  lays out 980 px wide) at a device scale of 1, a larger scale adding pixels and
  nothing to a layout check;
- `<stem>-print.png`, about A4's width (794 px), of `<stem>-print.html`, a copy
  of the page whose `@media print` reads `@media all` and whose every
  `<details>` is open: the print palette and the rules for the disclosures,
  with no page breaks;
- `<stem>.pdf`, the page printed, which has them (`break-inside` and the
  like). Read it in a PDF viewer; Chrome opens every `<details>` in print
  through `::details-content` (Chromium 131), so an older browser does not.

Each PNG is as tall as the page, up to `--height`, a longer page being cut. The
fonts are the pinned DejaVu, mounted in with a `fonts.conf` that stands it
behind every family the page names, so a render does not differ by the host's
fonts. `FONTCONFIG_FILE` must be absolute, a relative one being "not found" and
the page rendering in a fallback font. **The renders are for a person to read**:
nothing asserts on them.

**The browser is `IMAGE`**, pinned by version tag and index digest, the daily
builds being only partly kept. Its shell has no one-shot commands
(`--screenshot`, `--print-to-pdf`, `--dump-dom`): the image's build imports
Chromium's `build/args/headless.gn`, which sets `headless_enable_commands =
false`, so the flags are ignored and the page stays open. The page is driven
over the DevTools protocol with the standard library, the browser's port
published on 127.0.0.1 only, which a build with the commands would still need:
a flag cannot size a PNG to the page, wait on `document.fonts.ready`, or
screenshot a URL with a fragment, which paints blank above the target. A
fragment naming an element inside a closed `<details>` opens
its ancestors, not the element (HTML's ancestor revealing algorithm: Chrome
97), so read a reveal off the DOM. Rejected: Servo's `servoshell`, which has no
paged media and lays a phone's tables out otherwise than Chrome; a
`Containerfile` around Chrome for Testing's binary, a build step for Google's
signature on a binary that renders our own page; Debian's `chromium`, which
cannot be pinned exactly; Playwright, for the node package.

**The container runtime is a setting**: `--runtime`, else `BRUCE_SHOT_RUNTIME`
(a command, `sudo nerdctl` among them), else the first of `podman` and `docker`
that answers `info`, else an error naming the setting. PAGE is a path from
`scripts/`, where the task runs, or an absolute one.

Usage:

    mise run shot
    cd scripts && uv run shot.py [PAGE] [--out DIR] [--height PX]
        [--runtime COMMAND] [--fonts DIR]
    cd scripts && uv run python -m unittest test_shot
"""

from __future__ import annotations

import argparse
import base64
import http.client
import json
import os
import re
import secrets
import shlex
import shutil
import socket
import struct
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Callable, Sequence

REPO = Path(__file__).resolve().parent.parent
PAGE = Path("fixtures/antora/digest.html")
OUT = Path("runs/shot")
#: The mise tool `mise.toml` pins: the font.
FONT_TOOL = "github:dejavu-fonts/dejavu-fonts"
#: Chrome's headless shell, 2026-10-30's 157.0.8103.27 (MIT, built from Chromium
#: on debian:trixie-slim, no fonts installed).
IMAGE = (
    "docker.io/chromedp/headless-shell:157.0.8103.27"
    "@sha256:7bee201478b2f6266aae01243740ee85c16281e32f42d3aed9fdf04f4eace7a1"
)
RUNTIME_ENV = "BRUCE_SHOT_RUNTIME"
#: The runtimes tried in turn when none is named.
RUNTIMES = ("podman", "docker")
#: Each PNG's name suffix, window width in px and whether it emulates a phone;
#: the print one is of the copy.
WIDTHS = (("1280", 1280, False), ("390", 390, True), ("print", 794, False))
#: A page taller than this is cut, Chrome's texture limit being 16384.
HEIGHT = 16000
#: What a start, a load or a reply may take, in s; one that has not finished by
#: now never will.
TIMEOUT = 120
#: Where the container sees the font, the output directory and the page's.
FONT_MOUNT, WORK_MOUNT, SRC_MOUNT = "/fonts", "/work", "/src"
CONTAINER_CACHE = "/tmp/fontconfig-cache"
DEVTOOLS_PORT = 9222

PRINT_RE = re.compile(r"@media\s+print\b")
DETAILS_RE = re.compile(r"<details\b([^>]*)>")
OPEN_RE = re.compile(r"\sopen\b")


class ShotError(Exception):
    """A render cannot be made; the message says why."""


def print_copy(html: str) -> str:
    """`html` with its print rules applied on screen and every `<details>`
    open, or a ShotError where it has no `@media print` to apply."""
    applied, n = PRINT_RE.subn("@media all", html)
    if n == 0:
        raise ShotError("the page has no `@media print` rule to apply")

    def opened(m: re.Match[str]) -> str:
        attrs = m.group(1)
        return m.group(0) if OPEN_RE.search(attrs) else f"<details{attrs} open>"

    return DETAILS_RE.sub(opened, applied)


def fonts_conf(font_dir: str, cache_dir: str) -> str:
    """A fontconfig file serving DejaVu from `font_dir`, standing behind every
    family the page asks for, `ui-monospace` and `monospace` included."""
    mono = "DejaVu Sans Mono"
    match = (
        '  <match target="pattern"><test name="family"><string>{f}</string></test>'
        '<edit name="family" mode="prepend" binding="strong"><string>{to}</string>'
        "</edit></match>"
    )
    return "\n".join(
        [
            '<?xml version="1.0"?>',
            '<!DOCTYPE fontconfig SYSTEM "fonts.dtd">',
            "<fontconfig>",
            f"  <dir>{font_dir}</dir>",
            f"  <cachedir>{cache_dir}</cachedir>",
            match.format(f="ui-monospace", to=mono),
            match.format(f="monospace", to=mono),
            match.format(f="serif", to="DejaVu Serif"),
            '  <match target="pattern"><edit name="family" mode="append_last">'
            "<string>DejaVu Sans</string></edit></match>",
            "</fontconfig>",
            "",
        ]
    )


def mise_where(tool: str) -> Path:
    """The directory mise installed `tool` in, or a ShotError saying to
    install it."""
    done = subprocess.run(
        ["mise", "where", tool], cwd=REPO, capture_output=True, text=True
    )
    path = done.stdout.strip()
    if done.returncode != 0 or not path:
        raise ShotError(f"{tool} is not installed: run `mise install`")
    return Path(path)


def answers(runtime: str) -> bool:
    """Whether `runtime` is installed and its daemon, if it has one, replies."""
    if shutil.which(runtime) is None:
        return False
    try:
        done = subprocess.run(
            [runtime, "info"], capture_output=True, timeout=30, check=False
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    return done.returncode == 0


def runtime_command(
    flag: str | None,
    env: dict[str, str],
    answers: Callable[[str], bool] = answers,
) -> list[str]:
    """The container runtime as a command: the flag, else `RUNTIME_ENV`, taken
    as given, else the first of RUNTIMES that answers; a ShotError naming the
    setting where none does."""
    named = flag or env.get(RUNTIME_ENV)
    if named:
        command = shlex.split(named)
        if command:
            return command
    for runtime in RUNTIMES:
        if answers(runtime):
            return [runtime]
    raise ShotError(
        f"no container runtime answers ({', '.join(RUNTIMES)}): "
        f"set {RUNTIME_ENV} or pass --runtime, e.g. `sudo nerdctl`"
    )


def container_command(
    runtime: list[str], name: str, host_port: int, font_dir: Path, out: Path, src: Path
) -> list[str]:
    """Starts IMAGE detached under `name`, the browser's port on 127.0.0.1's
    `host_port`, the font, the output directory and the page's read-only. No
    `--rm`, which nerdctl refuses beside `-d`: `render` removes it."""

    def mount(host: Path, inside: str) -> list[str]:
        return ["-v", f"{host.resolve()}:{inside}:ro"]

    return [
        *runtime,
        "run",
        "-d",
        "--name",
        name,
        "-p",
        f"127.0.0.1:{host_port}:{DEVTOOLS_PORT}",
        "-e",
        f"FONTCONFIG_FILE={WORK_MOUNT}/fonts.conf",
        *mount(font_dir, FONT_MOUNT),
        *mount(out, WORK_MOUNT),
        *mount(src, SRC_MOUNT),
        IMAGE,
    ]


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


# The DevTools protocol over a WebSocket, which needs no more than this.


def encode_frame(payload: bytes, key: bytes | None = None) -> bytes:
    """One final text frame; masked with the 4 bytes `key`, as a client's must
    be, or not."""
    n = len(payload)
    mask = 0x80 if key else 0
    if n < 126:
        head = struct.pack("!BB", 0x81, mask | n)
    elif n < 1 << 16:
        head = struct.pack("!BBH", 0x81, mask | 126, n)
    else:
        head = struct.pack("!BBQ", 0x81, mask | 127, n)
    if key:
        payload = bytes(b ^ key[i % 4] for i, b in enumerate(payload))
        return head + key + payload
    return head + payload


class WebSocket:
    """A client connection that sends text and reads whole text messages."""

    def __init__(self, sock: socket.socket, leftover: bytes = b"") -> None:
        self.sock = sock
        self.buffer = leftover

    @classmethod
    def connect(cls, port: int, path: str) -> WebSocket:
        sock = socket.create_connection(("127.0.0.1", port), timeout=TIMEOUT)
        key = base64.b64encode(secrets.token_bytes(16)).decode()
        sock.sendall(
            (
                f"GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n"
                f"Upgrade: websocket\r\nConnection: Upgrade\r\n"
                f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
            ).encode()
        )
        data = b""
        while b"\r\n\r\n" not in data:
            chunk = sock.recv(4096)
            if not chunk:
                raise ShotError("the browser closed the DevTools connection")
            data += chunk
        head, _, rest = data.partition(b"\r\n\r\n")
        if b" 101 " not in head.split(b"\r\n")[0]:
            raise ShotError(f"DevTools refused the WebSocket: {head.decode(errors='replace')}")
        return cls(sock, rest)

    def send(self, text: str) -> None:
        self.sock.sendall(encode_frame(text.encode(), secrets.token_bytes(4)))

    def _take(self, n: int) -> bytes:
        while len(self.buffer) < n:
            chunk = self.sock.recv(1 << 20)
            if not chunk:
                raise ShotError("the browser closed the DevTools connection")
            self.buffer += chunk
        taken, self.buffer = self.buffer[:n], self.buffer[n:]
        return taken

    def receive(self) -> str:
        """The next text message, its fragments joined; a ping answered."""
        message = b""
        while True:
            b0, b1 = self._take(2)
            opcode, n = b0 & 0x0F, b1 & 0x7F
            if n == 126:
                (n,) = struct.unpack("!H", self._take(2))
            elif n == 127:
                (n,) = struct.unpack("!Q", self._take(8))
            payload = self._take(n)
            if opcode == 0x8:
                raise ShotError("the browser closed the DevTools connection")
            if opcode == 0x9:
                mask = secrets.token_bytes(4)
                masked = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
                self.sock.sendall(struct.pack("!BB", 0x8A, 0x80 | len(payload)) + mask + masked)
                continue
            if opcode in (0x0, 0x1, 0x2):
                message += payload
                if b0 & 0x80:
                    return message.decode()

    def close(self) -> None:
        self.sock.close()


class Page:
    """One browser tab, spoken to with `call`."""

    def __init__(self, ws: WebSocket) -> None:
        self.ws = ws
        self.next_id = 0
        self.events: list[str] = []

    def call(self, method: str, **params: Any) -> dict[str, Any]:
        self.next_id += 1
        self.ws.send(json.dumps({"id": self.next_id, "method": method, "params": params}))
        while True:
            message = json.loads(self.ws.receive())
            if message.get("id") == self.next_id:
                if "error" in message:
                    raise ShotError(f"{method}: {message['error'].get('message')}")
                return dict(message.get("result", {}))
            if "method" in message:
                self.events.append(message["method"])

    def wait_for(self, event: str) -> None:
        while event not in self.events:
            message = json.loads(self.ws.receive())
            if "method" in message:
                self.events.append(message["method"])
        self.events.remove(event)

    def load(self, url: str, width: int, height: int, mobile: bool = False) -> None:
        """Navigates to `url` in a window `width` by `height`, a phone's if
        `mobile`, and waits for the page and its fonts."""
        self.call("Emulation.setDeviceMetricsOverride",
                  width=width, height=height, deviceScaleFactor=1, mobile=mobile)
        self.events.clear()
        self.call("Page.navigate", url=url)
        self.wait_for("Page.loadEventFired")
        self.call("Runtime.evaluate", expression="document.fonts.ready",
                  awaitPromise=True)

    def content_height(self) -> int:
        result = self.call("Runtime.evaluate",
                           expression="document.documentElement.scrollHeight",
                           returnByValue=True)
        return int(result["result"]["value"])

    def png(self, url: str, width: int, cap: int, mobile: bool = False) -> bytes:
        """A screenshot of `url` at `width`, a phone's if `mobile`, as tall as
        the page up to `cap`."""
        self.load(url, width, 800, mobile)
        height = max(1, min(self.content_height(), cap))
        self.call("Emulation.setDeviceMetricsOverride",
                  width=width, height=height, deviceScaleFactor=1, mobile=mobile)
        return base64.b64decode(self.call("Page.captureScreenshot", format="png")["data"])

    def pdf(self, url: str) -> bytes:
        """`url` printed, on the default paper."""
        self.load(url, 1280, 800)
        data = self.call("Page.printToPDF", printBackground=True,
                         preferCSSPageSize=True)["data"]
        return base64.b64decode(data)


def open_tab(port: int) -> Page:
    """A new tab of the browser on 127.0.0.1's `port`, waiting for it to
    answer."""
    deadline = time.monotonic() + TIMEOUT
    while True:
        try:
            conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
            conn.request("PUT", "/json/new?about:blank")
            response = conn.getresponse()
            body = response.read()
            conn.close()
            if response.status == 200:
                target = json.loads(body)
                path = "/" + target["webSocketDebuggerUrl"].split("/", 3)[3]
                page = Page(WebSocket.connect(port, path))
                page.call("Page.enable")
                return page
        except (OSError, http.client.HTTPException, ValueError):
            pass
        if time.monotonic() > deadline:
            raise ShotError(f"the browser did not answer on port {port} in {TIMEOUT} s")
        time.sleep(0.25)


def render(page: Path, out: Path, height: int, runtime: list[str], fonts: Path) -> list[Path]:
    """Renders `page` at each of WIDTHS under `out`, and prints it; returns the
    files."""
    try:
        html = page.read_text(encoding="utf-8")
    except OSError as e:
        raise ShotError(f"{page}: {e}") from e
    out.mkdir(parents=True, exist_ok=True)
    (out / "fonts.conf").write_text(fonts_conf(FONT_MOUNT, CONTAINER_CACHE), encoding="utf-8")
    stem = page.stem
    printed = out / f"{stem}-print.html"
    printed.write_text(print_copy(html), encoding="utf-8")
    name, port = f"bruce-shot-{os.getpid()}", free_port()
    start = container_command(runtime, name, port, fonts, out, page.resolve().parent)
    done = subprocess.run(start, capture_output=True, text=True)
    if done.returncode != 0:
        raise ShotError(f"{' '.join(runtime)} could not start {IMAGE}:\n{done.stderr.strip()}")
    files = []
    tab = None
    try:
        tab = open_tab(port)
        for suffix, width, mobile in WIDTHS:
            url = f"file://{WORK_MOUNT}/{printed.name}" if suffix == "print" else (
                f"file://{SRC_MOUNT}/{page.name}")
            png = out / f"{stem}-{suffix}.png"
            png.write_bytes(tab.png(url, width, height, mobile))
            files.append(png)
        pdf = out / f"{stem}.pdf"
        pdf.write_bytes(tab.pdf(f"file://{SRC_MOUNT}/{page.name}"))
        files.append(pdf)
    except (OSError, TimeoutError) as e:
        raise ShotError(f"the browser stopped answering: {e}") from e
    finally:
        if tab is not None:
            tab.ws.close()
        subprocess.run([*runtime, "rm", "-f", name], capture_output=True)
    return files


def run(page: Path, out: Path, height: int, runtime: str | None, fonts: Path | None) -> int:
    try:
        command = runtime_command(runtime, dict(os.environ))
        font_dir = fonts or mise_where(FONT_TOOL) / "ttf"
        if not font_dir.exists():
            raise ShotError(f"{font_dir} does not exist")
        files = render(page, out, height, command, font_dir)
    except ShotError as e:
        print(f"shot: {e}", file=sys.stderr)
        return 1
    for file in files:
        print(f"shot: wrote {file}")
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("page", nargs="?", type=Path, default=REPO / PAGE, help="the HTML file")
    parser.add_argument("--out", type=Path, default=REPO / OUT, help="where the renders go")
    parser.add_argument("--height", type=int, default=HEIGHT, help="a PNG's greatest height, in px")
    parser.add_argument("--runtime", help=f"the container command, else ${RUNTIME_ENV}")
    parser.add_argument("--fonts", type=Path, help="a directory of DejaVu's TTFs, where not mise's")
    args = parser.parse_args(argv)
    if args.height < 1:
        parser.error("--height is at least 1")
    return run(args.page, args.out, args.height, args.runtime, args.fonts)


if __name__ == "__main__":
    raise SystemExit(main())
