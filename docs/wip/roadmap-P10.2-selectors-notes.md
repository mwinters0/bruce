# P10.2 — Selectors compiled once per run: notes

Read at `cf8966c` on 2026-10-31. Behaviour is unchanged: the extraction
snapshots and the 30-page fixture corpus pass untouched.

## What the next slice inherits

`profile::selectors` (re-exported from `profile`) holds the one parse of each
selector text: `compiled(text)` returns a `&'static Selector` from a process-wide
cache, and `element` and `text` are `lol_html::element!` and `text!` over it, so
a handler borrows the selector (`Cow::Borrowed`) and parses nothing. Every
selector the code writes goes through it (the core's title, link, edit-link and
generator readers, `antora`'s four, a profile's region and the hash scope's
roots, excludes and their `{r} *` descendants).

- **10.3's declarative profile needs no ownership of its selectors.** A
  `[profile.<name>]` rule's selector, `region` and `locale_links` are `compiled`
  by their text when its handlers are built; the loader still validates by
  parsing (`config::selectors`) and need not hand its result on. A text lol_html
  refuses is not cached and `compiled` returns the error, so a loader can use it
  as the check.
- **`Extract::region` is still `&'static [&'static str]`.** A declarative
  profile's region is config text, so 10.3 widens it (to `&[String]` or a
  `Cow`); the consumers (`anchors`, `hash`) already take `&[&str]` and pass the
  text to `compiled`.
- **The cache leaks by design**: one entry per distinct text, bounded by the
  selectors the code and the config write, never by pages. Failure is not
  cached.

## The negative result

Parsing was a small part of a page's selector cost: the 30 docs.fp.o pages'
`extract` bench went from 49.5 ms (49.07 to 50.02) to 48.5 ms (48.05 to 48.89),
criterion's "within noise threshold". What remains is lol_html building its
matching program from the parsed selectors in `HtmlRewriter::new` on every page
(`rewrite_controller.rs`, `Ast::add_selector`), which its public API cannot reuse
across rewriters. The saving a run should expect from this slice is a few
percent of extraction, not a step; more would take fewer handlers per page (the
hash's `*` handler and the `{r} *` descendants are the widest), which changes
the pass and is not this slice's.
