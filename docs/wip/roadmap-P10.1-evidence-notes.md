# P10.1 — Evidence: notes

Landed no `src/` code: `scripts/fixtures.py`'s `sites` step and the test
harness's skip. What the later
slices inherit and what the pages showed, read at `d88ee2b` on 2026-10-31.

## What is saved

`fixtures/docs-antora-org/` (13 pages, Antora 3.2.0, the default UI) and
`fixtures/sphinx-doc-org/` (9 pages, Sphinx's own documentation), each with a
`manifest.json` recording why a page is there, its hash and the date. The
lists are `SITES` in `scripts/fixtures.py`; `test_fixtures` holds every file
to its recorded hash. Neither `robots.txt` carries a `Crawl-delay`; the
saver keeps to one request a second anyway.

`tests/common`'s `fixtures()`, which the extraction snapshots and the bench
walk, skips both directories, so neither the snapshots nor the bench's corpus
move; the slice extracting a site's pages first reads them from there.

## What the Antora pages show (10.6's acceptance reads them)

- Every page, the 404 included, carries `<meta name="generator" content="Antora
  3.2.0">` and a `.nav-container`, so compiled `antora` is detected by the
  generator alone.
- `data-component` and `data-version` are on every 200 page. The unversioned
  component (`antora-ui-default`) renders `data-version=""`, and its URLs omit
  a version. The 404 renders the container with neither attribute.
- The edit link is `div.edit-this-page > a`, text "Edit this Page", no `title`
  or `rel`, onto `gitlab.com/antora/<repo>/-/edit/<ref>/<path>`; the older
  `antora/3.1` page's ref is `v3.1.x`, the rest `main` or `master`.
- `antora/3.1`'s canonical is the `latest` URL; `maven-plugin/latest/antora-goal`
  carries no canonical link.
- No page has an `a.language` menu or an `hreflang` alternate (`code.language-*`
  is a highlighted block's class).

## What the Sphinx pages show (10.3's and 10.4's fixture)

- No `generator` meta and no edit link: the profile is chosen only by a root
  naming it (spec, "Auto picks a declarative profile"), and a source can only
  come from a rule over the page's URL.
- The content is `div.body[role=main]`, which `generic`'s `[role=main]` region
  already matches, so a check that a declarative `region` took effect needs a
  region that differs (`div.document`, which holds the sidebar's links too).
- `meta[name=readthedocs-version-slug]` (`master`) is on every page, 404
  included: a `selector` plus `attr` fact with no `pattern`. It is Read the
  Docs' addon, not Sphinx's; `html[data-content_root]` is Sphinx's.
- **10.4 inherits a gap in the spec's source rule.** The page's source is
  `github.com/sphinx-doc/sphinx`, `doc/usage/quickstart.rst` at `master`, but
  the URL `/en/master/usage/quickstart.html` names neither the repo nor the
  `doc/` prefix nor the extension, and a regex's named captures cannot supply
  a literal. A rule needs a way to state constants (or a template over the
  captures) for `from = "url"` to serve a site like this one; 10.4 settles it.

## The "before" crawl

Launched 2026-10-31T16:47 UTC from `runs/p10-before-20261031-1647/bruce`, a
copy of the release binary built at `d88ee2b`, with a copy of `sim/bruce.toml`
there, into `en-us.duckdb` in the same directory; the files that directory
holds are in its `HANDOFF.md`. It is a **first crawl**: every database under
`runs/` was deleted before P10 (schema 6), so there was no baseline to copy and
`mise run crawl` would have refused without `--baseline`.

It finished at 18:04 UTC (1 h 17 min), `exit` 0: run 1, 173,502 targets, 9,002
pages (2,329 `antora`, 6,673 `generic`), 69 sitemap files; `bruce report
stats` counts 2,326 pages with anchor links, 826 broken URLs, 712 broken
anchors, 87 errors. Those are the figures 10.8's run must reproduce.

10.8 copies that database (`review.py crawl`, whose default is the newest
finished crawl's, this one) and crawls into the copy,
so both runs sit in one database. The guide's `fedora` extension makes its
config differ from this one's.
