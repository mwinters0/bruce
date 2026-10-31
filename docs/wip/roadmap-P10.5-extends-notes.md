# P10.5 — `extends`: notes

Read on 2026-10-31 at `fc4f5fa` plus this change. With no `extends` every extraction, crawl and report is what it
was: the 40-page corpus and its snapshots pass untouched. The row was split (below); the rest is 10.5.1.

## What the next slice inherits

- **`Definition::extends`** (`config::profile`) is the base's name, checked for shape and for not naming the
  profile itself; `cli::registry_for` is what knows the compiled names (`generic`, `antora`) and refuses another,
  a declared profile included (`profile.b: extends: \`a\` is not a compiled profile`). The recorded row is
  `profile.<name>.extends`, after `generator` and only where set, so every definition recorded before 10.5 is
  unchanged. 10.5.1 reads that row for the base's sitemap note.
- **`Declarative::new(&Definition, Option<Box<dyn Profile>>)`** wraps the base. Its Extract is the base's, then the
  extension's region (if stated), fact handlers, edit-link handler and sources; its finish runs the base's first.
  `detect` is its `generator` pattern, else the base's; `resolve` and `sitemap_note` are the base's; `views` is
  empty, the base being in every registry (`registry_for` always adds the compiled two) and installing its own once.
- **`Profile::namespace`** (default `name`) is what `Sink::new` takes in `extract`, and `Sink::fact_in` writes under
  another: an extension's namespace is its base's, so the base's `fact` calls keep `antora.*` and the extension's rules
  write `<name>.*` through `fact_in`. `Extracted::profile` stays the profile's name.
- **Reports need nothing built for resolution.** `crawl::xref::resolve` reads `Xref::profile`, the name the run's own
  extraction gave (`crawl.rs`, `defer_xref`), from the registry the run's config built, so resolution reads no recorded
  profile. The one reader of `pages.profile` that a report's registry serves is
  `report::digest::sections::sitemap_notes`, which finds a profile by name among the registry's: an extension's pages
  get no sitemap note from the compiled-only registry `cli` gives a report (`registry()`), which 10.5.1 fixes through
  the run's `extends` row (the spec's "A report reads a declarative profile from its run").

## Behaviours an extension has, as tested

- Over every fixture an extension with no rule extracts what `antora` does, summary and locales (`tests/extract.rs`),
  claiming the pages `antora` claims under `auto` and no other.
- A generator pattern of its own replaces the base's detection; a region of its own replaces the base's, none keeping
  `article.doc`.
- `extends = "generic"` is accepted and reads as `generic` does where it states no rule.

## Negative results

- Putting an extension's own facts in `antora.*` is no option: `antora.component` is read by name in `crawl`, `ci`
  and `report` (D54, D56), and a declarative rule written there could collide with a compiled fact.
- A declared profile's `prefer_edit_link` handler yields to a preference an earlier handler set, the base's coming first.
  `antora` states none, so this binds nothing today; an extension whose base preferred one would lose to it.
- The 10.5 row was a compiled-profile feature and a rework of the report path (a registry built per report, a
  comparison naming a changed rule, which needs a decision on what `compare` shows of a policy: the digest compares no
  policy between runs today). Per `docs/process.md`'s "Slice numbering" the first landed alone and the second is 10.5.1.
