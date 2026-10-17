# P1 Scaffold and parser spike: notes

What P1 left for later phases, and what it tried and refused. Its intent is
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md); what it built is the code,
[`../status/STATUS.md`](../status/STATUS.md)'s "What exists" and the concept
map.

## Reproducing the evidence

- **Fixtures** (`fixtures/`, chosen by `scripts/fixtures.py`). Reproduce the
  choice with `cd scripts && uv run fixtures.py select`. Re-measure with `uv
  run fixtures.py survey --deep quick-docs=90 --deep infra=40 --deep
  releases=20`, about 12 minutes at I2's `Crawl-delay`, resuming from
  `runs/fixtures/survey.jsonl`. A changed `measure` needs a fresh partial
  file: `--urls-from ../fixtures/docs-fp-o/survey.json --partial <new>`, as
  `runs/fixtures/tabs-20261017/` did (its `log` heads with the command).
- **The tokenizer spike** (`spikes/tokenizer/`, its own Cargo project): `cargo
  run --release` (correctness against an html5ever oracle), `cargo bench
  --bench extract` (speed), `cargo test`. Measured under rustc 1.98.0 on a
  Ryzen 9 3900X; logs in `runs/tokenizer-spike/`. It now builds under 1.99.1.

## What P2, the extractor, inherits

- **The workload.** `manifest.json`'s 30 pages, the largest `legal/all-allowed`
  (508 KB), the median about 27 KB. Every page but the 404 carries
  `<meta name="generator" content="Antora 3.1.15">`. The survey is a sample:
  698 of 2307 sitemap URLs. 14 of the 698 have a canonical other than their
  own URL (`fedora/f39/` → `latest`); 23 carry unresolved xrefs (I6).
- **lol_html's numbers** (D11). All three candidates match the oracle on 40 of
  40 pages; on 26 hand-written edge cases html5gum gets 24, lol_html 25. The
  workload over all 30 pages: lol_html 24.2 ms, html5gum's callback emitter
  23.3 ms, its token iterator 30.2 ms, the oracle 55.1 ms, with 8%
  run-to-run noise. About 14 ms of each is the shared code's owned strings,
  so the tokenizer is not where a page's time goes.
- **`Profile::extract` is lol_html's own type**, `Vec<(Cow<'static,
  Selector>, ElementContentHandlers<'h>)>`, appended to
  `Settings::element_content_handlers` unchanged. The facts sink is a
  `&RefCell<Facts>`; `Facts::record` namespaces each key. Region, edit-link
  and tagged-link output have no sink yet: adding them changes the trait's
  signature and both profiles, nothing else.
- **lol_html has no ancestor access from a handler.** Context is a flag set
  by an earlier handler on a contextual selector; handlers on one element run
  in registration order (lol_html 3.0.1, `handlers_dispatcher.rs`,
  `HandlerVec`). Region membership is a descendant selector
  (`article.doc [id]`).
- **lol_html hands over raw source.** Attribute values need
  `htmlize::unescape_attribute` (3 of 40 pages have `&amp;` in hrefs); text
  arrives in chunks and is buffered to `last_in_text_node()` before decoding.
  `with_strict(false)` keeps a page lol_html cannot place without a tree.
- **D33's selector vocabulary is lol_html's**: type, class, id, attribute
  (`=`, `~=`, `^=`, `*=`, `i`), descendant and child combinators, `:not`,
  `:nth-child`, `:first-child`, lists. Refused: `+`, `~`, `:has`,
  `:last-child`, `:nth-last-child`, pseudo-elements.
- **The generator meta is read before a profile is chosen.**
  `Registry::choose` takes its content; the core reads it in a first small
  pass or from the head before the rewriter starts (D35).
- **`content_hash` is a `BLOB`**: D15 names no hash, and P2 picks one.
- **`page_facts` has no uniqueness**; the Antora view takes `max(value)`, and
  its extract records each key once.
- **Minified markup.** The Material and Docsy fixtures keep unquoted
  attributes and one-line layout.

## What P3, the crawler, inherits

- **The writer is a std thread** behind `sync_channel`, a blocking send being
  the backpressure. From tokio the sender needs `spawn_blocking`, or the
  channel becomes tokio's with `blocking_recv`; the thread, appenders and
  `Writer::finish` stay.
- **`pages` has no URL column**: a page's URL is `targets.url` at the same
  `(run_id, url_id)`, so the writer needs a `Row::Target` before a page can be
  found.
- **`[host]` settings are loaded and enforced nowhere**; their precedence is
  `HostSettings`'s rustdoc and the manual. The rest is the P3 inbox.

## What P4 inherits

- **The D6 views are P4's.** A profile's views are `CREATE OR REPLACE`d on
  every open, so a changed view needs no schema version.

## Negative results

- **One tab set in 698 pages, and it is a demonstration**: the Docs team's
  markup guide. Content use of tabs is unobserved; served tab markup is the
  pre-JS form (`_tabs_1_tab_<x>`, `…--panel`), I9's static ids. The survey's
  first count missed it, matching only `<div class="tabs`.
- **Six pages have no `nav-container`**: each a `site-frontpage` index on the
  `without_menu` layout (I8; the P2 inbox has what Extract does).
- **All three edit links are on every sampled page**, told apart by `title`.
- **The 404 page has no generator meta**, so `auto` profiles it `generic`.
- **html5gum must be told to switch states**, or `<script>`, `<style>`,
  `<textarea>` and `<title>` bodies tokenize as markup (17 of 26 edge cases
  instead of 24), and it reads an SVG CDATA section as a bogus comment.
- **No candidate turns `<image>` into `<img>`**, as the tree builder does.
- **The oracle reads `<template>` content as document content**; all three
  candidates agree with it, so all are wrong the same way.
- **The fixtures exercise none of the edge cases**: `main.rs`'s list is the
  only evidence of where the tokenizers part.
- **gitlab.com serves a Cloudflare challenge** to `curl` on raw files.
- **`toml` 1.x reads an integer into an `f64`**, so `per_second = 2` needs no
  decimal point.
- **`url::Host::parse` refuses a URL or `host:port` only as `IdnaError`**, so
  `allowlist` and `host` refuse a `/` or `:` first, to say why.
- **A profile's tests cannot name a concrete profile**: `module_graph.py`
  confines `generic` and `antora` to `cli`, so `profile` and `store` test
  with stubs and the Antora view is tested in `cli`.
- **`information_schema` is the existence check** for `meta`, not a failed
  query, so a file with other tables is left unwritten.
- **No `[workspace]` table** is needed to keep `spikes/tokenizer/` out:
  cargo finds no workspace above it while the root manifest has none, and the
  checks skip a package nested below a member that is not one.
- **`check.py --affected` treats every `src/` change as outside a crate**, a
  root package's directory being `.`; for one crate that is the same set.
