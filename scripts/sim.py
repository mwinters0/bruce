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

A block may name more than one path (`toml file=a.toml file=b.toml`), and then
belongs to each: the weekly every-locale crawl's config shares the nightly one's
sections and differs in its root.

What it checks: the guide has tagged blocks, every path stays inside `sim/`,
`bruce.toml` and `bruce-locales.toml` are among them, and `cargo run --example
sim_config -- FILE` accepts each. That example loads a config as `bruce crawl
--config` does, with no request sent. Each script (`*.sh`) is parsed by `bash -n`
and made executable, and the systemd units are wired: a timer names a service
template whose environment file and script the guide also tags. The CI workflow
is read as `workflows.py` reads one, its steps being what `e2e.py` runs, and a
`uses:` fails it (`WORKFLOW`); `e2e.py` also parses it as YAML, which this check,
standard library only, cannot. The `bruce` lines of the scripts and of the
workflow are held to the command line by `cli::tests`, which reads the same
guide. **The crawl is not run**: at one to two hours it is launched detached from
`sim/` by `mise run crawl` (`review.py`); `e2e.py` runs the scripts over the
fixture site.

Only the files the guide names are written; whatever else `sim/` holds, a
crawl's logs included, is left alone. `sim/` is gitignored.

Usage:

    mise run sim
    cd scripts && uv run sim.py [--guide FILE] [--out DIR] [--no-load]
    cd scripts && uv run python -m unittest test_sim
"""

from __future__ import annotations

import argparse
import configparser
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Sequence, TextIO

import workflows

REPO = Path(__file__).resolve().parent.parent
GUIDE = Path("docs/manual/deploy.md")
OUT = Path("sim")
#: The tagged files the load check reads.
CONFIG = "bruce.toml"
CONFIGS = (CONFIG, "bruce-locales.toml")
#: The CI workflow the guide tags, where a content repo keeps it.
WORKFLOW = ".github/workflows/links.yml"

FENCE_RE = re.compile(r"^(`{3,})(.*)$")
TAG_RE = re.compile(r"(?:^|\s)file=(\S+)")
#: A unit's `%i`, the instance a template is started for.
INSTANCE = "%i"


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
        for tag in TAG_RE.findall(info):
            blocks.setdefault(tag, []).append("\n".join(body) + "\n")
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
    for config in CONFIGS:
        if config not in blocks:
            raise GuideError(f"{guide}: no code block is tagged `file={config}`")
    written = []
    for path, parts in blocks.items():
        target = out.joinpath(*safe_path(path).parts)
        target.parent.mkdir(parents=True, exist_ok=True)
        text = "\n".join(parts)
        target.write_text(text, encoding="utf-8")
        if text.startswith("#!"):
            target.chmod(0o755)
        written.append(target)
    return written


def units(out: Path) -> dict[str, configparser.ConfigParser]:
    """Each systemd unit under `out/systemd/` parsed, by file name."""
    parsed = {}
    for path in sorted((out / "systemd").glob("*")):
        ini = configparser.ConfigParser(strict=False, interpolation=None)
        ini.optionxform = str  # type: ignore[assignment,method-assign]
        try:
            ini.read_string(path.read_text(encoding="utf-8"))
        except configparser.Error as e:
            raise GuideError(f"{path.name}: {e}") from e
        parsed[path.name] = ini
    return parsed


def check_units(out: Path) -> list[str]:
    """Every way the guide's units are not wired to its other files: a service
    runs a script the guide tags, a timer starts an instance of a service
    template, and that template's environment file for the instance is tagged
    too."""
    problems = []
    try:
        parsed = units(out)
    except GuideError as e:
        return [str(e)]
    for name, ini in parsed.items():
        if name.endswith(".service"):
            exec_start = ini.get("Service", "ExecStart", fallback="").split()
            if not exec_start or not (out / PurePosixPath(exec_start[0]).name).is_file():
                problems.append(f"{name}: ExecStart={' '.join(exec_start)} is no script the guide tags")
        elif name.endswith(".timer"):
            start = ini.get("Timer", "Unit", fallback="")
            instance = re.fullmatch(r"(.+)@(.+)\.service", start)
            if not ini.has_option("Timer", "OnCalendar"):
                problems.append(f"{name}: no OnCalendar")
            if not instance:
                problems.append(f"{name}: Unit={start} is no instance of a service template")
                continue
            template = f"{instance.group(1)}@.service"
            if template not in parsed:
                problems.append(f"{name}: starts {start}, and the guide tags no {template}")
                continue
            env = parsed[template].get("Service", "EnvironmentFile", fallback="")
            if not (out / env.replace(INSTANCE, instance.group(2)).lstrip("/")).is_file():
                problems.append(f"{name}: {template}'s EnvironmentFile={env} is no file the guide tags for {start}")
    return problems


def check_scripts(written: list[Path]) -> list[str]:
    """Every script of the guide that `bash -n` refuses."""
    problems = []
    for path in written:
        if path.suffix == ".sh":
            done = subprocess.run(["bash", "-n", str(path)], capture_output=True, text=True)
            if done.returncode != 0:
                problems.append(f"{path.name}: {done.stderr.strip()}")
    return problems


def check_workflow(out: Path) -> list[str]:
    """Every way the guide's CI workflow is not one this repository can run: it is
    tagged, it uses no action (the standing rule, `workflows.py`'s) and its steps
    read as `workflows.py` reads them."""
    path = out / WORKFLOW
    if not path.is_file():
        return [f"the guide tags no {WORKFLOW}"]
    text = path.read_text(encoding="utf-8")
    problems = [f"{WORKFLOW}:{n}: `uses:`, and no workflow uses an action" for n in workflows.uses(text)]
    try:
        if not workflows.steps(text, WORKFLOW):
            problems.append(f"{WORKFLOW}: no steps")
    except workflows.WorkflowError as e:
        problems.append(str(e))
    return problems


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
    problems = check_scripts(written) + check_units(out) + check_workflow(out)
    for problem in problems:
        print(f"sim: {problem}", file=sys.stderr)
    if problems:
        return 1
    if not load:
        return 0
    for config in CONFIGS:
        done = subprocess.run(load_command(out / config), cwd=REPO)
        if done.returncode != 0:
            print(f"sim: {out / config} does not load", file=sys.stderr)
            return 1
        print(f"sim: {out / config} loads", file=stdout)
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
