#!/usr/bin/env python3
"""The end-to-end Antora site: the whole site and one component alone, built
(D24, `fixtures/antora/`).

`fixtures/antora/content/<component>/` holds three tiny components written by
hand: `guide` (version `1.0`), `tools` (versionless, `~`) and `ops` (version
`master`, two modules and a page-alias), xref-ing one another with fragments,
an anchor the others link to and an image another embeds. Antora reads a git
repository, so each is copied under `runs/e2e/repos/` and committed there with
a hosted `origin`, and a later step edits it to plant a change. The Fedora UI
bundle is the one `fixtures.py antora-ui` vendored, checked against its
recorded sha256 before every build; Antora is the exact version below,
installed from npm under `runs/e2e/antora/`.

Builds, all with `CI=true` as Forgejo Actions sets it:

- `runs/e2e/whole/`: all three components, `site.url`
  `http://127.0.0.1:<--port>/en-US` as production's is `/en-US`, so it has a
  sitemap, canonical links and the page-alias's redirect page, and its
  footer and language links are production's, rooted at `/en-US/`. Its pages
  carry the edit link Antora makes from the hosted `origin`, as the pages of
  docs.fedoraproject.org do, so `antora`'s Resolve can find a page by its
  source path. A static server serves this directory under `/en-US/`.
- `runs/e2e/partial/`: `tools` alone, with no `site.url` and no `origin` as a
  content repo's CI build has neither (`docs/wip/roadmap-P5-ci.md`, "Facts
  found while grilling"): no page carries an edit link, and its xrefs into
  `guide` and `ops` render as `#<refspec>`.
- `runs/e2e/partial-image/`: `partial` less the image `guide` embeds.
- `runs/e2e/partial-changed/`: `partial` with `plant`'s changes.

Then the gate (D24): `bruce crawl` of the served site, `export-baseline`, and
`check-dir` of each partial build against that baseline, the exit status and
every section of the JSON asserted by `verify`. The unchanged build must be
silent (exit 0), the build with the image gone fails on the inverse check
alone (and passes under `--inverse warn`), and the planted build fails on
exactly the rows `PLANTED_*` name. Nothing leaves the machine: the
configuration allowlists no host. Needs `cargo`, node and, on the first run,
the network.

`--save-pages` also copies a few of the builds' pages into
`fixtures/antora/pages/`, recorded in a manifest, so tests read what a build
writes rather than what one is remembered to write. The run is slow beside the
rest of the checks, so it is not in `mise run check`.

Usage:

    mise run e2e
    cd scripts && uv run e2e.py [--port N] [--save-pages]
    cd scripts && uv run python -m unittest test_e2e
"""

from __future__ import annotations

import argparse
import functools
import hashlib
import http.server
import json
import os
import shutil
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any, Sequence
from urllib.parse import urlsplit

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
#: A hosted-git remote Antora derives an edit link from, the whole build's pages
#: carrying one as production's do and a content repo's CI build not.
ORIGIN = "https://github.com/bruce-fixture/{}.git"
GIT_ENV = {
    "GIT_AUTHOR_NAME": "bruce", "GIT_AUTHOR_EMAIL": "bruce@example.org",
    "GIT_COMMITTER_NAME": "bruce", "GIT_COMMITTER_EMAIL": "bruce@example.org",
}


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
    `repos`, replacing what was there, with the hosted `origin` Antora makes an
    edit link from."""
    if repos.exists():
        shutil.rmtree(repos)
    made = {}
    for name in COMPONENTS:
        dest = repos / name
        shutil.copytree(content / name, dest)
        run(["git", "init", "-q", "-b", "main"], dest, GIT_ENV)
        run(["git", "remote", "add", "origin", ORIGIN.format(name)], dest, GIT_ENV)
        run(["git", "add", "-A"], dest, GIT_ENV)
        run(["git", "commit", "-q", "-m", name], dest, GIT_ENV)
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


class Site(http.server.SimpleHTTPRequestHandler):
    """Serves a directory under `SITE_PATH` and nothing elsewhere, as the
    production host serves the whole build under `/en-US/`."""

    def translate_path(self, path: str) -> str:
        path = urlsplit(path).path
        if path != SITE_PATH and not path.startswith(SITE_PATH + "/"):
            return os.path.join(str(self.directory), "not-served")
        return super().translate_path(path[len(SITE_PATH):] or "/")

    def log_message(self, format: str, *args: Any) -> None:
        pass


def serve(site: Path, port: int) -> http.server.ThreadingHTTPServer:
    """`site` served under `SITE_PATH` on a background thread, until `shutdown()` and `server_close()`."""
    try:
        server = http.server.ThreadingHTTPServer(
            ("127.0.0.1", port), functools.partial(Site, directory=str(site))
        )
    except OSError as err:
        raise E2eError(f"cannot serve the whole site on port {port}: {err}") from err
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


def bruce_config(port: int) -> str:
    """The deployment's configuration for both the crawl and `check-dir`: the
    fixture host is fetched fast, and no external host is on the allowlist, so
    nothing leaves the machine."""
    return (
        "[rate]\nper_second = 500\nconcurrency = 8\n\n"
        '[host."127.0.0.1"]\nper_second = 500\n\n'
        f'[[root]]\nurl = "http://127.0.0.1:{port}{SITE_PATH}/"\n'
    )


def bruce_bin() -> Path:
    """The debug `bruce`, built if out of date."""
    run(["cargo", "build", "--quiet"], REPO)
    return REPO / "target" / "debug" / "bruce"


def bruce(bin: Path, *args: str | Path) -> tuple[int, str, str]:
    """One `bruce` command: its exit status, stdout and stderr."""
    proc = subprocess.run([str(bin), *map(str, args)], cwd=REPO, capture_output=True, text=True)
    return proc.returncode, proc.stdout, proc.stderr


def replace(path: Path, old: str, new: str) -> None:
    """`old` in `path` replaced by `new`; it must be there, or the fixture and
    the plant have drifted apart."""
    text = path.read_text()
    if old not in text:
        raise E2eError(f"{path}: no {old!r} to plant over")
    path.write_text(text.replace(old, new, 1))


def plant(tools: Path) -> None:
    """The changes a PR to `tools` makes, in its working tree: a deleted page
    (also linked by `link:`), a removed anchor, a deleted image, and new broken
    links, xrefs and images, two valid xrefs out of the build, one of them
    through a page-alias (D57)."""
    pages = tools / "modules" / "ROOT" / "pages"
    (pages / "reference.adoc").unlink()
    (tools / "modules" / "ROOT" / "images" / "diagram.svg").unlink()
    replace(tools / "modules" / "ROOT" / "nav.adoc", "** xref:reference.adoc[Reference]\n", "")
    replace(pages / "index.adoc", "xref:reference.adoc[reference]", "link:reference/[reference]")
    replace(pages / "usage.adoc", "[#running]", "[#run]")
    with (pages / "index.adoc").open("a") as f:
        f.write("\nHow to run a tool is in xref:usage.adoc#running[usage].\n")
    with (pages / "usage.adoc").open("a") as f:
        f.write(
            "\nA link to nowhere: link:../nowhere/[nowhere].\n"
            "\nAn xref out of the build to a page there is not: xref:guide::nonexistent.adoc[no page].\n"
            "\nOne to a page there is: xref:guide::upgrade.adoc#rollback[rolling back].\n"
            "\nOne through a page-alias: xref:ops::disks.adoc[the disks].\n"
            "\nAn image that is not there:\n\nimage::missing.png[Missing]\n"
        )


def check_dir(
    bin: Path, build: Path, port: int, config: Path, baseline: Path, db: Path, *extra: str,
) -> tuple[int, Any, str]:
    """`bruce check-dir` on `build`, as a content repo's CI runs it: the exit
    status, its JSON object and stderr."""
    code, out, err = bruce(
        bin, "check-dir", build, "--prefix", f"http://127.0.0.1:{port}{SITE_PATH}/",
        "--baseline", baseline, "--config", config, "--db", db, "--format", "json", *extra,
    )
    return code, (json.loads(out) if out.strip() else None), err


def rows(doc: dict[str, Any], section: str, prefix: str, *keys: str) -> set[tuple[Any, ...]]:
    """`section`'s rows as tuples of `keys`, a URL cut to its path after `prefix`."""
    return {
        tuple(v[len(prefix):] if isinstance(v, str) and v.startswith(prefix) else v for v in (r[k] for k in keys))
        for r in doc[section]
    }


#: An unchanged build of `tools` has nothing introduced, though it lists the
#: links that redirect: each page's home link reaches the site's `index.html`,
#: which refreshes to the start page of the build (`tools/`) where the baseline's
#: refreshes to `guide/`. They never fail the check.
UNCHANGED_REDIRECTED = {
    ("tools/", "..", "refresh"), ("tools/reference/", "../..", "refresh"), ("tools/usage/", "../..", "refresh"),
}
#: The xref through the page-alias is among them, its page a meta refresh (D57).
PLANTED_REDIRECTED = {
    ("tools/", "..", "refresh"), ("tools/usage/", "../..", "refresh"), ("tools/usage/", "#ops::disks.adoc", "refresh"),
}
#: What `plant` introduces, each rule once: (page, raw_href, kind, outcome,
#: reason, baseline). The valid xrefs out of the build, the alias's too, are absent, and so is the
#: deleted image no built page names, which only the inverse check finds.
PLANTED_INTRODUCED = {
    # a deleted page, linked by `link:`
    ("tools/", "reference/", "anchor", "broken_url", "not_built", "ok"),
    # the same page, linked by an xref that no longer resolves in a held component
    ("tools/usage/", "#reference.adoc#options", "anchor", "broken_url", "unresolved_xref", "new"),
    # an xref out of the build that the baseline cannot resolve
    ("tools/usage/", "#guide::nonexistent.adoc", "anchor", "broken_url", "unresolved_xref", "new"),
    # a new broken link and a new broken image
    ("tools/usage/", "../nowhere/", "anchor", "broken_url", "not_built", "new"),
    ("tools/usage/", "missing.png", "image", "broken_url", "not_built", "new"),
    # an anchor others link to, removed, and linked from the build itself
    ("tools/", "usage/#running", "anchor", "broken_anchor", None, "new"),
}
#: What others' links to `tools` now lack: (page, raw_href, kind, removed, component).
PLANTED_INVERSE = {
    ("ops/", "../tools/reference/", "anchor", "page", "ops"),
    ("ops/admin/partitions/", "../../../tools/usage/#running", "anchor", "anchor", "ops"),
    ("guide/1.0/upgrade/", "../../../tools/_images/diagram.svg", "image", "file", "guide"),
}


def redirected(doc: dict[str, Any], prefix: str) -> set[tuple[Any, ...]]:
    """The redirected links as (page, raw_href, the first hop's kind)."""
    return {
        (r["page"][len(prefix):], r["raw_href"], r["chain"].split()[0]) for r in doc["redirected"]
    }


#: Deleting the image alone, which no built page names, is inverse breakage and nothing else.
IMAGE_ONLY_INVERSE = {("guide/1.0/upgrade/", "../../../tools/_images/diagram.svg", "image", "file", "guide")}


def verify(
    unchanged: tuple[int, Any, str], changed: tuple[int, Any, str], port: int,
    image_only: tuple[tuple[int, Any, str], tuple[int, Any, str]] | None = None,
    reports: tuple[list[Any], list[Any]] | None = None,
) -> list[str]:
    """Every way the two `check-dir` runs differ from what the fixture and the
    planted changes promise, empty when there is none."""
    prefix = f"http://127.0.0.1:{port}{SITE_PATH}/"
    problems: list[str] = []

    def expect(what: str, got: Any, want: Any) -> None:
        if got != want:
            problems.append(f"{what}: got {got!r}, want {want!r}")

    for name, (code, doc, err), want_code in (("unchanged", unchanged, 0), ("planted", changed, 1)):
        expect(f"{name} exit status", code, want_code)
        if doc is None:
            problems.append(f"{name}: no JSON on stdout; stderr: {err.strip()}")
            continue
        expect(f"{name} errors", doc["errors"], [])
        expect(f"{name} baseline run", doc["baseline_run"], 1)
    if unchanged[1]:
        doc = unchanged[1]
        expect("unchanged introduced", doc["introduced"], [])
        expect("unchanged inverse", doc["inverse"], [])
        expect("unchanged redirected", redirected(doc, prefix), UNCHANGED_REDIRECTED)
        # The footer's three root-relative links, 404 in the baseline, on each of three pages.
        expect("unchanged already broken", doc["already_broken"]["count"], 9)
    if changed[1]:
        doc = changed[1]
        keys = ("page", "raw_href", "kind", "outcome", "reason", "baseline")
        expect("planted introduced", rows(doc, "introduced", prefix, *keys), PLANTED_INTRODUCED)
        expect("planted introduced count", len(doc["introduced"]), len(PLANTED_INTRODUCED))
        expect(
            "planted inverse",
            rows(doc, "inverse", prefix, "page", "raw_href", "kind", "removed", "component"),
            PLANTED_INVERSE,
        )
        expect("planted inverse count", len(doc["inverse"]), len(PLANTED_INVERSE))
        # Two built pages now, and the deleted page's three footer links are gone.
        expect("planted already broken", doc["already_broken"]["count"], 6)
        expect("planted redirected", redirected(doc, prefix), PLANTED_REDIRECTED)
        if reports:
            expect("report introduced agrees with the gate", reports[0], doc["introduced"])
            expect("report inverse agrees with the gate", reports[1], doc["inverse"])
    for name, (code, doc, err), want_code in zip(("fail", "warn"), image_only or (), (1, 0)):
        expect(f"image only, --inverse={name}, exit status", code, want_code)
        if doc is None:
            problems.append(f"image only, --inverse={name}: no JSON on stdout; stderr: {err.strip()}")
            continue
        expect(f"image only, --inverse={name}, introduced", doc["introduced"], [])
        expect(
            f"image only, --inverse={name}, inverse",
            rows(doc, "inverse", prefix, "page", "raw_href", "kind", "removed", "component"),
            IMAGE_ONLY_INVERSE,
        )
    return problems


def main(argv: Sequence[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--port", type=int, default=PORT, help="the whole site's port (default: %(default)s)")
    ap.add_argument("--save-pages", action="store_true", help="copy a few built pages into fixtures/antora/pages/")
    args = ap.parse_args(argv)
    server = None
    try:
        ui = check_ui()
        antora = install_antora()
        repos = materialise(FIXTURES / "content", OUT / "repos")
        whole = build(
            antora, "whole", [repos[c] for c in COMPONENTS], "guide::index.adoc", ui, OUT,
            f"http://127.0.0.1:{args.port}{SITE_PATH}",
        )
        run(["git", "remote", "remove", "origin"], repos[PARTIAL], GIT_ENV)
        partial = build(antora, "partial", [repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT)
        if args.save_pages:
            save_pages({"whole": whole, "partial": partial}, args.port, ui)
        print(f"whole: {sum(1 for _ in whole.rglob('*.html'))} html files -> {whole}")
        print(f"partial: {sum(1 for _ in partial.rglob('*.html'))} html files -> {partial}")

        bin = bruce_bin()
        config = OUT / "bruce.toml"
        config.write_text(bruce_config(args.port))
        for f in OUT.glob("*.duckdb*"):
            f.unlink()
        server = serve(whole, args.port)
        crawl_db, baseline, ci_db = OUT / "crawl.duckdb", OUT / "baseline.duckdb", OUT / "ci.duckdb"
        for step in (
            ("crawl", "--config", config, "--db", crawl_db),
            ("export-baseline", "--db", crawl_db, "--out", baseline),
        ):
            code, out, err = bruce(bin, *step)
            if code:
                raise E2eError(f"bruce {step[0]} exited {code}:\n{out}{err}")
            print(out + err, end="")
        ci = (args.port, config, baseline, ci_db)
        unchanged = check_dir(bin, partial, *ci)
        (repos[PARTIAL] / "modules" / "ROOT" / "images" / "diagram.svg").unlink()
        image_build = build(antora, "partial-image", [repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT)
        image_only = (check_dir(bin, image_build, *ci), check_dir(bin, image_build, *ci, "--inverse", "warn"))
        run(["git", "checkout", "--", "."], repos[PARTIAL])
        plant(repos[PARTIAL])
        changed_build = build(antora, "partial-changed", [repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT)
        changed = check_dir(bin, changed_build, *ci)
        reports = []
        for kind in ("introduced", "inverse"):
            code, out, err = bruce(
                bin, "report", kind, "--db", ci_db, "--run", str(changed[1]["run"]), "--format", "json"
            )
            reports.append(json.loads(out) if code == 0 and out.strip() else f"exit {code}: {err.strip()}")
        problems = verify(unchanged, changed, args.port, image_only, (reports[0], reports[1]))
        if problems:
            raise E2eError("the gate is not what the planted changes promise:\n  " + "\n  ".join(problems))
        print(f"gate: unchanged build exit {unchanged[0]}, planted build exit {changed[0]}, as planted")
    except E2eError as err:
        print(f"e2e: {err}", file=sys.stderr)
        return 1
    finally:
        if server:
            server.shutdown()
            server.server_close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
