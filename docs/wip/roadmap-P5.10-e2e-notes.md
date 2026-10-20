# P5.10 notes: the end-to-end test

Outcome: `mise run e2e` (`scripts/e2e.py`) builds the fixture site whole,
serves it under `/en-US/`, runs `bruce crawl` and `export-baseline` on it, and
runs `check-dir` on four builds of `tools` against that baseline, asserting
the exit status and every section of the JSON (`verify`, with `UNCHANGED_*`,
`PLANTED_*` and `IMAGE_ONLY_INVERSE` as the expected rows, and
`test_e2e.py` testing `verify`, `plant` and the server without Antora):

- **unchanged**: exit 0, nothing introduced or inverse, no `error`, three
  redirected links, nine already broken;
- **the image alone deleted**: inverse breakage of kind `file` and nothing
  else, exit 1, and 0 under `--inverse warn`;
- **planted**: exit 1 on the six introduced and the three inverse rows
  `PLANTED_*` lists, `report introduced` and `report inverse` on that run
  agreeing with the gate's JSON.

Nothing leaves the machine: the configuration allowlists no host, and the
fixture's one external link is recorded and not checked.

## What the next slice inherits

- **P5.11 will see the home link as an introduced redirected link on every
  built page.** The partial build's `index.html` refreshes to its start page
  where the baseline's refreshes to the site's, so each page's breadcrumb link
  `..` lands on the build's start page: listed, never failing
  (`UNCHANGED_REDIRECTED`). quick-docs' build has the same `index.html`.
- **The footer's three root-relative links are the nine already-broken rows**
  of the unchanged build, three on each of its three pages: the fixture server
  has no `/en-US/legal/`, so the baseline has them broken, and the prefix
  resolves the partial build's the same way. A real site that serves them
  shows none.
- **The planted deleted page is two different rows.** An xref to it from the
  same component is `unresolved_xref` (Antora had the whole catalog, so
  nothing resolves it), and a `link:` to its URL is `not_built`. An xref
  from another component's page is not in the build, so it is inverse
  breakage, `removed = page`.
- **A removed anchor is a `broken_anchor` row only where a built page links to
  it.** `tools` linking its own `#running` is that row; `ops` linking it from
  outside is the inverse row, the only one without the first.
- **`error` is empty** in every run here, so the question P5.8 left for P5.11,
  whether the errors section is noise, is not answered by this fixture.

## Negative results

- **The baseline must be built with edit links.** The whole build was first
  built with no `origin` on its repositories, as the partial one is, and every
  xref into `guide` and `ops` stayed `unresolved_xref` in the unchanged
  build: `antora`'s Resolve finds a page by its `module` and `rel_path`, which
  come only from the edit link (KD7). Giving the whole build's repositories a
  github.com `origin` makes Antora write the link, as production's pages have
  one (I15); the partial build keeps none, as a content repo's CI build has
  none. The saved whole pages carry the link now, and no Rust test reads them.
- **A block image macro in a paragraph is text.** `An image: image::x.png[]`
  renders the macro's own characters, so the planted broken image is on a line
  of its own.
- **The fixture's old `guide` index and `ops` pages did not link to anything
  `tools` loses.** The fixture gained an image in `tools`, embedded by
  `guide`'s `upgrade`, and a link from `ops`' index to `tools`' `reference`,
  so each `removed` value has a linking page outside the build. None of the
  saved pages changed.
