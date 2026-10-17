#!/usr/bin/env python3
"""The concept map held to the tree: every production file claimed once.

`docs/design/concepts.md` names each concept the code holds and the modules
holding it, crate by crate (`docs/process.md`, "Seams"). This holds the map to
the workspace:

* a **claim** is a bullet `- **<crate>** — …` under a concept's `## ` heading,
  its wrapped lines included; every backticked path in it ending `.rs` or `/`
  is claimed, relative to that crate's `src/`, the crate being a workspace
  member by its package name. A directory claims every file under it and
  the `<dir>.rs` beside it;
* every **production file** -- a `.rs` under a member's `src/` that is not a
  test file by `file_limits.py`'s reading -- is claimed by exactly one
  concept, a crate root (`lib.rs`, `main.rs`) excepted unless claimed;
* every claimed path names a production file or directory of that crate, and
  every crate named is a workspace member;
* every concept is headed once and claims something.

With no `Cargo.toml` there is no member and no production file, so a map
listing no concept passes.

What a concept is, and whether a file is in the right one, is the review's;
this checks only that the map and the tree agree. Prose outside a claim may
name any path, including another crate's.

Usage:

    cd scripts && uv run concepts.py
    cd scripts && uv run python -m unittest test_concepts
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path
from typing import TextIO

import file_limits

REPO = Path(__file__).resolve().parent.parent
MAP = "docs/design/concepts.md"

HEADING_RE = re.compile(r"^## (.+?)\s*$")
CLAIM_RE = re.compile(r"^- \*\*([\w-]+)\*\* — ")
PATH_RE = re.compile(r"`([\w./-]+(?:\.rs|/))`")
ROOTS = ("lib.rs", "main.rs")
NOT_IN_ONE_PLACE = "**Not in one place.**"


def claims(text: str) -> tuple[dict[str, list[tuple[str, str, int]]], list[str], list[str]]:
    """Each concept to its claims `(crate, path, line)`, the concepts with a
    "Not in one place" paragraph, and the problems found reading the map."""
    found: dict[str, list[tuple[str, str, int]]] = {}
    scattered: list[str] = []
    problems: list[str] = []
    concept: str | None = None
    crate: str | None = None
    for n, line in enumerate(text.splitlines(), 1):
        if m := HEADING_RE.match(line):
            concept, crate = m.group(1), None
            if concept in found:
                problems.append(f"{MAP}:{n}: concept {concept!r} is headed twice")
            found[concept] = []
            continue
        if line.startswith("#"):
            concept, crate = None, None
            continue
        if m := CLAIM_RE.match(line):
            if concept is None:
                problems.append(f"{MAP}:{n}: a claim outside any concept")
                crate = None
                continue
            crate = m.group(1)
        elif not (crate and line.startswith("  ")):
            crate = None
        if concept is not None and line.startswith(NOT_IN_ONE_PLACE):
            scattered.append(concept)
        if crate is not None and concept is not None:
            for p in PATH_RE.findall(line):
                found[concept].append((crate, p, n))
    return found, scattered, problems


def crate_name(member: Path) -> str:
    """A member's package name, or its directory's name where its manifest
    names none."""
    try:
        manifest = tomllib.loads((member / "Cargo.toml").read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        manifest = {}
    return manifest.get("package", {}).get("name") or member.name


def source_dirs(repo: Path) -> dict[str, str]:
    """Each member's crate name to its `src/`, repo-relative with a trailing
    `/` (`src/` alone for a root package)."""
    out: dict[str, str] = {}
    for m in file_limits.members(repo):
        rel = m.relative_to(repo).as_posix()
        out[crate_name(m)] = "src/" if rel == "." else f"{rel}/src/"
    return out


def production_files(repo: Path) -> dict[str, set[str]]:
    """Each member's crate name to its production files, relative to its
    `src/`."""
    dirs = source_dirs(repo)
    out: dict[str, set[str]] = {name: set() for name in dirs}
    # The deepest `src/` holding a file is its crate's, a root package's being
    # the shallowest.
    by_depth = sorted(dirs.items(), key=lambda kv: -len(kv[1]))
    for r in file_limits.readings(repo):
        if r.is_test:
            continue
        home = next(((n, d) for n, d in by_depth if r.path.startswith(d)), None)
        if home is not None:
            out[home[0]].add(r.path[len(home[1]) :])
    return out


def resolve(claim: str, files: set[str]) -> set[str]:
    """The files a claimed path names: itself, or a directory's contents and
    the `<dir>.rs` beside it."""
    if claim.endswith("/"):
        stem = claim.rstrip("/")
        return {f for f in files if f.startswith(claim) or f == f"{stem}.rs"}
    return {claim} & files


def check(repo: Path = REPO, out: TextIO = sys.stdout) -> int:
    path = repo / MAP
    if not path.exists():
        print(f"error: {MAP} is missing", file=out)
        return 1
    found, scattered, problems = claims(path.read_text(encoding="utf-8"))
    files = production_files(repo)
    src = source_dirs(repo)
    owner: dict[tuple[str, str], str] = {}
    for concept, held in found.items():
        if not held:
            problems.append(f"{MAP}: concept {concept!r} claims nothing")
        for crate, claim, n in held:
            if crate not in files:
                problems.append(f"{MAP}:{n}: {crate} is not a workspace member")
                continue
            named = resolve(claim, files[crate])
            if not named:
                problems.append(
                    f"{MAP}:{n}: {crate}'s `{claim}` names no production file"
                )
            for f in sorted(named):
                if (crate, f) in owner:
                    problems.append(
                        f"{MAP}:{n}: {src[crate]}{f} is claimed by "
                        f"{owner[crate, f]!r} and {concept!r}"
                    )
                else:
                    owner[crate, f] = concept
    for crate, held in sorted(files.items()):
        for f in sorted(held):
            if (crate, f) not in owner and f not in ROOTS:
                problems.append(f"{src[crate]}{f} is claimed by no concept in {MAP}")
    if problems:
        for p in problems:
            print(f"error: {p}", file=out)
        print(f"concepts: {len(problems)} problem(s)", file=out)
        return 1
    spanning = sum(1 for held in found.values() if len({c for c, _, _ in held}) > 1)
    print(
        f"concepts: {len(found)} concepts claim {len(owner)} files once each; "
        f"{spanning} span crates, {len(scattered)} not in one place",
        file=out,
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--repo", type=Path, default=REPO, help="repository root (tests use this)")
    args = parser.parse_args(argv)
    return check(args.repo)


if __name__ == "__main__":
    raise SystemExit(main())
