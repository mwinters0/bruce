#!/usr/bin/env python3
"""The en-US crawl a review reads: launched, its digest written, and served.

A digest's two-run sections need a database holding two crawls, so `crawl`
copies the newest finished en-US crawl's database (`runs/*/en-us.duckdb`
whose directory's `exit` reads 0) into a new `runs/en-us-<YYYYMMDD-HHMM>/`
and crawls into the copy: the new run is the latest, and the digest's default
`--from` (the finished crawl run before it) is the copied one. Each crawl so
copies the last, and a database carries every run before it.

- `crawl` makes a release build, regenerates `sim/` (`sim.py`) and launches
  `bruce crawl --config bruce.toml` from there in its own session, so it
  outlives the terminal, and returns (`CLAUDE.md`, "Long-running processes").
  The directory holds `en-us.duckdb`, `baseline` (the database copied),
  `pgid`, `started`, `crawl.log`, `time.log` (`/usr/bin/time -v`), `exit`
  and `ended`, the last two written when the crawl ends. `crawl.log` is empty
  until then, the run line printing on exit. Stop it with `pkill -g <pgid>`;
  the run so far stays in the database, marked unfinished.
- `digest [DIR]` writes `DIR/digest/en-us-<date>.{html,json}` with a release
  build, once the crawl in DIR (the newest `runs/en-us-*`) has finished, and
  otherwise says whether it is running or how it ended.
- `serve [DIR]` serves `DIR/digest` on every interface, port 8086, until
  interrupted.

Usage:

    mise run crawl
    mise run digest
    mise run serve
    cd scripts && uv run review.py crawl [--baseline DB]
    cd scripts && uv run review.py digest [DIR]
    cd scripts && uv run review.py serve [DIR] [--port N]
    cd scripts && uv run python -m unittest test_review
"""

from __future__ import annotations

import argparse
import datetime
import os
import shlex
import shutil
import socket
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence

import sim

REPO = Path(__file__).resolve().parent.parent
RUNS = Path("runs")
BRUCE = Path("target/release/bruce")
TIME = Path("/usr/bin/time")
DB = "en-us.duckdb"
NAME = "en-us"
#: A review crawl's directory under `runs/` is this and its UTC minute.
PREFIX = "en-us-"
PORT = 8086


class ReviewError(Exception):
    """The step cannot go on; the message says why."""


@dataclass(frozen=True)
class State:
    """Where a crawl directory's crawl stands: `finished` (exit 0),
    `failed` (another exit), `running`, `vanished` (no exit, no process) or
    `unlaunched` (no `started`)."""

    name: str
    exit: int | None = None


def alive(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def state(run: Path) -> State:
    exit_file = run / "exit"
    if exit_file.exists():
        try:
            code = int(exit_file.read_text(encoding="utf-8").strip())
        except ValueError:
            return State("failed")
        return State("finished" if code == 0 else "failed", code)
    if not (run / "started").exists():
        return State("unlaunched")
    try:
        pgid = int((run / "pgid").read_text(encoding="utf-8").strip())
    except (OSError, ValueError):
        return State("vanished")
    return State("running" if alive(pgid) else "vanished")


def baseline(runs: Path) -> Path | None:
    """The newest finished crawl's database under `runs`, by its `ended`."""
    found = []
    for db in runs.glob(f"*/{DB}"):
        run = db.parent
        if state(run).name != "finished":
            continue
        try:
            ended = (run / "ended").read_text(encoding="utf-8").strip()
        except OSError:
            continue
        found.append((ended, db))
    return max(found)[1] if found else None


def review_dirs(runs: Path) -> list[Path]:
    """The review crawls' directories, oldest first (their names sort by
    time)."""
    return sorted(p for p in runs.glob(f"{PREFIX}*") if p.is_dir())


def newest(runs: Path) -> Path:
    dirs = review_dirs(runs)
    if not dirs:
        raise ReviewError(f"no {runs}/{PREFIX}* directory: run `mise run crawl` first")
    return dirs[-1]


def crawl_script(run: Path, bruce: Path) -> str:
    """The shell line the detached session runs, from `sim/`."""
    def q(p: Path) -> str:
        return shlex.quote(str(p))

    return (
        f"date -Is > {q(run / 'started')}; "
        f"{q(TIME)} -v -o {q(run / 'time.log')} {q(bruce)} crawl --config bruce.toml "
        f"--db {q(run / DB)} > {q(run / 'crawl.log')} 2>&1; "
        f"echo $? > {q(run / 'exit')}; "
        f"date -Is > {q(run / 'ended')}"
    )


def cargo_release() -> None:
    done = subprocess.run(["cargo", "build", "--release"], cwd=REPO)
    if done.returncode != 0:
        raise ReviewError("the release build failed")


def opens(bruce: Path, db: Path) -> None:
    """Refuses a database this build cannot open (a schema it does not
    carry), before an hour's crawl finds out."""
    done = subprocess.run(
        [str(bruce), "report", "stats", "--db", str(db)],
        capture_output=True,
        text=True,
    )
    if done.returncode != 0:
        raise ReviewError(f"{db} does not open: {done.stderr.strip()}")


def crawl(runs: Path, chosen: Path | None, now: datetime.datetime) -> Path:
    for run in review_dirs(runs):
        if state(run).name == "running":
            pgid = (run / "pgid").read_text(encoding="utf-8").strip()
            raise ReviewError(f"{run} is still crawling (stop it with `pkill -g {pgid}`)")
    source = chosen or baseline(runs)
    if source is None:
        raise ReviewError(f"no finished crawl under {runs} to copy: pass --baseline DB")
    if not source.is_file():
        raise ReviewError(f"{source} is not a file")
    if not TIME.exists():
        raise ReviewError(f"{TIME} is not installed")
    cargo_release()
    if sim.run(REPO / sim.GUIDE, REPO / sim.OUT, load=True) != 0:
        raise ReviewError("sim/ could not be regenerated")
    run = runs / f"{PREFIX}{now:%Y%m%d-%H%M}"
    if run.exists():
        raise ReviewError(f"{run} exists already")
    run.mkdir(parents=True)
    run = run.resolve()
    shutil.copyfile(source, run / DB)
    (run / "baseline").write_text(f"{source.resolve()}\n", encoding="utf-8")
    bruce = (REPO / BRUCE).resolve()
    try:
        opens(bruce, run / DB)
    except ReviewError:
        shutil.rmtree(run)
        raise
    proc = subprocess.Popen(
        ["sh", "-c", crawl_script(run, bruce)],
        cwd=REPO / sim.OUT,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    (run / "pgid").write_text(f"{proc.pid}\n", encoding="utf-8")
    return run


def digest(run: Path) -> list[Path]:
    now = state(run)
    if now.name != "finished":
        detail = f", exit {now.exit}" if now.exit is not None else ""
        raise ReviewError(f"{run}: the crawl is {now.name}{detail}, not finished (crawl.log, time.log)")
    cargo_release()
    out = run / "digest"
    done = subprocess.run(
        [str(REPO / BRUCE), "report", "digest", "--db", str(run / DB), "--out", str(out), "--name", NAME]
    )
    if done.returncode != 0:
        raise ReviewError(f"{run}: the digest failed")
    return sorted(out.glob("*.html"))


def serve(run: Path, port: int) -> None:
    out = run / "digest"
    pages = sorted(out.glob("*.html"))
    if not pages:
        raise ReviewError(f"{out} holds no digest: run `mise run digest` first")
    with socket.socket() as probe:
        try:
            probe.bind(("0.0.0.0", port))
        except OSError as e:
            raise ReviewError(f"port {port} is taken ({e.strerror}): stop that server or pass --port") from e
    for page in pages:
        print(f"review: http://localhost:{port}/{page.name}")
    sys.stdout.flush()
    os.execv(
        sys.executable,
        [sys.executable, "-m", "http.server", str(port), "--bind", "0.0.0.0", "--directory", str(out)],
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    steps = parser.add_subparsers(dest="step", required=True)
    c = steps.add_parser("crawl", help="launch a crawl into a copy of the last")
    c.add_argument("--baseline", type=Path, help="the database to copy [default: the newest finished crawl's]")
    d = steps.add_parser("digest", help="write a finished crawl's digest")
    d.add_argument("dir", nargs="?", type=Path, help=f"the crawl's directory, relative to the repository [default: the newest {PREFIX}*]")
    s = steps.add_parser("serve", help="serve a crawl's digest")
    s.add_argument("dir", nargs="?", type=Path, help=f"the crawl's directory, relative to the repository [default: the newest {PREFIX}*]")
    s.add_argument("--port", type=int, default=PORT)
    args = parser.parse_args(argv)
    runs = REPO / RUNS
    # `mise run` starts in `scripts/`, so a relative path is the repository's.
    chosen = REPO / args.dir if getattr(args, "dir", None) else None
    try:
        if args.step == "crawl":
            now = datetime.datetime.now(datetime.timezone.utc)
            run = crawl(runs, args.baseline, now)
            pgid = (run / "pgid").read_text(encoding="utf-8").strip()
            print(f"review: crawling into {run / DB}, a copy of {(run / 'baseline').read_text().strip()}")
            print(f"review: process group {pgid}; stop it with `pkill -g {pgid}`")
            print("review: about 70 minutes; then `mise run digest` and `mise run serve`")
        elif args.step == "digest":
            for page in digest(chosen or newest(runs)):
                print(f"review: wrote {page}")
        else:
            serve(chosen or newest(runs), args.port)
    except ReviewError as e:
        print(f"review: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
