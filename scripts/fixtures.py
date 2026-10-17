#!/usr/bin/env python3
"""Choose and save the docs.fp.o fixture pages (D24, `fixtures/README.md`).

Three steps, each its own subcommand, so the choice is reproducible and the
network is touched only by the first and the last:

- `survey` reads the sitemap index, takes an evenly spaced sample of each
  component's pages, fetches each at the site's `Crawl-delay` (I2) and
  writes what it measured to `fixtures/docs-fp-o/survey.json`, committed so
  the selection can be re-run from a hash without the 12 minutes.
- `select` reads that survey and writes `fixtures/docs-fp-o/manifest.json`:
  the pages chosen for shape, each with the reason it was chosen. It makes
  no request.
- `save` fetches every page the manifest names and writes the raw HTML
  under `fixtures/docs-fp-o/`, recording each page's size, hash and fetch
  date back in the manifest.

The pages are stored as served (`Accept-Encoding: identity`), so a tokenizer
reads what a crawl would. Stdlib only, like every script here.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = "https://docs.fedoraproject.org"
INDEX = f"{HOST}/en-US/sitemap.xml"
AGENT = "bruce-fixtures/0 (+link checker fixture survey)"
# I2: `Crawl-delay: 1`. A survey is a crawl and keeps to it.
DELAY = 1.0
MANIFEST = ROOT / "fixtures" / "docs-fp-o" / "manifest.json"
SURVEY = ROOT / "fixtures" / "docs-fp-o" / "survey.json"
# A survey in progress, one measurement a line (gitignored with `runs/`).
PARTIAL = ROOT / "runs" / "fixtures" / "survey.jsonl"
# I2: `Disallow: /*/Fedora*/` -- a capital F, so the lower-case component is
# not caught. Pages under these are never fetched.
DISALLOWED = re.compile(r"^/[^/]+/(Fedora[^/]*|Community_Services_Infrastructure)/")
FROZEN = re.compile(r"/en-US/fedora/f3\d/")
TARGET = 30
# The pages the invariants' re-verify commands name (I6, I8, I9), measured
# whether or not the sample reaches them.
WITNESSES = {
    f"{HOST}/en-US/quick-docs/bugzilla-file-a-bug/": "witness of I6",
    f"{HOST}/en-US/packaging-guidelines/ReviewGuidelines/": "witness of I8 and I9",
}


def fetch(url: str, attempts: int = 4) -> tuple[int, bytes]:
    """GET `url`; an HTTP error status is a result, not an exception.

    A dropped connection is retried after a pause that doubles, since a
    survey of hundreds of pages meets one.
    """
    req = urllib.request.Request(
        url, headers={"User-Agent": AGENT, "Accept-Encoding": "identity"}
    )
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                return resp.status, resp.read()
        except urllib.error.HTTPError as err:
            return err.code, err.read()
        except (urllib.error.URLError, TimeoutError, ConnectionError):
            if attempt == attempts - 1:
                raise
            time.sleep(DELAY * 5 * 2**attempt)
    raise AssertionError("unreachable")


def locs(xml: str) -> list[str]:
    return re.findall(r"<loc>([^<]*)</loc>", xml)


def sample(urls: list[str], n: int) -> list[str]:
    """At most `n` of `urls`, evenly spaced, the first always included."""
    if len(urls) <= n:
        return list(urls)
    return [urls[i * len(urls) // n] for i in range(n)]


def attr(tag: str, name: str) -> str | None:
    m = re.search(rf'\b{name}="([^"]*)"', tag)
    return m.group(1) if m else None


# An opening tag, and the two marks of an `@asciidoctor/tabs` block on it (I9):
# a `tabs` class token, or an id `_tabs_<n>` (its tabs are `_tabs_<n>_tab_<x>`).
TAG = re.compile(r"<[a-zA-Z][^>]*>")
TAB_SET_ID = re.compile(r'\bid="_tabs_\d+"')
CLASS = re.compile(r'\bclass="([^"]*)"')


def tab_sets(html: str) -> int:
    """How many tab blocks a page holds, each tag counted once."""
    n = 0
    for tag in TAG.findall(html):
        cls = CLASS.search(tag)
        if TAB_SET_ID.search(tag) or (cls and "tabs" in cls.group(1).split()):
            n += 1
    return n


def measure(url: str, status: int, body: bytes) -> dict:
    """What the selection rules read off one page."""
    html = body.decode("utf-8", "replace")
    generator = re.search(r'<meta name="generator" content="([^"]*)"', html)
    canonical = re.search(r'<link rel="canonical" href="([^"]*)"', html)
    nav = re.search(r'<div class="nav-container[^"]*"[^>]*>', html)
    return {
        "url": url,
        "status": status,
        "bytes": len(body),
        "generator": generator.group(1) if generator else None,
        "canonical": canonical.group(1) if canonical else None,
        "component": attr(nav.group(0), "data-component") if nav else None,
        "version": attr(nav.group(0), "data-version") if nav else None,
        "ids": len(re.findall(r'\sid="[^"]+"', html)),
        "tabs": tab_sets(html),
        "unresolved": len(re.findall(r'class="xref unresolved"', html)),
        "edit_titles": sorted(
            set(re.findall(r'<a href="[^"]*" title="([^"]*)"', html))
            & {"Edit this Page", "Page history", "Report an issue"}
        ),
    }


def survey(
    per_component: int,
    deep: dict[str, int],
    out: Path,
    partial: Path = PARTIAL,
    urls_from: Path | None = None,
) -> None:
    if urls_from:
        # Re-measure an earlier survey's pages, not a fresh sample: the
        # sitemap moves, and the point is the same pages measured again.
        chosen = [p["url"] for p in json.loads(urls_from.read_text())["pages"]]
    else:
        chosen = choose(per_component, deep)
    out.parent.mkdir(parents=True, exist_ok=True)
    # One measurement a line, appended as it is taken: a run that dies is
    # resumed by running it again, which skips the pages already measured.
    # A partial file holds the counts of the `measure` that wrote it, so a
    # changed count starts a fresh one.
    partial.parent.mkdir(parents=True, exist_ok=True)
    done = {}
    if partial.exists():
        for line in partial.read_text().splitlines():
            m = json.loads(line)
            done[m["url"]] = m
    chosen += list(WITNESSES)
    with partial.open("a") as log:
        for i, url in enumerate(dict.fromkeys(chosen), 1):
            if url in done:
                continue
            time.sleep(DELAY)
            status, body = fetch(url)
            done[url] = measure(url, status, body)
            log.write(json.dumps(done[url]) + "\n")
            log.flush()
            print(f"{i} {status} {len(body)} {url}", flush=True)
    pages = [done[u] for u in dict.fromkeys(chosen)]
    out.write_text(json.dumps({"date": str(date.today()), "pages": pages}, indent=1))


def choose(per_component: int, deep: dict[str, int]) -> list[str]:
    """The sample: each sitemap's pages, evenly spaced."""
    status, body = fetch(INDEX)
    if status != 200:
        sys.exit(f"{INDEX}: HTTP {status}")
    chosen: list[str] = []
    for sitemap in locs(body.decode()):
        time.sleep(DELAY)
        status, body = fetch(sitemap)
        if status != 200:
            sys.exit(f"{sitemap}: HTTP {status}")
        urls = [
            u
            for u in locs(body.decode())
            if not DISALLOWED.match(urllib.parse.urlparse(u).path)
        ]
        name = re.search(r"sitemap-(.*)\.xml$", sitemap)
        n = deep.get(name.group(1) if name else "", per_component)
        picks = sample(urls, n)
        # I5: the version `latest` stands for is the one real copy.
        picks += [u for u in urls if u.endswith("/fedora/latest/")]
        # The frozen versions are most of the `fedora` component, and a
        # spaced sample can miss them.
        picks += [u for u in sample([u for u in urls if FROZEN.search(u)], 4)]
        chosen += [u for u in dict.fromkeys(picks)]
    return chosen


def select(pages: list[dict], target: int = TARGET) -> list[dict]:
    """Pick pages for shape; each carries the `why` it was picked for."""
    ok = [p for p in pages if p["status"] == 200 and p["generator"]]
    picked: dict[str, dict] = {}

    def take(p: dict, why: str) -> None:
        if p["url"] in picked:
            picked[p["url"]]["why"].append(why)
        else:
            picked[p["url"]] = {"url": p["url"], "why": [why]}

    def top(key: str, n: int, why: str, keep=lambda p: True) -> None:
        for p in sorted(filter(keep, ok), key=lambda p: -p[key])[:n]:
            if p[key] > 0:
                take(p, f"{why} ({key}={p[key]})")

    for p in ok:
        if p["url"] in WITNESSES:
            take(p, WITNESSES[p["url"]])
    top("bytes", 1, "largest page sampled")
    top("tabs", 3, "tabs, I9")
    top("unresolved", 3, "unresolved xrefs, I6")
    top("ids", 1, "most ids")
    # I8: an unversioned component reports a placeholder version its URLs omit.
    # `latest` is I5's alias for a version's own number, not an omission.
    for p in sorted(ok, key=lambda p: p["url"]):
        segments = urllib.parse.urlparse(p["url"]).path.split("/")
        if (
            p["component"]
            and p["version"]
            and p["version"] not in segments
            and "latest" not in segments
        ):
            take(p, f"unversioned component {p['component']}, I8")
            break
    # I5: a frozen version is a full copy whose canonical points at `latest`.
    for p in ok:
        if FROZEN.search(p["url"]) and p["canonical"] and "/latest/" in p["canonical"]:
            take(p, "frozen f3x page, canonical to latest, I5")
            break
    for p in ok:
        if "/fedora/latest/" in p["url"]:
            take(p, "the latest fedora version, I5")
            break
    three = ["Edit this Page", "Page history", "Report an issue"]
    with_three = [p for p in ok if p["edit_titles"] == three]
    if with_three:
        take(min(with_three, key=lambda p: p["bytes"]), "the three edit-this-page links, I12")
    # A page with no nav-container at all: the `without_menu` layout, I8.
    bare = [p for p in ok if p["component"] is None]
    if bare:
        take(bare[0], "no component on the page (the without_menu layout), I8")
    # Fill the rest with one page per component, the smallest first, so the
    # set spans the site rather than its biggest component.
    seen = {ok_p["component"] for ok_p in ok if ok_p["url"] in picked}
    for p in sorted(ok, key=lambda p: p["bytes"]):
        if len(picked) >= target - 1:
            break
        if p["component"] not in seen:
            seen.add(p["component"])
            take(p, f"component {p['component']}")
    return sorted(picked.values(), key=lambda e: e["url"])


def rel_path(url: str, status: int = 200) -> str:
    """Where a page is stored under `fixtures/docs-fp-o/`."""
    path = urllib.parse.urlparse(url).path.strip("/")
    name = "index.html" if status == 200 else f"{status}.html"
    return f"{path}/{name}" if path else name


def save(manifest: Path) -> None:
    doc = json.loads(manifest.read_text())
    for entry in doc["pages"]:
        time.sleep(DELAY)
        status, body = fetch(entry["url"])
        entry |= {
            "status": status,
            "path": rel_path(entry["url"], status),
            "bytes": len(body),
            "sha256": hashlib.sha256(body).hexdigest(),
            "fetched": str(date.today()),
        }
        dest = manifest.parent / entry["path"]
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(body)
        print(status, len(body), entry["path"], flush=True)
    manifest.write_text(json.dumps(doc, indent=1) + "\n")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("survey", help="measure a sample of every component")
    s.add_argument("--per-component", type=int, default=12)
    s.add_argument(
        "--deep", action="append", default=[], metavar="COMPONENT=N",
        help="sample N pages of one component instead (repeatable)",
    )
    s.add_argument("--out", type=Path, default=SURVEY)
    s.add_argument(
        "--partial", type=Path, default=PARTIAL,
        help="the resumable one-line-a-page log (default: %(default)s)",
    )
    s.add_argument(
        "--urls-from", type=Path, metavar="SURVEY",
        help="re-measure the pages an earlier survey.json names, not a fresh sample",
    )
    c = sub.add_parser("select", help="write the manifest from a survey")
    c.add_argument("--survey", type=Path, default=SURVEY)
    c.add_argument("--manifest", type=Path, default=MANIFEST)
    v = sub.add_parser("save", help="fetch every page the manifest names")
    v.add_argument("--manifest", type=Path, default=MANIFEST)
    args = ap.parse_args()
    if args.cmd == "survey":
        deep = {k: int(v) for k, v in (d.split("=") for d in args.deep)}
        survey(args.per_component, deep, args.out, args.partial, args.urls_from)
    elif args.cmd == "select":
        pages = json.loads(args.survey.read_text())["pages"]
        entries = select(pages)
        entries.append({"url": f"{HOST}/en-US/no-such-page-bruce-fixture/", "why": ["a 404"]})
        args.manifest.parent.mkdir(parents=True, exist_ok=True)
        # A page kept keeps what `save` recorded of it.
        if args.manifest.exists():
            old = {e["url"]: e for e in json.loads(args.manifest.read_text())["pages"]}
            entries = [old.get(e["url"], {}) | e for e in entries]
        args.manifest.write_text(json.dumps({"pages": entries}, indent=1) + "\n")
        print(f"{len(entries)} pages -> {args.manifest}")
    else:
        save(args.manifest)


if __name__ == "__main__":
    main()
