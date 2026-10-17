# P2.4 Antora's Extract: notes

What the next slices inherit, and what was tried and refused. The intent is
[`roadmap-P2-extractor.md`](roadmap-P2-extractor.md), "Antora's coordinates";
what exists is the code (`src/profile/antora.rs`, `src/profile.rs`) and
[`../status/STATUS.md`](../status/STATUS.md).

## What the next slices inherit

- **A finish step is a boxed `'static` closure** (`profile::Finish`), no
  longer a `fn` pointer. A profile's handlers and its finish step share what
  the pass read through an `Rc<RefCell<…>>` the closure owns; Antora's holds
  whether a nav container was seen and the search script's
  `data-site-root-path`. It cannot borrow the `'h` sink: the core takes the
  sink back before the finish step runs, and a closure tied to `'h` keeps
  every handler's borrow alive past the pass, which fails to compile.
- **`antora`'s facts, in order:** `component` and `version` from the first
  `.nav-container`, each only where its attribute is present; then, from the
  finish step, `component` from the URL (no nav container at all) and
  `module` and `rel_path`. `rel_path` keeps its `.adoc` and its directories
  (`policy/community-publishing-platforms.adoc`), as an Antora resource ID
  spells it. Module and path are percent-decoded; `component` from a URL too.
- **The URL fallback needs `data-site-root-path`** (I13), which only the lunr
  extension's search script writes. A site without search gets no component
  on a `without_menu` page. On docs.fp.o the root is relative on every page
  and absolute (`/en-US`) on the 404, and `Url::join` reads both.
- **A tag is the href less one leading `#`**, everything after kept, so
  `program_management::changes_policy.adoc#_system_wide_changes` reaches
  Resolve whole. An `a.xref.unresolved` whose href lacks the `#` is not
  tagged; none exists (I6).
- **Fixture facts.** Every non-404 docs.fp.o page records component, module
  and `rel_path`; 28 record a version (`""` 15, `master` 11, `f39`, `f44`),
  and `arm-sbc` none, its component from the URL. On every page with a nav
  container, its component equals the URL's first segment under `/en-US/`,
  which is what makes the fallback's premise checkable. The ten tagged links
  are the survey's nine and `generators/antora/tabs.html`'s one. The 404 is
  `generic` (no generator meta), so its attribute-less nav container is moot.
- **Tests.** `tests/extract.rs` asserts the coordinates and the tags over the
  fixtures; the URL and edit-path parsing cases are in `src/profile/antora.rs`.
  Each Antora fixture's snapshot gained its facts.

## Negative results

- **Putting the site root on `Settled`** was not done: it is Antora's
  attribute, and the core reading it would make the core a library of one
  profile (D32).
- **A tag with the leading `#` kept** was not taken: I6 defines the refspec
  as what follows it, and Resolve would strip it on every call.
