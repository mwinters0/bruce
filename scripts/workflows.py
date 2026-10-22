#!/usr/bin/env python3
"""The workflows under `.github/workflows/`: no third-party action in any, and
a workflow's steps printed as one shell script to run by hand.

**No workflow uses an action** (`docs/design/roadmap.md`, "Standing rules"):
every step is a command in the job's container, a checkout included. A
`uses:` key written as a block key, a step's or a job's (a reusable
workflow), fails the check; a comment naming one does not, and nor does one
in a flow mapping (`{uses: …}`), which no workflow here writes. What an action
would give is built from commands, so a step runs the same on a maintainer's
machine.

`script` is that second half. A workflow's `run:` steps are joined, in order,
each in a subshell with `set -e`, in its `working-directory` under
`$GITHUB_WORKSPACE`, so a release's build is rehearsed in a `debian:trixie`
container before a tag is pushed. The runner supplies `GITHUB_REF_NAME`,
`GITHUB_SHA`, `GITHUB_REPOSITORY` and the rest, and `env:` is not read, so the
caller sets what its steps use. `--skip` leaves a step out and `--before`
stops ahead of one, each by name.

The parser reads the subset the repo's workflows are written in: the first
`steps:` list, whose steps carry `name:`, `working-directory:` and `run:` as a
scalar or a `|` block; a later job's steps are not read. A step's other keys
(`if:`, `shell:`, `env:`) are dropped, a nested mapping's keys read as the
step's own. A step with no `run:`, a line that is no `key: value` and a folded
block are errors.

Usage:

    cd scripts && uv run workflows.py [--dir DIR]
    cd scripts && uv run workflows.py script release.yml [--skip NAME]... [--before NAME]
    cd scripts && uv run python -m unittest test_workflows
"""

from __future__ import annotations

import argparse
import re
import shlex
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence, TextIO

REPO = Path(__file__).resolve().parent.parent
WORKFLOWS = Path(".github/workflows")

USES_RE = re.compile(r"""^\s*(?:-\s+)?["']?uses["']?\s*:""")
STEP_RE = re.compile(r"^(\s*)-\s+(.*)$")
KEY_RE = re.compile(r"^([A-Za-z][\w-]*)\s*:\s*(.*)$")


class WorkflowError(Exception):
    """A workflow cannot be read; the message says where and why."""


@dataclass(frozen=True)
class Step:
    name: str
    run: str
    directory: str


def uses(text: str) -> list[int]:
    """The 1-based lines holding a `uses:` key."""
    return [n for n, line in enumerate(text.splitlines(), 1) if USES_RE.match(line)]


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def steps(text: str, name: str = "workflow") -> list[Step]:
    """The `run:` steps of a workflow, in order. A step with no `run:` is an
    error: it would be an action, and nothing here runs one."""
    lines = text.splitlines()
    out: list[Step] = []
    i = 0
    while i < len(lines) and not re.match(r"^\s*steps\s*:\s*$", lines[i]):
        i += 1
    i += 1
    while i < len(lines):
        m = STEP_RE.match(lines[i])
        if not m:
            if lines[i].strip() and not lines[i].lstrip().startswith("#"):
                break
            i += 1
            continue
        base = len(m.group(1))
        start = i + 1
        fields: dict[str, str] = {}
        # The step's first key sits on the dash's line; the rest are indented
        # two past the dash.
        pending = [(i, m.group(2))]
        i += 1
        while i < len(lines):
            line = lines[i]
            if not line.strip() or line.lstrip().startswith("#"):
                i += 1
                continue
            if _indent(line) <= base:
                break
            pending.append((i, line.strip()))
            i += 1
        j = 0
        while j < len(pending):
            at, entry = pending[j]
            k = KEY_RE.match(entry)
            if not k:
                raise WorkflowError(f"{name}:{at + 1}: not a `key: value` line: {entry!r}")
            key, value = k.group(1), k.group(2)
            j += 1
            if value in ("|", "|-", ">", ">-"):
                if value[0] == ">":
                    raise WorkflowError(f"{name}:{at + 1}: a folded `{key}` is not read")
                body: list[str] = []
                first = pending[j - 1][0] + 1
                while first < len(lines) and (
                    not lines[first].strip() or _indent(lines[first]) > base + 2
                ):
                    body.append(lines[first])
                    first += 1
                while body and not body[-1].strip():
                    body.pop()
                width = min((_indent(b) for b in body if b.strip()), default=0)
                fields[key] = "\n".join(b[width:] for b in body)
                while j < len(pending) and pending[j][0] < first:
                    j += 1
            else:
                value = value.strip()
                if key != "run" and len(value) > 1 and value[0] == value[-1] and value[0] in "\"'":
                    value = value[1:-1]
                fields[key] = value
        if "run" not in fields:
            raise WorkflowError(f"{name}:{start}: a step with no `run:`")
        out.append(Step(fields.get("name", f"step {len(out) + 1}"), fields["run"], fields.get("working-directory", ".")))
    return out


def script(chosen: Sequence[Step]) -> str:
    """The steps as one bash script, each a subshell under `set -e`."""
    parts = ["#!/usr/bin/env bash", "set -eu", ': "${GITHUB_WORKSPACE:=$PWD}"']
    for s in chosen:
        parts.append(f"\necho {shlex.quote('=== ' + s.name)}")
        parts.append("(")
        parts.append("set -e")
        parts.append(f'cd "$GITHUB_WORKSPACE"/{shlex.quote(s.directory)}')
        parts.append(s.run)
        parts.append(")")
    return "\n".join(parts) + "\n"


def select(all_steps: Sequence[Step], skip: Sequence[str], before: str | None) -> list[Step]:
    names = [s.name for s in all_steps]
    for wanted in [*skip, *([before] if before else [])]:
        if wanted not in names:
            raise WorkflowError(f"no step named {wanted!r}; the steps are {names}")
    chosen: list[Step] = []
    for s in all_steps:
        if s.name == before:
            break
        if s.name not in skip:
            chosen.append(s)
    return chosen


def workflow_files(repo: Path, directory: Path = WORKFLOWS) -> list[Path]:
    root = repo / directory
    if not root.is_dir():
        return []
    return sorted(p for p in root.iterdir() if p.suffix in (".yml", ".yaml"))


def check(repo: Path = REPO, directory: Path = WORKFLOWS, out: TextIO = sys.stdout) -> int:
    files = workflow_files(repo, directory)
    problems: list[str] = []
    for f in files:
        text = f.read_text()
        rel = f.relative_to(repo)
        for n in uses(text):
            problems.append(f"{rel}:{n}: `uses:`, and no workflow uses an action")
        if not uses(text):
            try:
                steps(text, str(rel))
            except WorkflowError as e:
                problems.append(str(e))
    for p in problems:
        print(f"error: {p}", file=out)
    if problems:
        print(f"workflows: {len(problems)} problem(s)", file=out)
        return 1
    print(f"workflows: {len(files)} checked, no action used", file=out)
    return 0


def main(argv: Sequence[str] | None = None, out: TextIO = sys.stdout) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--dir", type=Path, default=WORKFLOWS, help="workflows directory, relative to the repo")
    sub = ap.add_subparsers(dest="command")
    sc = sub.add_parser("script", help="print a workflow's steps as one shell script")
    sc.add_argument("workflow", help="a file name under the workflows directory")
    sc.add_argument("--skip", action="append", default=[], metavar="NAME", help="leave a step out")
    sc.add_argument("--before", metavar="NAME", help="stop ahead of a step")
    args = ap.parse_args(argv)
    if args.command != "script":
        return check(REPO, args.dir, out)
    try:
        path = REPO / args.dir / args.workflow
        chosen = select(steps(path.read_text(), args.workflow), args.skip, args.before)
    except (OSError, WorkflowError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    out.write(script(chosen))
    return 0


if __name__ == "__main__":
    sys.exit(main())
