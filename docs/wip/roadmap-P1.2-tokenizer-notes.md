# P1.2 Tokenizer spike: notes

What landed, and what the next slices inherit. The row is `P1.2` in
[`../status/STATUS.md`](../status/STATUS.md), its intent in
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md). The outcome is D11:
lol_html.

## What landed

`spikes/tokenizer/`, a standalone Cargo project (no workspace, its own
`Cargo.lock`), the spec's workload written three times over one shared
`Raw` → `Facts` finalizer:
- `gum.rs`: html5gum's `Token` iterator, plus the open-element stack both
  html5gum variants share;
- `gum_cb.rs`: html5gum's `CallbackEmitter`, borrowed slices;
- `lol.rs`: lol_html, the workload as selectors;
- `oracle.rs`: the same workload over html5ever's tree (scraper), the
  correctness reference, since it builds the DOM a browser would;
- `main.rs`: every candidate against the oracle on all 40 fixtures, then on
  26 hand-written edge cases the fixtures lack;
- `benches/extract.rs`: criterion over the 30 docs.fp.o pages;
- `tests/selectors.rs`: the selectors lol_html compiles and refuses.

Reproduce from the commit that lands this: `cd spikes/tokenizer && cargo run
--release` (correctness), `cargo bench --bench extract` (speed), `cargo
test`. Measured with rustc 1.98.0 on a Ryzen 9 3900X; the logs are
`runs/tokenizer-spike/`.

## The numbers

Correctness: all three candidates give the oracle's facts on **40 of 40**
pages (488 anchors, 7540 links). On the edge cases, html5gum matches 24 of
26, lol_html 25 of 26.

Speed, one iteration being all 30 docs.fp.o pages (1,607,544 bytes), the
criterion median. Two runs differed by up to 8%, so a gap under that is
noise.

| Run | Workload | Bare (start tags counted) |
|---|---|---|
| html5gum, `Token` iterator | 30.2 ms, 51 MiB/s | 19.7 ms, 78 MiB/s |
| html5gum, `CallbackEmitter` | 23.3 ms, 66 MiB/s | 10.3 ms, 149 MiB/s |
| lol_html | 24.2 ms, 63 MiB/s | 8.3 ms, 184 MiB/s |
| html5ever tree (oracle) | 55.1 ms, 28 MiB/s | — |

The workload costs about 14 ms over either bare tokenizer, almost all of it in
the shared code's owned `String`s, so the tokenizer is not where a page's time
goes. At I2's `Crawl-delay` the pass is under 0.1% of a page's budget.

Code: `lol.rs`'s extract is about 130 lines, `gum.rs`'s about 120 with the
stack, `gum_cb.rs`'s 45 more on the same stack. html5gum pulls in 2 crates,
lol_html 42.

## What the next slices inherit

- **P1.3's scaffold must keep `spikes/` out of the crate.** `file_limits.py`
  walks a root package's whole tree ("A root package's walk reaches the nested
  members too"), and `concepts.py` claims any `src/` below a member, so a root
  `bruce` package sees `spikes/tokenizer/src/` unless the workspace excludes it
  and the scripts skip it.
- **P1.5's Extract hook, drawn from lol_html.** The core runs one
  `HtmlRewriter` a page; a profile contributes `(selector, handler)` pairs to
  it rather than receiving tokens. Region membership is a descendant selector
  (`article.doc [id]`), so no stack. lol_html has no parent or ancestor access
  from a handler: what a handler needs to know about an element's context is
  a flag set by an earlier handler matching a contextual selector. Handlers on
  one element run in registration order (lol_html 3.0.1
  `src/rewriter/handlers_dispatcher.rs`, `HandlerVec`), which makes that
  pattern sound.
- **lol_html hands over raw source.** Attribute values and text are
  undecoded, and text arrives in chunks split anywhere. Without
  `htmlize::unescape_attribute`, 3 of the 40 pages get wrong hrefs
  (`&amp;` in bugzilla query strings). Text is buffered to
  `last_in_text_node()` before decoding, so an entity is never split.
- **`with_strict(false)`.** lol_html's default aborts the whole page on
  markup it cannot place without a tree (`<select><xmp>`); for reading, its
  guess there matched the oracle.
- **D33's selector vocabulary** is lol_html's: type, class, id, attribute
  (`=`, `~=`, `^=`, `*=`, the `i` flag), descendant and child combinators,
  `:not`, `:nth-child`, `:first-child`, selector lists. Refused: `+`, `~`,
  `:has`, `:last-child`, `:nth-last-child`, pseudo-elements.

## Negative results

- **html5gum must be told to switch states.** `naively_switch_states` is off
  by default, and without it `<script>`, `<style>`, `<textarea>` and `<title>`
  bodies tokenize as markup: 17 of 26 edge cases instead of 24. The fixtures
  never show it.
- **html5gum reads a CDATA section in SVG as a bogus comment**, ending at the
  first `>`, so markup after it becomes elements. Fixing it needs a custom
  `Emitter` answering whether the current node is foreign.
- **None of the three turns `<image>` into `<img>`**, as the tree builder
  does. One line wherever the image link kind is decided.
- **The oracle reads `<template>` content as document content**, as scraper
  attaches it to the tree; a browser does not scroll to it. All three
  candidates agree with the oracle there, and so all are wrong the same way.
- **docs.fp.o's 404 page carries no generator meta**, so `auto` (D35)
  profiles it `generic`, its region drawn by `<main>` rather than its
  `article.doc`. Filed to the P2 inbox.
- **The fixtures exercise none of the edge cases**: every candidate agreed on
  all 40 pages, including the `Token`-iterator html5gum with state switching
  off. The edge-case list in `main.rs` is the only evidence of where the
  tokenizers part.
