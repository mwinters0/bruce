#!/usr/bin/env python3
"""The deployment guide's scripts, run (D42): the scheduled job over a site
and the pull over what it published.

`docs/manual/deploy.md` tags `bruce-nightly.sh`, which a timer runs, and
`bruce-pull.sh`, which a web server's timer runs. `sim.py` parses them; this
module runs them, as `e2e.py` does over the fixture Antora site and
`test_deployment.py` does over a stub `bruce`:

- `night` runs the job once, with the guide's variables in its environment,
  `bruce` first on the `PATH`.
- `serve_directory` serves a directory over HTTP, as Pages or a web server
  serves the published one.
- `pull` runs the pull script against it.
- `check_published` holds the published directory to the guide's contract: the
  fixed names, the files named by content, `SHA256SUMS` listing what it should
  and every sum right.

Nothing here is a command line; `e2e.py` calls it.
"""

from __future__ import annotations

import functools
import hashlib
import http.server
import json
import re
import shutil
import subprocess
import tarfile
import threading
from pathlib import Path
from typing import Mapping, Sequence

import sim

REPO = Path(__file__).resolve().parent.parent
NIGHTLY = "bruce-nightly.sh"
PULL = "bruce-pull.sh"
#: The names the contract gives a night's three files: fixed, for people, and
#: by content, for a fetcher, 16 hex digits of the sha256 in the second.
FIXED = ("bruce.tar.gz", "bruce.toml", "baseline.duckdb.gz")
BY_CONTENT = (
    re.compile(r"bruce-[0-9a-f]{16}\.tar\.gz"),
    re.compile(r"config-[0-9a-f]{16}\.toml"),
    re.compile(r"baseline-[0-9a-f]{16}\.duckdb\.gz"),
)


class DeploymentError(Exception):
    """A script cannot run, or its result is not the guide's; the message says which."""


def extract(guide: Path, out: Path) -> Path:
    """The guide's tagged files written under `out`; returns `out`."""
    try:
        sim.extract(guide, out)
    except sim.GuideError as e:
        raise DeploymentError(str(e)) from e
    return out


def night(
    guide_dir: Path, work: Path, path: Sequence[Path], env: Mapping[str, str],
) -> subprocess.CompletedProcess[str]:
    """The job run once in `work`, with `env` over a minimal environment and
    the directories of `path` ahead of the system's."""
    proc_env = {
        "PATH": ":".join([*map(str, path), "/usr/local/bin", "/usr/bin", "/bin"]),
        "HOME": str(work),
        "LC_ALL": "C",
        **env,
    }
    work.mkdir(parents=True, exist_ok=True)
    return subprocess.run(
        ["bash", str(guide_dir / NIGHTLY)], cwd=work, env=proc_env, capture_output=True, text=True,
    )


def serve_directory(directory: Path, port: int) -> http.server.ThreadingHTTPServer:
    """`directory` served at `/` on a background thread, until `shutdown()` and `server_close()`."""

    class Quiet(http.server.SimpleHTTPRequestHandler):
        def log_message(self, format: str, *args: object) -> None:
            pass

    try:
        server = http.server.ThreadingHTTPServer(
            ("127.0.0.1", port), functools.partial(Quiet, directory=str(directory)),
        )
    except OSError as err:
        raise DeploymentError(f"cannot serve {directory} on port {port}: {err}") from err
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


def pull(guide_dir: Path, url: str, dest: Path, work: Path) -> subprocess.CompletedProcess[str]:
    """The pull script over `url`, into `dest`."""
    work.mkdir(parents=True, exist_ok=True)
    return subprocess.run(
        ["bash", str(guide_dir / PULL)],
        cwd=work,
        env={"PATH": "/usr/local/bin:/usr/bin:/bin", "URL": url, "DEST": str(dest), "HOME": str(work)},
        capture_output=True,
        text=True,
    )


def sums(directory: Path) -> dict[str, str]:
    """`SHA256SUMS` as name to sum."""
    listed = {}
    for line in (directory / "SHA256SUMS").read_text().splitlines():
        digest, name = line.split(None, 1)
        listed[name.removeprefix("*")] = digest
    return listed


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_sums(directory: Path) -> list[str]:
    """Every way `directory` does not hold what its `SHA256SUMS` lists, as
    `sha256sum -c` would say, empty when there is none."""
    if not (directory / "SHA256SUMS").is_file():
        return [f"{directory} has no SHA256SUMS"]
    problems = []
    for name, want in sums(directory).items():
        if not (directory / name).is_file():
            problems.append(f"SHA256SUMS lists {name}, which is not there")
        elif sha256(directory / name) != want:
            problems.append(f"{name} does not hash to its line in SHA256SUMS")
    return problems


def check_published(pub: Path) -> list[str]:
    """Every way `pub` is not the contract's directory, empty when there is none."""
    problems = check_sums(pub)
    if not (pub / "SHA256SUMS").is_file():
        return problems
    listed = sums(pub)
    for fixed in FIXED:
        if not (pub / fixed).is_file():
            problems.append(f"no {fixed}")
    for pattern in BY_CONTENT:
        named = [n for n in listed if pattern.fullmatch(n)]
        if len(named) != 1:
            problems.append(f"SHA256SUMS lists {len(named)} files matching {pattern.pattern}, not one")
            continue
        # The name is the content's: its 16 hex digits are the file's own.
        if (pub / named[0]).is_file() and sha256(pub / named[0])[:16] not in named[0]:
            problems.append(f"{named[0]} is not named for its content")
    for name in listed:
        if "/" in name and not name.startswith("digests/"):
            problems.append(f"SHA256SUMS lists {name}, outside the contract's directories")
    for loose in sorted(p.name for p in pub.iterdir() if p.is_file()):
        if loose in ("SHA256SUMS", *FIXED):
            continue
        if loose not in listed and not any(pattern.fullmatch(loose) for pattern in BY_CONTENT):
            problems.append(f"{loose} is neither a fixed name, a file by content nor listed")
    return problems


def release_archive(binary: Path, dest: Path) -> Path:
    """`binary` archived as a release is: a directory of its own holding `bruce`."""
    with tarfile.open(dest, "w:gz") as tar:
        tar.add(binary, arcname="bruce-v0-x86_64-unknown-linux-gnu/bruce")
    return dest


def exercise(binary: Path, config: Path, work: Path) -> list[str]:
    """The guide's job run twice with the real `binary` over the site `config` crawls,
    and its pull over what the second night published; every way the result is not
    the guide's contract, empty when there is none."""
    problems: list[str] = []
    guide = extract(REPO / sim.GUIDE, work / "guide")
    bindir = work / "bin"
    bindir.mkdir(parents=True, exist_ok=True)
    (bindir / "bruce").unlink(missing_ok=True)
    (bindir / "bruce").symlink_to(binary)
    env = {
        "ARCHIVE": str(release_archive(binary, work / "release.tar.gz")),
        "CONFIG": str(config),
        "DB": "nightly.duckdb",
        "PUB": "pub",
        "KEEP": "30",
    }
    run = work / "run"
    shutil.rmtree(run, ignore_errors=True)
    pub = run / "pub"
    first_names: set[str] = set()
    for night_no in (1, 2):
        done = night(guide, run, [bindir], env)
        if done.returncode != 0:
            return problems + [f"night {night_no} exited {done.returncode}:\n{done.stdout}{done.stderr}"]
        problems += [f"night {night_no}: {p}" for p in check_published(pub)]
        compared = json.loads((pub / "digest.json").read_text())["compared_with"]
        if night_no == 1:
            first_names = set(sums(pub))
            if compared is not None:
                problems.append(f"night 1's digest compares with {compared}, there being no baseline")
        else:
            if not (compared and compared["from_file"]):
                problems.append(f"night 2's digest does not compare with the baseline file: {compared}")
            for name in first_names:
                if not name.startswith("digest") and not (pub / name).is_file():
                    problems.append(f"night 1's {name} did not stay one more night")
    if (pub / "bruce.toml").read_text() != config.read_text():
        problems.append("the published bruce.toml is not the config the crawl ran under")

    server = serve_directory(pub, 0)
    try:
        dest = work / "served"
        url = f"http://127.0.0.1:{server.server_address[1]}"
        shutil.rmtree(dest, ignore_errors=True)
        done = pull(guide, url, dest, work / "pull")
        if done.returncode != 0:
            return problems + [f"the pull exited {done.returncode}:\n{done.stdout}{done.stderr}"]
        if sums(dest) != sums(pub):
            problems.append("the pulled SHA256SUMS is not the published one")
        problems += [f"pulled: {p}" for p in check_sums(dest)]
        if {p.relative_to(dest).as_posix() for p in dest.rglob("*") if p.is_file()} != {*sums(dest), "SHA256SUMS"}:
            problems.append("the pulled directory holds more or less than SHA256SUMS lists")
        before = (dest / "SHA256SUMS").stat().st_ino
        again = pull(guide, url, dest, work / "pull")
        if again.returncode != 0 or (dest / "SHA256SUMS").stat().st_ino != before:
            problems.append(f"a second pull of an unchanged set changed the directory (exit {again.returncode})")
    finally:
        server.shutdown()
        server.server_close()
    return problems
