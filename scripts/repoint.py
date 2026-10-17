#!/usr/bin/env python3
"""The repoint check: the record's caps, and how far the live record has grown
since it was last read against the code.

`docs/process.md`, "Repointing" is the rule. Nothing between one keystone and
the next reads the standing record against the code, and nothing caps what each
landing adds to it, so a record left alone grows faster than anyone re-reads
it and its claims rot unseen. Both halves are mechanical, so both are here:

* **The caps.** `decisions.md` at its line cap and seven lines an entry; every
  dated entry at the length `docs/status/history/README.md` states;
  `CLAUDE.md`, `docs/process.md` and each skill at the sizes that keep
  orientation cheap. Adding under a full one means striking; a session never
  raises a cap to fit what it is landing, and only the maintainer moves one.
* **No narration where a conclusion belongs.** No phase, ledger or date
  provenance in a source comment (a comment states the contract and cites
  `D<k>`).
* **The meter.** `STATUS.md` carries `<!-- repointed: <sha> -->`, the commit
  the record was last read against. The net growth since then of the *live*
  record -- every standing `.md` outside `history/`, the frozen
  `historical/` and the phase docs, the skills, and the comment lines of every
  workspace member's `.rs` -- is reported, and past the budget the check is
  red. Red is the trigger for a repoint; the phase docs are centering and
  history is deleted at a keystone, so neither is counted. **`none` is no
  baseline yet**: before any commit has been read against, growth is measured
  from the empty tree.

Usage:

    cd scripts
    uv run repoint.py [--stamp <sha>] [--budget <lines>]
    uv run python -m unittest test_repoint
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

REPO = Path(__file__).resolve().parent.parent

#: The register's caps: `docs/process.md`, "The decision register".
REGISTER_LINES = 700
ENTRY_LINES = 7
#: The one statement of this is `docs/status/history/README.md`; this is that
#: number, held against every entry.
HISTORY_LINES = 350
#: Orientation is what the read-triggers pull into context, so these are the
#: caps that decide what a session costs before it has done anything.
ORIENTATION_CAPS = {
    "CLAUDE.md": 150,
    "docs/process.md": 550,
}
SKILL_LINES = 225
#: Net lines the live record may grow between repoints before this goes red.
#: Under the caps and the citation rule that is many sessions' work. Tune it,
#: never to fit.
GROWTH_BUDGET = 3000

#: `none` is the stamp before any commit has been read against (`NO_BASELINE`).
STAMP_RE = re.compile(r"<!-- repointed: ([0-9a-f]{7,40}|none) -->")
NO_BASELINE = "none"
#: How the meter names the base it measured from when there is no stamp.
EMPTY_BASE = "the empty tree"
ENTRY_RE = re.compile(r"^### (D\d+)\b")
HISTORY_NAME_RE = re.compile(r"^(\d{4}-\d{2}-\d{2})\.md$")
#: Provenance: a slice, a ledger row, a date. `D<k>`, `KD<k>`, `I<n>`, `S<k>`
#: and `UF<k>` are handles, not provenance, and are what a comment should carry.
PROVENANCE_RE = re.compile(r"\bP\d+\.\d+\b|\bM\d{1,3}\b|\b20[2-9]\d-\d\d-\d\d\b")
COMMENT_RE = re.compile(r"^\s*//")
#: Never this project's record, though a root package's tree holds them.
NOT_SOURCE = ("vendor/", "target/")


@dataclass(frozen=True)
class Growth:
    docs: int
    comments: int

    @property
    def total(self) -> int:
        return self.docs + self.comments


def _lines(path: Path) -> list[str]:
    return path.read_text(encoding="utf-8").splitlines()


def register_problems(repo: Path) -> list[str]:
    path = repo / "docs" / "design" / "decisions.md"
    if not path.exists():
        return [f"{path.relative_to(repo)}: missing"]
    lines = _lines(path)
    problems = []
    if len(lines) > REGISTER_LINES:
        problems.append(
            f"decisions.md is {len(lines)} lines against a cap of {REGISTER_LINES}"
        )
    entry, body = None, 0
    for line in lines + ["## end"]:
        if line.startswith("## ") or ENTRY_RE.match(line):
            if entry and body > ENTRY_LINES:
                problems.append(
                    f"decisions.md {entry} runs {body} lines against {ENTRY_LINES}"
                )
            m = ENTRY_RE.match(line)
            entry, body = (m.group(1) if m else None), 0
        elif line.strip():
            body += 1
    return problems


def history_problems(repo: Path) -> list[str]:
    root = repo / "docs" / "status" / "history"
    problems = []
    for path in sorted(root.glob("*.md")) if root.exists() else []:
        m = HISTORY_NAME_RE.match(path.name)
        if not m:
            continue
        n = len(_lines(path))
        if n > HISTORY_LINES:
            problems.append(f"history/{path.name} is {n} lines against {HISTORY_LINES}")
    return problems


def orientation_problems(repo: Path) -> list[str]:
    problems = []
    for rel, cap in ORIENTATION_CAPS.items():
        path = repo / rel
        if path.exists() and len(_lines(path)) > cap:
            problems.append(f"{rel} is {len(_lines(path))} lines against {cap}")
    for path in sorted((repo / ".claude" / "skills").glob("*/SKILL.md")):
        n = len(_lines(path))
        if n > SKILL_LINES:
            problems.append(
                f"{path.relative_to(repo)} is {n} lines against {SKILL_LINES}"
            )
    return problems


def roots_of(manifest: str) -> tuple[str, ...]:
    """The workspace members' roots a `Cargo.toml` lists, `.` for a root
    package; a glob's root is the directory above its `*`."""
    try:
        parsed = tomllib.loads(manifest)
    except tomllib.TOMLDecodeError:
        return ()
    roots = ["."] if "package" in parsed else []
    for m in parsed.get("workspace", {}).get("members", []):
        root = m.split("*", 1)[0].rstrip("/") or "."
        if root not in roots:
            roots.append(root)
    return tuple(roots)


def code_roots(repo: Path) -> tuple[str, ...]:
    """The live tree's member roots; none before a `Cargo.toml` exists."""
    path = repo / "Cargo.toml"
    return roots_of(path.read_text(encoding="utf-8")) if path.is_file() else ()


def _source_files(repo: Path) -> Iterable[Path]:
    roots = code_roots(repo)
    for path in sorted({p for root in roots for p in (repo / root).rglob("*.rs")}):
        if is_source(path.relative_to(repo).as_posix(), roots):
            yield path


def comment_problems(repo: Path) -> list[str]:
    problems = []
    for path in _source_files(repo):
        for n, line in enumerate(_lines(path), 1):
            if COMMENT_RE.match(line) and PROVENANCE_RE.search(line):
                problems.append(
                    f"{path.relative_to(repo)}:{n} carries provenance: {line.strip()}"
                )
    return problems


def read_stamp(repo: Path) -> str | None:
    """The one stamp's commit, or `NO_BASELINE`; `None` for zero or two."""
    path = repo / "docs" / "status" / "STATUS.md"
    if not path.exists():
        return None
    found = STAMP_RE.findall(path.read_text(encoding="utf-8"))
    return found[0] if len(found) == 1 else None


def _git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=repo, check=True, capture_output=True, text=True
    ).stdout


def is_live_doc(path: str) -> bool:
    if not path.endswith(".md") or path.startswith("vendor/"):
        return False
    # History and the frozen bootstrap plan are never raked; the phase docs
    # (in flight under `docs/wip/`, wrapped or inboxes beside the roadmap) are
    # centering.
    if "/history/" in path or path.startswith(("docs/design/historical/", "docs/wip/")):
        return False
    if re.match(r"docs/design/roadmap-P\d+[.-]", path):
        return False
    return (
        path.startswith("docs/")
        or path.startswith(".claude/skills/")
        or path in ("CLAUDE.md", "README.md", "CONTRIBUTING.md")
    )


def is_source(path: str, roots: Sequence[str]) -> bool:
    """A `.rs` under a member root, never a vendored copy or a build's output."""
    if not path.endswith(".rs") or path.startswith(NOT_SOURCE) or "/target/" in path:
        return False
    return "." in roots or path.startswith(tuple(f"{root}/" for root in roots))


def members_at(repo: Path, rev: str) -> tuple[str, ...]:
    """The workspace members' roots as `rev`'s manifest lists them: a member
    since moved was a source where it stood, so a file whose move git does not
    pair with its old path is subtracted there."""
    try:
        return roots_of(_git(repo, "show", f"{rev}:Cargo.toml"))
    except subprocess.CalledProcessError:
        return ()


def empty_tree(repo: Path) -> str:
    """The id of the empty tree in this repository's hash."""
    return subprocess.run(
        ["git", "hash-object", "-t", "tree", "--stdin"],
        cwd=repo, check=True, capture_output=True, text=True, input="",
    ).stdout.strip()


def growth_since(repo: Path, stamp: str) -> Growth:
    """Net lines added to the live record between `stamp` and the working tree,
    untracked files included, so a round is measured before it is committed."""
    docs = 0
    for row in _git(repo, "diff", "--numstat", stamp, "--", ".").splitlines():
        added, deleted, path = row.split("\t", 2)
        if added != "-" and is_live_doc(path):
            docs += int(added) - int(deleted)
    comments = 0
    # `is_source` per file, as the untracked pass below does: a `.rs` outside
    # the members is not this project's record, and a re-vendor of a frozen
    # copy would otherwise spend the budget on comments no repoint may rake.
    live = code_roots(repo)
    roots = (*live, *members_at(repo, stamp))
    old, counting = "", False
    for line in _git(repo, "diff", "-U0", stamp, "--", "*.rs").splitlines():
        if line.startswith("--- "):
            old = line[6:] if line.startswith("--- a/") else ""
            continue
        if line.startswith("+++ "):
            new = line[6:] if line.startswith("+++ b/") else ""
            counting = is_source(new or old, roots)
            continue
        if counting and line[:1] in "+-" and COMMENT_RE.match(line[1:]):
            comments += 1 if line[0] == "+" else -1
    for path in _git(repo, "ls-files", "--others", "--exclude-standard").splitlines():
        lines = _lines(repo / path)
        if is_live_doc(path):
            docs += len(lines)
        elif is_source(path, live):
            comments += sum(1 for line in lines if COMMENT_RE.match(line))
    return Growth(docs, comments)


def meter_problems(repo: Path, stamp: str | None, budget: int, out) -> list[str]:
    if stamp is None:
        return ["STATUS.md carries no single `<!-- repointed: <sha> -->` stamp"]
    if stamp == NO_BASELINE:
        # No commit has been read against yet: everything is growth.
        base, named = empty_tree(repo), EMPTY_BASE
    else:
        try:
            _git(repo, "merge-base", "--is-ancestor", stamp, "HEAD")
        except subprocess.CalledProcessError:
            return [f"the repoint stamp {stamp} is not an ancestor of HEAD"]
        base, named = stamp, stamp
    g = growth_since(repo, base)
    print(
        f"since {named}: live docs {g.docs:+d}, source comments {g.comments:+d}, "
        f"net {g.total:+d} of a {budget}-line budget",
        file=out,
    )
    if g.total > budget:
        return [f"the live record has grown {g.total} lines since {named}; repoint"]
    return []


def check(
    repo: Path = REPO, stamp: str | None = None, budget: int = GROWTH_BUDGET, out=sys.stdout
) -> int:
    problems = (
        register_problems(repo)
        + history_problems(repo)
        + orientation_problems(repo)
        + comment_problems(repo)
        + meter_problems(repo, stamp or read_stamp(repo), budget, out)
    )
    for p in problems:
        print(f"!! {p}", file=out)
    print("repoint: " + ("red" if problems else "green"), file=out)
    return 1 if problems else 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--repo", type=Path, default=REPO, help="repository root")
    parser.add_argument("--stamp", help="measure growth from this commit instead of the stamp")
    parser.add_argument("--budget", type=int, default=GROWTH_BUDGET)
    args = parser.parse_args(argv)
    return check(args.repo.resolve(), args.stamp, args.budget)


if __name__ == "__main__":
    sys.exit(main())
