# P1.1 Fixtures: notes

What landed, and what the next slices inherit. The row is `P1.1` in
[`../status/STATUS.md`](../status/STATUS.md), its intent in
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md).

## What landed

- **`fixtures/docs-fp-o/`**: 30 saved pages, 1.7 MB, as served. `manifest.json`
  holds each page's URL, why it was chosen, status, size, sha256 and fetch date
  (2026-10-17). `survey.json` holds the measurements the choice was read from.
  `fixtures/NOTICE` carries the CC BY-SA 4.0 attribution.
- **`fixtures/generators/`**: ten hand-written pages. One edit-link page each for
  Docusaurus, MkDocs Material, Docsy, Furo and the Read the Docs theme, one
  Antora tabs page, and four `generic/` pages (`<main>`, `role=main`, no
  landmark, competing edit links).
- **`scripts/fixtures.py`** with `test_fixtures.py`: `survey`, `select`, `save`.
  `select` and `save` are re-runnable and keep what the manifest already
  records. The only tests reading the tree are `Committed`'s, which hold every
  saved page to its recorded hash.

Reproduce the choice with `cd scripts && uv run fixtures.py select`. Repeat the
measurement with `uv run fixtures.py survey --deep quick-docs=90 --deep
infra=40 --deep releases=20`, about 12 minutes at I2's `Crawl-delay`; it
resumes from `runs/fixtures/survey.jsonl` if it dies, as one did here on a
connection reset.

## What the next slices inherit

- **P1.2's workload** is `manifest.json`'s pages. The largest is
  `legal/all-allowed` (508 KB); the median is about 27 KB. Every page carries
  `<meta name="generator" content="Antora 3.1.15">`.
- **Minified markup.** The Material and Docsy fixtures keep their real
  unquoted attributes (`rel=edit`, `class=md-main`) and one-line layout, which
  a tokenizer must read as it does quoted ones.
- **The survey is a sample**: 698 of the 2307 sitemap URLs, up to 12 a
  component and more for the largest. "Largest page" and "most ids" mean
  those of the sample.
- **Fixture 404:** `no-such-page-bruce-fixture/404.html` is the site's own
  not-found page, a full Antora page with the generator meta.
- **Frozen versions:** `fedora/f39/` has `rel=canonical` pointing at
  `/en-US/fedora/latest/`. 14 of the 698 sampled pages have a canonical other
  than their own URL.
- **Unresolved xrefs** (I6) are on 23 of the 698.

## Negative results

- **No docs.fp.o page with tabs was found, and the sample is no evidence.**
  The survey's `tabs` count matches only `<div class="tabs` and measures
  `generators/antora/tabs.html` at zero, so the 698 are unmeasured for tabs
  (`P1.1.1`). The `[tabs]` search of the quick-docs,
  release-notes, beginners-guide, accounts-docs and fedora-coreos-docs sources
  came up empty. The image does carry the extension (`historical/initial.md`).
  `generators/antora/tabs.html` is reconstructed from I9's ids and is not
  observed markup; its comment says so. A real page is `P1.1.1`.
- **Six pages have no `nav-container`.** `arm-sbc`, `docs`, `engineering`,
  `mindshare`, `spins-labs` and `tools` (each the only page of its sitemap)
  are on the `without_menu` layout, so they have no `data-component` or
  `data-version`. They still carry an edit link, to `docs/site-frontpage`
  (`…/src/branch/main/arm-sbc/modules/ROOT/pages/index.adoc`). `arm-sbc/` is
  the fixture. I8 states the layout condition, and the P2 inbox carries what
  the extractor does about it.
- **All three edit links are on every sampled page**, so "the three
  edit-this-page links" picks the smallest such page, not a rare one.
- **gitlab.com serves a Cloudflare challenge** to `curl` on raw files, so
  the asciidoctor-tabs source could not be read there.
