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
  footer and language links are production's, rooted at `/en-US/`, its
  language menu naming a second locale, `fr`, which is never served, as
  production's build names every locale (`LANGUAGES`). Its pages
  carry the edit link Antora makes from the hosted `origin`, as the pages of
  docs.fedoraproject.org do, so `antora`'s Resolve can find a page by its
  source path. A static server serves this directory under `/en-US/`.
- `runs/e2e/partial/`: `tools` alone, with no `site.url` and no `origin` as a
  content repo's CI build has neither (D53; `docs/design/site-invariants.md`,
  "I15"): no page carries an edit link, and its xrefs into
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

The deployment guide's digest view (`fedora.html`) renders the crawl's digest too: the built-in
view's page with docs.fp.o's palette and nothing else changed (`check_guide_view`).

The deployment guide's scripts run over this site (`deployment.py`): its scheduled job
twice, with the real binary, then its pull script over the directory the second night
published, served locally. The guide's CI workflow runs over that directory too, less the steps
that install the tools and Antora and fetch the repo: its build step builds the `tools` repo
(`ci_run`), its fetch step takes the binary, config and baseline the second night published, and
its check step must pass the unchanged repo, fail the planted one and, given a directory whose
baseline fails its sum or one with no `SHA256SUMS`, fail the job as could not run (exit 2). The workflow also
parses with Antora's `js-yaml` as the steps `workflows.py` reads, and pins the Antora `ANTORA` names.

The crawl's digest is written too (`bruce report digest`), and its HTML, less
what varies from run to run (`normalise_digest`), is held to `fixtures/antora/
digest.html`, the golden file of the rendering over this site; `--save-digest`
writes it anew, to be read in a diff before it is committed.

The digest is written through two more views, the litmus tests of the views
(`fixtures/antora/views/`): `cs.html`, the whole digest in Czech, and
`links.csv`, one record to each finding. Each is held to a golden file beside
them (`views/golden/`), the Czech page less what varies like the English one,
and the CSV is parsed and counted against the JSON, which `--save-digest`
writes anew too.

`--save-pages` also copies a few of the builds' pages into
`fixtures/antora/pages/`, recorded in a manifest, so tests read what a build
writes rather than what one is remembered to write. The run is slow beside the
rest of the checks, so `mise run check` runs it only when the binary can have
changed (`check.py`, `E2E_READS`).

Usage:

    mise run e2e
    cd scripts && uv run e2e.py [--port N] [--save-pages]
    cd scripts && uv run python -m unittest test_e2e
"""

from __future__ import annotations

import argparse
import collections
import csv
import decimal
import difflib
import functools
import hashlib
import http.server
import io
import json
import os
import re
import shutil
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any, Sequence
from urllib.parse import urlsplit

import deployment
import workflows
from sim import GUIDE, VIEW as GUIDE_VIEW, tagged_blocks

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
#: The language menu production's build writes over the UI's
#: (`docs/design/site-invariants.md`, "I25"), for `en-US` and one other locale.
LANGUAGES = """<div class="page-languages">
<button class="languages-menu-toggle" title="Show other languages of the site">en-US</button>
<div class="languages-menu">
<a class="language" href="{{siteRootPath}}/../en-US{{page.url}}">en-US</a>
<a class="language" href="{{siteRootPath}}/../fr{{page.url}}">fr</a>
</div>
</div>
"""
PAGES = FIXTURES / "pages"
#: The digest's HTML over the whole site's crawl, as `normalise_digest` leaves it.
GOLDEN = FIXTURES / "digest.html"
#: The litmus views, each named for the file it writes, and the golden
#: files of what they write over the whole site's crawl.
VIEWS = FIXTURES / "views"
CS_VIEW, CSV_VIEW = VIEWS / "cs.html", VIEWS / "links.csv"
GOLDEN_CS, GOLDEN_CSV = VIEWS / "golden" / "cs.html", VIEWS / "golden" / "links.csv"
#: The columns of `links.csv`, one record to each finding in the JSON's `broken`
#: and `errors` that the severity map lists.
CSV_COLUMNS = [
    "page", "title", "source_url", "written_as", "resolves_to", "fragment", "kind", "level", "outcome", "reason",
    "status", "hint", "new",
]
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


def playbook(
    repos: Sequence[Path], start: str, ui: Path, output: Path, cache: Path, languages: bool = False,
) -> str:
    """An Antora playbook as YAML (JSON is YAML, so paths are quoted for free).

    `branches: HEAD` with the default worktree setting builds the repository's
    working tree, so an edit needs no commit. No `site.url`: the whole build
    gets it from `--url`. With `languages`, the UI's language menu is
    `LANGUAGES`, as production's is; a content repo's CI build keeps the UI's."""
    doc = {
        "site": {"title": "Bruce fixture site", "start_page": start},
        "content": {"sources": [{"url": str(r), "branches": "HEAD"} for r in repos]},
        "ui": {"bundle": {"url": str(ui)}, "default_layout": "with_menu"},
        "urls": {"html_extension_style": "indexify"},
        "output": {"clean": True, "dir": str(output)},
        "runtime": {"fetch": False, "cache_dir": str(cache)},
    }
    if languages:
        doc["ui"]["supplemental_files"] = [{"path": "partials/page-languages.hbs", "contents": LANGUAGES}]
    return json.dumps(doc, indent=1) + "\n"


def build(
    antora: Path, name: str, repos: Sequence[Path], start: str, ui: Path, out: Path,
    url: str | None = None, languages: bool = False,
) -> Path:
    """Build one site into `out/<name>/`; returns that directory."""
    site = out / name
    pb = out / f"{name}.yml"
    pb.write_text(playbook(repos, start, ui, site, out / "cache", languages))
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


def guide_profile() -> str:
    """The `[[profile]]` the deployment guide writes: the build's language menu
    names a second locale, so the deployment's profile is what learns it."""
    blocks = tagged_blocks((REPO / GUIDE).read_text(), str(GUIDE)).get("bruce.toml", [])
    profiles = [block for block in blocks if "[[profile]]" in block]
    if len(profiles) != 1:
        raise E2eError(f"{GUIDE} holds {len(profiles)} blocks of bruce.toml with a profile, not one")
    return profiles[0]


def bruce_config(port: int) -> str:
    """The deployment's configuration for both the crawl and `check-dir`: the
    fixture host is fetched fast, and no external host is on the allowlist, so
    nothing leaves the machine. The guide's profile is the deployment's own."""
    return (
        "[default]\nper_second = 500\nconcurrency = 8\n\n"
        '[host."127.0.0.1"]\nper_second = 500\n\n'
        f'[[root]]\nurl = "http://127.0.0.1:{port}{SITE_PATH}/"\n\n' + guide_profile()
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
    links, xrefs and images, a mistyped scheme among them (D66), two valid xrefs
    out of the build, one of them through a page-alias (D57)."""
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
            "\nA mistyped scheme: link:hhttps://example.org/tools/[the tools site].\n"
        )


#: What a digest holds that differs between two crawls of one site: the run's
#: times and duration, the date in its title and heading, and the tool version.
VOLATILE = (
    (re.compile(r'<time datetime="[^"]*">[^<]*</time>'), '<time datetime="T">T</time>'),
    (re.compile(r'(<span class="(?:duration|version|date)">)[^<]*'), r"\1V"),
    (re.compile(r"(<title>Link check digest )\d{4}-\d{2}-\d{2}"), r"\1DATE"),
    (re.compile(r"(<title>Přehled kontroly odkazů )\d+\. \d+\. \d{4}"), r"\1DATE"),
)


def normalise_digest(html: str) -> str:
    """`html` with `VOLATILE` replaced by fixed text, so two crawls of one site
    give one document."""
    for pattern, fixed in VOLATILE:
        html = pattern.sub(fixed, html)
    return html


def digest(bin: Path, db: Path, out: Path) -> dict[str, str]:
    """`bruce report digest` of the crawl in `db` through the built-in view and the
    litmus views: what it wrote, by the part of the file's name after the date
    (`json`, `html`, `cs.html`, `links.csv`)."""
    if out.exists():
        shutil.rmtree(out)
    code, stdout, err = bruce(
        bin, "report", "digest", "--db", db, "--out", out, "--name", "e2e",
        "--view", "default", "--view", CS_VIEW, "--view", CSV_VIEW,
    )
    if code:
        raise E2eError(f"bruce report digest exited {code}:\n{stdout}{err}")
    written = {re.sub(r"^e2e-\d{4}-\d{2}-\d{2}[-.]", "", f.name): f.read_text() for f in out.iterdir()}
    if sorted(written) != ["cs.html", "html", "json", "links.csv"]:
        raise E2eError(f"bruce report digest wrote {sorted(f.name for f in out.iterdir())}, not the four files asked")
    return written


def check_workflow(guide: Path, modules: Path) -> list[str]:
    """Every way the guide's CI workflow is not what e2e runs: a YAML parser reads
    other steps than `workflows.py` does, or it pins another Antora than `ANTORA`."""
    text = deployment.workflow_text(guide)
    problems = []
    try:
        parsed = deployment.yaml_steps(text, modules)
    except deployment.DeploymentError as err:
        return [str(err)]
    ours = workflows.steps(text, "the guide's CI workflow")
    if parsed != ours:
        problems.append(f"the guide's CI workflow reads as {[s.name for s in parsed]} by js-yaml and {[s.name for s in ours]} by workflows.py")
    pin = deployment.workflow_env(text).get("ANTORA_VERSION")
    if pin != ANTORA:
        problems.append(f"the guide's CI workflow pins Antora {pin}, e2e builds with {ANTORA}")
    return problems


def ci_run(
    guide: Path, port: int, antora_dir: Path, playbook_text: str, workspace: Path, url: str,
    only: Sequence[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """The guide's CI workflow over the `tools` repo as `playbook_text` builds it, fetching
    from `url`, in a fresh `workspace`: what a content repo's pull request runs. With
    `only`, just the steps it names."""
    shutil.rmtree(workspace, ignore_errors=True)
    workspace.mkdir(parents=True)
    (workspace / "ci.yml").write_text(playbook_text)
    env = {
        "PUB_URL": url,
        "PREFIX": f"http://127.0.0.1:{port}{SITE_PATH}/",
        "PLAYBOOK": "ci.yml",
        "SITE_DIR": "ci-site",
        "ANTORA_DIR": str(antora_dir),
    }
    return deployment.ci_job(guide, workspace, env, skip=deployment.INSTALL_STEPS, only=only)


def sums_baseline(pub: Path) -> str:
    """The name of the baseline `pub`'s `SHA256SUMS` lists, the one by content."""
    named = [n for n in deployment.sums(pub) if deployment.BY_CONTENT[2].fullmatch(n)]
    if len(named) != 1:
        raise E2eError(f"{pub}'s SHA256SUMS lists {len(named)} baselines, not one")
    return named[0]


def check_ci_jobs(unchanged: Any, planted: Any, broken: Any, missing: Any) -> list[str]:
    """Every way the CI workflow's four runs differ from what the guide promises, empty
    when there is none: a pass, a failed check, and two jobs that could not run."""
    problems: list[str] = []

    def expect(what: str, ok: bool, done: Any) -> None:
        if not ok:
            problems.append(f"CI workflow, {what}: exit {done.returncode}\n{done.stdout}{done.stderr}")

    expect("the unchanged build passes", unchanged.returncode == 0, unchanged)
    expect(
        "the planted build fails the check, the report in the log",
        planted.returncode == 1 and "nowhere/" in planted.stdout and "breaks the links listed above" in planted.stderr,
        planted,
    )
    expect(
        "a baseline failing its sum fails the job as could not run",
        broken.returncode == 2 and "fails its sum" in broken.stderr and "says nothing of the change" in broken.stderr,
        broken,
    )
    expect(
        "no published directory fails the job as could not run",
        missing.returncode == 2 and "no SHA256SUMS" in missing.stderr,
        missing,
    )
    return problems


def check_digest(js: str, html: str, port: int, golden: Path = GOLDEN, save: bool = False) -> list[str]:
    """Every way the digest of the whole site's crawl differs from what the site
    promises, empty when there is none. The golden file is written anew with `save`."""
    problems: list[str] = []
    doc = json.loads(js)
    totals = doc["totals"]
    if not (totals["pages"] and totals["links"]):
        problems.append(f"the digest's totals are empty: {totals}")
    if f"http://127.0.0.1:{port}{SITE_PATH}/" not in doc["run"]["roots"]:
        problems.append(f"the digest's roots are not the site's: {doc['run']['roots']}")
    for banned in ("<script", "<link", " src=", "@import"):
        if banned in html:
            problems.append(f"the digest's HTML holds {banned!r}")
    seen = normalise_digest(html)
    if save:
        golden.write_text(seen)
    elif port != PORT:
        print(f"digest: golden file not compared, it is the port {PORT} crawl's")
    elif not golden.exists():
        problems.append(f"no golden file {golden}: run with --save-digest")
    elif seen != golden.read_text():
        problems.append(f"the digest's HTML is not {golden}: read the diff, then run with --save-digest")
    return problems


#: A line of a digest's palette: a custom property or the colour scheme.
PALETTE_LINE = re.compile(r"^\s*(--[\w-]+:|color-scheme:)")
#: docs.fp.o's accent, which the guide's view sets, and the built-in view's own.
GUIDE_ACCENT, BUILT_IN_ACCENT = "#51a2da", "#a58de0"


def guide_view(bin: Path, db: Path, view: Path, out: Path) -> str:
    """The HTML of the crawl in `db` written through the guide's `view`."""
    if out.exists():
        shutil.rmtree(out)
    code, stdout, err = bruce(bin, "report", "digest", "--db", db, "--out", out, "--name", "e2e", "--view", view, "--no-json")
    if code:
        raise E2eError(f"bruce report digest through the guide's view exited {code}:\n{stdout}{err}")
    (written,) = out.iterdir()
    return written.read_text()


def check_guide_view(builtin: str, through: str) -> list[str]:
    """Every way the digest written through the guide's view is not the built-in view's
    with docs.fp.o's palette: it differs from it on palette lines alone, those it
    changes are its colours, and none of the built-in view's accent is left."""
    problems: list[str] = []
    changed = [
        line[1:]
        for line in difflib.unified_diff(
            normalise_digest(builtin).splitlines(), normalise_digest(through).splitlines(), lineterm="", n=0
        )
        if line[:1] in "+-" and line[:3] not in ("+++", "---")
    ]
    if not changed:
        problems.append("the guide's view changes nothing of the built-in view's digest")
    other = [line for line in changed if not PALETTE_LINE.match(line)]
    if other:
        problems.append(f"the guide's view changes more than the palette: {other[:3]}")
    if GUIDE_ACCENT not in through:
        problems.append(f"the guide's view lacks docs.fp.o's accent {GUIDE_ACCENT}")
    if BUILT_IN_ACCENT in through:
        problems.append(f"the guide's view leaves the built-in accent {BUILT_IN_ACCENT} in the page")
    if through.count(":root") != builtin.count(":root"):
        problems.append("the guide's view has not both of the palette's :root rules")
    return problems


def records(text: str) -> list[list[str]]:
    """`text` read as CSV, each record a list of its fields."""
    return list(csv.reader(io.StringIO(text, newline="")))


def czech_share(part: int, whole: int) -> str:
    """The share the Czech view prints, from the JSON's totals: to the nearest tenth of a
    percent, a half up, with a decimal comma and a no-break space before the sign. Decimal
    arithmetic, so it does not copy the view's integer formula."""
    tenth = decimal.Decimal("0.1")
    pct = (decimal.Decimal(part) * 100 / decimal.Decimal(whole)).quantize(tenth, rounding=decimal.ROUND_HALF_UP)
    return f"{pct}".replace(".", ",") + "\u00a0%"


def check_views(
    js: str, cs: str, links: str, golden_cs: Path = GOLDEN_CS, golden_csv: Path = GOLDEN_CSV, save: bool = False,
    port: int = PORT,
) -> list[str]:
    """Every way the Czech and the CSV view's output differs from what the site
    promises, empty when there is none. Their golden files are written anew with `save`."""
    problems: list[str] = []
    doc = json.loads(js)
    seen = normalise_digest(cs)
    if '<html lang="cs">' not in cs:
        problems.append("the Czech view is not a Czech document")
    for english in ("Figures", "Housekeeping", "How to read", "Written as", "Redirected links", "Unchecked links"):
        if english in cs:
            problems.append(f"the Czech view holds the English {english!r}")
    totals = doc["totals"]
    share = czech_share(totals["broken_url"] + totals["broken_anchor"], totals["links"])
    for czech in ("Přehled kontroly odkazů", "Údržba", "Jak číst tento přehled", share):
        if czech not in cs:
            problems.append(f"the Czech view lacks {czech!r}")
    rows = records(links)
    if rows[:1] != [CSV_COLUMNS]:
        problems.append(f"the CSV view's header is {rows[:1]}, not {CSV_COLUMNS}")
    wide = [r for r in rows if len(r) != len(CSV_COLUMNS)]
    if wide:
        problems.append(f"{len(wide)} CSV records are not {len(CSV_COLUMNS)} fields wide: {wide[0]}")
    listed = collections.Counter((r["page"], r["raw_href"]) for r in doc["broken"] + doc["errors"])
    got = collections.Counter((r[0], r[3]) for r in rows[1:])
    if got != listed:
        problems.append(f"the CSV view's links are not the JSON's: {list((got - listed) + (listed - got))[:3]}")
    for text, golden in ((seen, golden_cs), (links, golden_csv)):
        if save:
            golden.parent.mkdir(parents=True, exist_ok=True)
            golden.write_text(text)
        elif port != PORT:
            print(f"views: {golden.name} not compared, it is the port {PORT} crawl's")
        elif not golden.exists():
            problems.append(f"no golden file {golden}: run with --save-digest")
        elif text != golden.read_text():
            problems.append(f"the view's output is not {golden}: read the diff, then run with --save-digest")
    return problems


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


#: An unchanged build of `tools` has nothing introduced and no redirected link: each
#: page's home link reaches the site's `index.html`, a meta refresh, as the baseline's does,
#: and matches it with its final `/` toggled (D101).
UNCHANGED_REDIRECTED: set[tuple[str, str, str]] = set()
#: The xref through the page-alias alone, its page a meta refresh (D57).
PLANTED_REDIRECTED = {("tools/usage/", "#ops::disks.adoc", "refresh")}
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
    # a link no reader can follow, judged as a crawl judges it
    ("tools/usage/", "hhttps://example.org/tools/", "anchor", "broken_url", "bad_scheme", "new"),
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
        # The footer's three root-relative links, 404 in the baseline, on each of three pages,
        # and usage's `file:` link (D66).
        expect("unchanged already broken", doc["already_broken"]["count"], 10)
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
        expect("planted already broken", doc["already_broken"]["count"], 7)
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
    ap.add_argument("--save-digest", action="store_true", help="write the digest's HTML as the golden file")
    ap.add_argument("--save-pages", action="store_true", help="copy a few built pages into fixtures/antora/pages/")
    args = ap.parse_args(argv)
    server = pub_server = None
    try:
        ui = check_ui()
        antora = install_antora()
        repos = materialise(FIXTURES / "content", OUT / "repos")
        whole = build(
            antora, "whole", [repos[c] for c in COMPONENTS], "guide::index.adoc", ui, OUT,
            f"http://127.0.0.1:{args.port}{SITE_PATH}", languages=True,
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
        written = digest(bin, crawl_db, OUT / "digest")
        js, html = written["json"], written["html"]
        problems = check_digest(js, html, args.port, save=args.save_digest)
        problems += check_views(js, written["cs.html"], written["links.csv"], save=args.save_digest, port=args.port)
        if problems:
            raise E2eError("the digest is not what the site promises:\n  " + "\n  ".join(problems))
        print(f"digest: {len(html)} bytes of HTML, {len(js)} of JSON, as the golden file has it")
        print(f"views: {len(written['cs.html'])} bytes of Czech, {len(written['links.csv'])} of CSV, as their golden files have them")
        problems = deployment.exercise(bin, config, OUT / "deployment")
        if problems:
            raise E2eError("the deployment guide's scripts are not what it promises:\n  " + "\n  ".join(problems))
        print("deployment: two nights of the guide's job published, and the guide's pull fetched them")
        guide_dir = OUT / "deployment" / "guide"
        problems = check_guide_view(html, guide_view(bin, crawl_db, guide_dir / GUIDE_VIEW, OUT / "digest-guide"))
        if problems:
            raise E2eError("the guide's digest view is not what it promises:\n  " + "\n  ".join(problems))
        print("view: the guide's, docs.fp.o's palette over the built-in digest")
        problems = check_workflow(guide_dir, OUT / "antora" / "node_modules")
        if problems:
            raise E2eError("the guide's CI workflow is not what e2e runs:\n  " + "\n  ".join(problems))
        pub = deployment.published(OUT / "deployment")
        pub_server = deployment.serve_directory(pub, 0)
        pub_url = f"http://127.0.0.1:{pub_server.server_address[1]}"
        ci_playbook = playbook([repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT / "ci" / "ci-site", OUT / "cache")
        ci_args = (guide_dir, args.port, OUT / "antora", ci_playbook, OUT / "ci")
        ci = (args.port, config, baseline, ci_db)
        unchanged = check_dir(bin, partial, *ci)
        ci_unchanged = ci_run(*ci_args, pub_url)
        (repos[PARTIAL] / "modules" / "ROOT" / "images" / "diagram.svg").unlink()
        image_build = build(antora, "partial-image", [repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT)
        image_only = (check_dir(bin, image_build, *ci), check_dir(bin, image_build, *ci, "--inverse", "warn"))
        run(["git", "checkout", "--", "."], repos[PARTIAL])
        plant(repos[PARTIAL])
        changed_build = build(antora, "partial-changed", [repos[PARTIAL]], f"{PARTIAL}::index.adoc", ui, OUT)
        changed = check_dir(bin, changed_build, *ci)
        ci_planted = ci_run(*ci_args, pub_url)
        broken = OUT / "deployment" / "broken"
        shutil.rmtree(broken, ignore_errors=True)
        shutil.copytree(pub, broken, copy_function=os.link)
        (broken / sums_baseline(pub)).unlink()
        (broken / sums_baseline(pub)).write_bytes(b"not the baseline")
        broken_server = deployment.serve_directory(broken, 0)
        try:
            ci_broken = ci_run(
                *ci_args, f"http://127.0.0.1:{broken_server.server_address[1]}", only=[deployment.FETCH_STEP]
            )
        finally:
            broken_server.shutdown()
            broken_server.server_close()
        ci_missing = ci_run(*ci_args, f"{pub_url}/nothing", only=[deployment.FETCH_STEP])
        problems = check_ci_jobs(ci_unchanged, ci_planted, ci_broken, ci_missing)
        if problems:
            raise E2eError("the guide's CI workflow is not what it promises:\n  " + "\n  ".join(problems))
        print("ci workflow: the unchanged build passes, the planted one fails, a bad set or none fails the job")
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
        for s in (server, pub_server):
            if s:
                s.shutdown()
                s.server_close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
