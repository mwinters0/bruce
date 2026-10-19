#!/usr/bin/env python3
"""The simulated deployment: the deployment guide's tagged blocks extracted
into `sim/`, and the config checked to load (D41, D42).

`docs/manual/deploy.md` is the guide a deployer follows. A fenced block whose
info string is `<lang> file=<path>` is **tagged**: its text belongs to `<path>`
under `sim/`. Blocks naming one path are joined in the order the guide gives
them, a blank line between, so a config can be built up section by section
beside the prose explaining each. Untagged blocks (a command to type, an
output) are the reader's and are never extracted. A guide that drifts from the
tool fails here, and any machine rebuilds `sim/` from the guide alone.

What it checks: the guide has tagged blocks, every path stays inside `sim/`,
`bruce.toml` is among them, and `cargo run --example sim_config -- bruce.toml`
accepts it. That example loads a config as `bruce crawl --config` does, with no
request sent. **The crawl is not run**: at one to two hours it is launched by
hand from `sim/` (`CLAUDE.md`, "Long-running processes").

Only the files the guide names are written; whatever else `sim/` holds, a
crawl's logs included, is left alone. `sim/` is gitignored.

Usage:

    mise run sim
    cd scripts && uv run sim.py [--guide FILE] [--out DIR] [--no-load]
    cd scripts && uv run python -m unittest test_sim
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Sequence, TextIO

REPO = Path(__file__).resolve().parent.parent
GUIDE = Path("docs/manual/deploy.md")
OUT = Path("sim")
#: The tagged file the load check reads.
CONFIG = "bruce.toml"

FENCE_RE = re.compile(r"^(`{3,})(.*)$")
TAG_RE = re.compile(r"(?:^|\s)file=(\S+)")


class GuideError(Exception):
    """The guide cannot be extracted; the message says where and why."""


def tagged_blocks(text: str, name: str = "guide") -> dict[str, list[str]]:
    """Each tagged path to its blocks' texts, in the order the guide gives
    them. A block closes at a fence of at least its opener's length and no
    info string; one never closed is an error."""
    blocks: dict[str, list[str]] = {}
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        m = FENCE_RE.match(lines[i])
        if not m:
            i += 1
            continue
        fence, info = m.group(1), m.group(2)
        start = i + 1
        body: list[str] = []
        i += 1
        while i < len(lines):
            close = FENCE_RE.match(lines[i])
            if close and close.group(2).strip() == "" and len(close.group(1)) >= len(fence):
                break
            body.append(lines[i])
            i += 1
        else:
            raise GuideError(f"{name}:{start}: a code block is never closed")
        i += 1
        tag = TAG_RE.search(info)
        if tag:
            blocks.setdefault(tag.group(1), []).append("\n".join(body) + "\n")
    return blocks


def safe_path(path: str) -> PurePosixPath:
    """`path` as a relative path staying under `sim/`, or a GuideError."""
    p = PurePosixPath(path)
    if p.is_absolute() or ".." in p.parts or not p.parts:
        raise GuideError(f"file={path}: a tagged path is relative and stays inside sim/")
    return p


def extract(guide: Path, out: Path) -> list[Path]:
    """Writes the guide's tagged files under `out`; returns what it wrote."""
    try:
        text = guide.read_text(encoding="utf-8")
    except OSError as e:
        raise GuideError(f"{guide}: {e}") from e
    blocks = tagged_blocks(text, str(guide))
    if not blocks:
        raise GuideError(f"{guide}: no code block is tagged `file=<path>`")
    if CONFIG not in blocks:
        raise GuideError(f"{guide}: no code block is tagged `file={CONFIG}`")
    written = []
    for path, parts in blocks.items():
        target = out.joinpath(*safe_path(path).parts)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("\n".join(parts), encoding="utf-8")
        written.append(target)
    return written


def load_command(config: Path) -> list[str]:
    return ["cargo", "run", "--quiet", "--example", "sim_config", "--", str(config)]


def run(guide: Path, out: Path, load: bool, stdout: TextIO = sys.stdout) -> int:
    try:
        written = extract(guide, out)
    except GuideError as e:
        print(f"sim: {e}", file=sys.stderr)
        return 1
    for path in written:
        print(f"sim: wrote {path}", file=stdout)
    if not load:
        return 0
    done = subprocess.run(load_command(out / CONFIG), cwd=REPO)
    if done.returncode != 0:
        print(f"sim: {out / CONFIG} does not load", file=sys.stderr)
        return 1
    print(f"sim: {out / CONFIG} loads", file=stdout)
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--guide", type=Path, default=REPO / GUIDE, help="the deployment guide")
    parser.add_argument("--out", type=Path, default=REPO / OUT, help="where the files go")
    parser.add_argument("--no-load", action="store_true", help="extract only; skip the load check")
    args = parser.parse_args(argv)
    return run(args.guide, args.out, load=not args.no_load)


if __name__ == "__main__":
    raise SystemExit(main())
