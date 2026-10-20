# P5.1 notes: the Antora fixture site

Outcome: `mise run e2e` builds `fixtures/antora/content/`'s three components
whole and `tools` alone, into `runs/e2e/{whole,partial}/`, and
`--save-pages` keeps seven of the pages under `fixtures/antora/pages/` with a
manifest. No `bruce` assertion yet. The mechanism is `scripts/e2e.py`'s
docstring; the external behaviour it relies on is `site-invariants.md`, I15.

## What the next slices inherit

- **The site.** `guide` `1.0` (the anchors `supported-hardware`, `first-boot`
  and `rollback`), `tools` `~` (`running`, `options`) and `ops` `master`
  (modules `ROOT` and `admin`, `admin/partitions` aliased from `ROOT:disks`,
  the anchor `layout`). `tools` is the partial build: it xrefs `guide` twice
  and `ops` once, so its partial build holds three unresolved xrefs, all on
  `tools/usage/` (`#guide::install.adoc#supported-hardware`,
  `#1.0@guide::install.adoc`, `#ops:admin:partitions.adoc#layout`), while
  `ops` links `tools:usage#running` and `guide` links `tools:usage`. P5.10's planted changes
  fit it: delete `tools/reference` (linked from `index` and `usage`), remove
  `tools`'s `running` (linked from `ops`), add a broken link, an xref into
  `guide` that does not resolve and one that does.
- **The whole build must be served under `/en-US/`.** The UI writes its
  footer as `{{site.path}}/legal/…` and a language link
  `../../../en-US/<component>/<page>/` on every page. With `site.url`
  `http://127.0.0.1:<port>/en-US`, as `e2e.py` builds, they are production's;
  served at the root, the language link is a 404 on every page of the
  baseline. P5.10's server maps `/en-US/` to `runs/e2e/whole/`, and the
  `check-dir` prefix is `http://127.0.0.1:<port>/en-US/`, where the partial
  build's `/legal/…` footer links resolve under it (P5 spec, "Root-relative
  hrefs resolve against the prefix"). Port 8099 is the default; the saved
  whole-build pages carry it, so `--save-pages` refuses another.
- **What each build holds.** The whole build is 11 `.html` files: `404.html`,
  the start page's redirect at `index.html`, `ops/disks/` (the alias) and
  eight pages. The partial build is 4: the `index.html` redirect to `tools/`,
  `tools/`, `tools/usage/` and `tools/reference/`; no `404.html`, sitemap or
  canonical link. The `ops` and `tools` pages have no version segment in
  their path; `tools` renders `data-version=""` and `ops`
  `data-version="master"`.
- **A partial build's pages have no edit link** under `CI=true` and no hosted
  `origin`, so the fixture pages test P5's "found by the component-versions the
  build holds", not a forge URL; the whole build's, from repositories given a
  github.com `origin`, have one (I15).
- **The build is deterministic**: two runs write byte-identical pages, so
  `--save-pages` shows a diff only when Antora or the bundle changed. The
  sitemaps carry timestamps and are not saved.
- **The UI bundle** is `fixtures/antora/ui-bundle.zip` (688501 bytes),
  vendored by `fixtures.py antora-ui` with the URL, job, commit `7f5fb3d`
  and sha256 in `ui-bundle.json`; `e2e.py` refuses one that does not match.

## Negative results

- **A plain directory is not a content source**: Antora fatals ("must be a
  git repository"), so `e2e.py` commits a copy of each component under
  `runs/e2e/repos/`. `branches: ~` finds no pages in one; `HEAD` builds its
  working tree, so a planted change needs no commit.
- **The job record gives no commit**: gitlab's job API answers 401, so
  `ui-bundle.json` takes the newest successful `main` pipeline's sha, which
  is the job `HEAD` named unless a push landed between the two requests; the
  bundle's file dates (2026-10-02 15:33) match that pipeline.
