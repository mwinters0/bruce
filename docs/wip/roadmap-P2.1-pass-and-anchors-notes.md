# P2.1 The pass and its anchors: notes

What the next slices inherit, and what was tried and refused. The intent is
[`roadmap-P2-extractor.md`](roadmap-P2-extractor.md); what exists is the code
(`src/extract.rs`, `src/extract/anchors.rs`, `src/profile.rs`) and
[`../status/STATUS.md`](../status/STATUS.md).

## What the next slices inherit

- **`Sink` carries parts nothing reads yet.** `take_link_tag` is the core's to
  call when it reads a link (P2.2), and `preferred_edit_link` when it settles
  the edit link (P2.3). `Settled` holds the page's URL only; P2.3 adds the
  chosen edit link. How Antora's `data-site-root-path` reaches its finish step
  is P2.4's: it is Antora's own attribute, so its handler reads it and the
  finish step needs a place to find it.
- **The core's handlers run before the profile's** on one element (title,
  then anchors, then `Extract::handlers`), so a profile's handler may read
  what the core's flag handlers settled but never the reverse.
- **A region selector is one compound selector.** The core extends each with
  `[id]`, ` [id]` and ` a[name]` into one list, so `main` and `[role=main]`
  work and `a + b` does not; an unparseable one panics, being a profile's bug.
  P2.4's `article.doc` fits.
- **Anchors are one row per distinct value per element.** `<a id=x name=x>`
  gives one, `<a id=i name=n>` two, a repeated id on two elements two. Nothing
  deduplicates across elements; the store's join does not need it. `name`
  counts on `<a>` only (D12) and an empty `id=""` on none.
- **The title is the first `<title>` outside an `<svg>`**, entities decoded
  and ASCII whitespace collapsed; none, or an empty one, is `None`.
- **Fixture facts.** Over the 30 docs.fp.o pages the anchors come to 300
  `content` and 150 `chrome`, none `unknown` (the survey's 329 inside counted
  29 empty `id=""`). The 404 has a `<main>`, so `generic` finds a region there
  too. Every page has the five chrome ids (I9).
- **Tests.** `tests/extract.rs` holds the hand-written assertions and one
  snapshot a fixture (`tests/snapshots/`); `tests/common/mod.rs` finds the
  fixtures and is shared with the bench by `#[path]`. A page's URL there is its
  docs.fp.o one, or a stand-in under `example.org`, and P2.2's relative links
  resolve against it. Regenerate with `INSTA_UPDATE=always cargo nextest run`.
  The pass's own cases, against a stub profile, are `src/extract/tests.rs`:
  `extract` cannot name `generic` or `antora` (D39).
- **The bench runs in the suite.** `[[bench]] test = true` makes `cargo test`
  and nextest list `bruce::bench/extract` and run it once in criterion's test
  mode (about 0.3 s), so `check.py` needed no change and `--affected` reaches
  it through the fixtures path like any test.

## Negative results

- **`Element::on_end_tag` cannot carry a borrow**: its handler is `'static`,
  so tying a title's text to its element cannot close over the state. The pass
  counts `<title>` elements and takes the text of the chosen one's number.
- **Strict mode refuses `<select><xmp>…`**, as the spike found. Non-strict
  reads the `<xmp>` body as text, so an id inside it is no anchor, as in a
  browser.
- **The generator pre-pass stops at `<body>`** by returning an error from a
  handler. A generator meta after `<body>` starts is ignored; none of the 40
  fixtures has one.
