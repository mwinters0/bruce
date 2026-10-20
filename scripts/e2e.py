#!/usr/bin/env python3
"""The end-to-end Antora site: the whole site and one component alone, built
(D24, `fixtures/antora/`).

`fixtures/antora/content/<component>/` holds three tiny components written by
hand: `guide` (version `1.0`), `tools` (versionless, `~`) and `ops` (version
`master`, two modules and a page-alias), xref-ing one another with fragments
and an anchor the others link to. Antora reads a git repository, so each is
copied under `runs/e2e/repos/` and committed there, where a later step edits
it to plant a change. The Fedora UI bundle is the one `fixtures.py antora-ui`
vendored, checked against its recorded sha256 before every build; Antora is
the exact version below, installed from npm under `runs/e2e/antora/`.

Two builds, both with `CI=true` as Forgejo Actions sets it, which is why no
page carries an edit link:

- `runs/e2e/whole/`: all three components, `site.url`
  `http://127.0.0.1:<--port>/en-US` as production's is `/en-US`, so it has a
  sitemap, canonical links and the page-alias's redirect page, and its
  footer and language links are production's, rooted at `/en-US/`. A static
  server serving this directory under `/en-US/` serves production's URLs.
- `runs/e2e/partial/`: `tools` alone, with no `site.url` as a content repo's
  `site.yml` has none (`docs/wip/roadmap-P5-ci.md`, "Facts found while
  grilling"): its xrefs into `guide` and `ops` render as `#<refspec>`.

`--save-pages` also copies a few of those pages into
`fixtures/antora/pages/`, recorded in a manifest, so tests read what a build
writes rather than what one is remembered to write. It needs node and
network on the first run, and is slow beside the rest of the checks, so it is
not in `mise run check`.

Usage:

    mise run e2e
    cd scripts && uv run e2e.py [--port N] [--save-pages]
    cd scripts && uv run python -m unittest test_e2e
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Sequence

REPO = Path(__file__).resolve().parent.parent
FIXTURES = REPO / "fixtures" / "antora"
OUT = REPO / "runs" / "e2e"
#: The Antora production builds with (docs-fp-o's image carries 3.1.12; the
#: files the checker reads match between them).
ANTORA = "3.1.15"
#: Where the whole site is served: `http://127.0.0.1:PORT` plus `SITE_PATH`,
#: the path production's `site.url` carries, which the UI writes into its
#: root-relative footer links and a language link.
PORT = 8099
SITE_PATH = "/en-US"
COMPONENTS = ("guide", "tools", "ops")
#: The component the partial build holds.
PARTIAL = "tools"
#: Pages kept as fixtures, per build, as published paths under `public/`.
SAVED = {
    "partial": [
        "index.html",
        "tools/index.html",
        "tools/usage/index.html",
        "tools/reference/index.html",
    ],
    "whole": [
        "guide/1.0/install/index.html",
        "ops/disks/index.html",
        "ops/admin/partitions/index.html",
    ],
}
PAGES = FIXTURES / "pages"


class E2eError(Exception):
    """A step cannot run; the message says which and why."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_ui(fixtures: Path = FIXTURES) -> Path:
    """The vendored bundle, once it hashes to its record."""
    bundle, record = fixtures / "ui-bundle.zip", fixtures / "ui-bundle.json"
    if not bundle.exists() or not record.exists():
        raise E2eError("no UI bundle: run `cd scripts && uv run fixtures.py antora-ui`")
    want = json.loads(record.read_text())["sha256"]
    if sha256(bundle) != want:
        raise E2eError(f"{bundle} does not hash to {want}, ui-bundle.json's record")
    return bundle


def run(cmd: Sequence[str | Path], cwd: Path, env: dict[str, str] | None = None) -> str:
    """Run `cmd`; its output on failure goes into the error."""
    proc = subprocess.run(
        [str(c) for c in cmd],
        cwd=cwd,
        env={**os.environ, **(env or {})},
        capture_output=True,
        text=True,
    )
    if proc.returncode:
        raise E2eError(f"{' '.join(map(str, cmd))} exited {proc.returncode}:\n{proc.stdout}{proc.stderr}")
    return proc.stdout + proc.stderr


def install_antora(out: Path = OUT) -> Path:
    """The `antora` executable of the pinned version, installed on first use."""
    prefix = out / "antora"
    exe = prefix / "node_modules" / ".bin" / "antora"
    if exe.exists() and f"@antora/cli: {ANTORA}" in run([exe, "--version"], out.parent):
        return exe
    prefix.mkdir(parents=True, exist_ok=True)
    run(
        [
            "npm", "install", "--no-audit", "--no-fund", "--prefix", prefix,
            f"@antora/cli@{ANTORA}", f"@antora/site-generator@{ANTORA}",
        ],
        out.parent,
    )
    return exe


def materialise(content: Path, repos: Path) -> dict[str, Path]:
    """Each component under `content` as a committed git repository under
    `repos`, replacing what was there."""
    if repos.exists():
        shutil.rmtree(repos)
    made = {}
    for name in COMPONENTS:
        dest = repos / name
        shutil.copytree(content / name, dest)
        env = {
            "GIT_AUTHOR_NAME": "bruce", "GIT_AUTHOR_EMAIL": "bruce@example.org",
            "GIT_COMMITTER_NAME": "bruce", "GIT_COMMITTER_EMAIL": "bruce@example.org",
        }
        run(["git", "init", "-q", "-b", "main"], dest, env)
        run(["git", "add", "-A"], dest, env)
        run(["git", "commit", "-q", "-m", name], dest, env)
        made[name] = dest
    return made


def playbook(repos: Sequence[Path], start: str, ui: Path, output: Path, cache: Path) -> str:
    """An Antora playbook as YAML (JSON is YAML, so paths are quoted for free).

    `branches: HEAD` with the default worktree setting builds the repository's
    working tree, so an edit needs no commit. No `site.url`: the whole build
    gets it from `--url`."""
    doc = {
        "site": {"title": "Bruce fixture site", "start_page": start},
        "content": {"sources": [{"url": str(r), "branches": "HEAD"} for r in repos]},
        "ui": {"bundle": {"url": str(ui)}, "default_layout": "with_menu"},
        "urls": {"html_extension_style": "indexify"},
        "output": {"clean": True, "dir": str(output)},
        "runtime": {"fetch": False, "cache_dir": str(cache)},
    }
    return json.dumps(doc, indent=1) + "\n"


def build(
    antora: Path, name: str, repos: Sequence[Path], start: str, ui: Path, out: Path,
    url: str | None = None,
) -> Path:
    """Build one site into `out/<name>/`; returns that directory."""
    site = out / name
    pb = out / f"{name}.yml"
    pb.write_text(playbook(repos, start, ui, site, out / "cache"))
    cmd: list[str | Path] = [antora]
    if url:
        cmd += ["--url", url]
    log = run([*cmd, pb], out, {"CI": "true"})
    (out / f"{name}.log").write_text(log)
    return site


def save_pages(sites: dict[str, Path], port: int, ui: Path, pages: Path = PAGES) -> None:
    """Copy `SAVED`'s pages out of the builds, with a manifest."""
    if port != PORT:
        raise E2eError(f"saved pages are the port {PORT} build's; got --port {port}")
    if pages.exists():
        shutil.rmtree(pages)
    entries = []
    for which, paths in SAVED.items():
        for rel in paths:
            src = sites[which] / rel
            dest = pages / which / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dest)
            entries.append(
                {"build": which, "path": f"{which}/{rel}", "bytes": dest.stat().st_size, "sha256": sha256(dest)}
            )
    manifest = {"antora": ANTORA, "port": port, "ui_sha256": sha256(ui), "pages": entries}
    (pages / "manifest.json").write_text(json.dumps(manifest, indent=1) + "\n")


def main(argv: Sequence[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--port", type=int, default=PORT, help="the whole site's port (default: %(default)s)")
    ap.add_argument("--save-pages", action="store_true", help="copy a few built pages into fixtures/antora/pages/")
    args = ap.parse_args(argv)
    try:
        ui = check_ui()
        antora = install_antora()
        repos = materialise(FIXTURES / "content", OUT / "repos")
        whole = build(
            antora, "whole", [repos[c] for c in COMPONENTS], "guide::index.adoc", ui, OUT,
            f"http://127.0.0.1:{args.port}{SITE_PATH}",
        )
        partial = build(antora, "partial", [repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT)
        if args.save_pages:
            save_pages({"whole": whole, "partial": partial}, args.port, ui)
    except E2eError as err:
        print(f"e2e: {err}", file=sys.stderr)
        return 1
    print(f"whole: {sum(1 for _ in whole.rglob('*.html'))} html files -> {whole}")
    print(f"partial: {sum(1 for _ in partial.rglob('*.html'))} html files -> {partial}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
