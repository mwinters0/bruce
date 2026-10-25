#!/usr/bin/env python3
"""An HTML page rendered headless to PNGs, for review.

`bruce report digest` writes a page nobody can check by reading it, and its
layout fails at a phone's width and in print. This renders it with
Servo's `servoshell`, pinned in `mise.toml`, at three sizes under `--out`:

- `<stem>-1280.png`, a desktop's width;
- `<stem>-390.png`, a phone's;
- `<stem>-print.png`, about A4's width (794 px), of `<stem>-print.html`, a copy
  of the page whose `@media print` reads `@media all` and whose every
  `<details>` is open. It shows the print palette and the rules for the
  disclosures, and **no page breaks**: Servo has no paged media and cannot
  print to PDF, so `break-inside` and the like are checked by eye in another
  browser.

The window alone is captured, `--height` tall (a flag, since a page's height is
its own); a page longer than that is cut. This machine may have no fonts, so a
`fonts.conf` beside the renders points fontconfig at the pinned DejaVu, and
DejaVu stands behind every family the page names, so a render does not differ by
the host's fonts. **The renders are for a person to read**: nothing asserts on
them. Closed, a page renders as Chrome's headless shell renders it, at both
widths; opened, Servo lays a phone's tables out otherwise (it breaks at every
`/`, scrolls at another column and paints no edge shadow, where Chrome crushes
the columns), drops the underscores in an `<h3>`'s `<code>` and reveals no
fragment's closed ancestors. So a phone's layout and print are checked in
Chrome, and this is a first look. PAGE is a path from `scripts/`, where the
task runs, or an absolute one. Rejected: Playwright with Chrome's headless
shell, for the node package, the 278 MB browser and the system libraries it
brings.

Usage:

    mise run shot
    cd scripts && uv run shot.py [PAGE] [--out DIR] [--height PX]
        [--servoshell FILE] [--fonts DIR]
    cd scripts && uv run python -m unittest test_shot
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Sequence

REPO = Path(__file__).resolve().parent.parent
PAGE = Path("fixtures/antora/digest.html")
OUT = Path("runs/shot")
#: The mise tools `mise.toml` pins: the browser and the font.
SERVO_TOOL = "github:servo/servo"
FONT_TOOL = "github:dejavu-fonts/dejavu-fonts"
#: Each render's name suffix and window width, in px.
WIDTHS = (("1280", 1280), ("390", 390), ("print", 794))
HEIGHT = 3200
#: A render that has not finished by now never will.
TIMEOUT = 120

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


def fonts_conf(font_dir: Path, cache_dir: Path) -> str:
    """A fontconfig file serving DejaVu, standing behind every family the
    page asks for, `ui-monospace` and `monospace` included."""
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


def render_command(servoshell: Path, page: Path, png: Path, width: int, height: int) -> list[str]:
    return [
        str(servoshell),
        "-z",
        "-x",
        f"--window-size={width}x{height}",
        "-o",
        str(png),
        page.resolve().as_uri(),
    ]


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


def render(page: Path, out: Path, height: int, servoshell: Path, fonts: Path) -> list[Path]:
    """Renders `page` at each of WIDTHS under `out`; returns the PNGs."""
    try:
        html = page.read_text(encoding="utf-8")
    except OSError as e:
        raise ShotError(f"{page}: {e}") from e
    out.mkdir(parents=True, exist_ok=True)
    conf = out / "fonts.conf"
    conf.write_text(fonts_conf(fonts.resolve(), (out / "fontconfig-cache").resolve()), encoding="utf-8")
    env = {**os.environ, "FONTCONFIG_FILE": str(conf.resolve())}
    stem = page.stem
    printed = out / f"{stem}-print.html"
    printed.write_text(print_copy(html), encoding="utf-8")
    pngs = []
    for name, width in WIDTHS:
        source = printed if name == "print" else page
        png = out / f"{stem}-{name}.png"
        png.unlink(missing_ok=True)
        command = render_command(servoshell, source, png, width, height)
        try:
            done = subprocess.run(
                command, env=env, capture_output=True, text=True, timeout=TIMEOUT
            )
        except subprocess.TimeoutExpired as e:
            raise ShotError(f"{png}: servoshell did not finish in {TIMEOUT} s") from e
        if done.returncode != 0 or not png.exists():
            raise ShotError(
                f"{png}: servoshell exited {done.returncode} and wrote "
                f"{'no file' if not png.exists() else 'the file'}\n{done.stderr.strip()}"
            )
        pngs.append(png)
    return pngs


def run(page: Path, out: Path, height: int, servoshell: Path | None, fonts: Path | None) -> int:
    try:
        shell = servoshell or mise_where(SERVO_TOOL) / "servoshell"
        font_dir = fonts or mise_where(FONT_TOOL) / "ttf"
        for tool in (shell, font_dir):
            if not tool.exists():
                raise ShotError(f"{tool} does not exist")
        pngs = render(page, out, height, shell, font_dir)
    except ShotError as e:
        print(f"shot: {e}", file=sys.stderr)
        return 1
    for png in pngs:
        print(f"shot: wrote {png}")
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("page", nargs="?", type=Path, default=REPO / PAGE, help="the HTML file")
    parser.add_argument("--out", type=Path, default=REPO / OUT, help="where the renders go")
    parser.add_argument("--height", type=int, default=HEIGHT, help="each window's height, in px")
    parser.add_argument("--servoshell", type=Path, help="servoshell, where not mise's")
    parser.add_argument("--fonts", type=Path, help="a directory of DejaVu's TTFs, where not mise's")
    args = parser.parse_args(argv)
    if args.height < 1:
        parser.error("--height is at least 1")
    return run(args.page, args.out, args.height, args.servoshell, args.fonts)


if __name__ == "__main__":
    raise SystemExit(main())
