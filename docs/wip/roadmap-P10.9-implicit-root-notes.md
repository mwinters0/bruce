# P10.9 — Antora's implicit ROOT module: notes

Read at `253080d` on 2026-11-01. `module_and_path` (`profile/antora.rs`) takes a `modules/<module>/pages/` tail
first and otherwise the first `pages` segment as `ROOT`'s (I26). docs.fp.o's pages are all under `modules/`, so
its snapshots and 10.8's evidence stand; `docs-antora-org`'s `assembler/latest/` now records `ROOT` and
`index.adoc`, and an xref to it resolves (`an_xref_to_a_page_in_the_implicit_root_module_resolves`).

## What the next slice inherits

- **The ambiguity is a property** (rustdoc of `module_and_path`): an edit path begins at the repo's root and does
  not say where the start path ends, so a start path with a directory named `pages` above the real one is read
  with that directory as the implicit one's. Nothing in the page names the start path; a source rule (D93) states
  it for a site that has one.

## Negative results

- Antora turns the implicit `ROOT` off for a whole content source holding any `modules/` directory
  (`hasModulesDir`), which a page's edit path cannot show: a `pages/` below `modules/` is read as its module's, never as `ROOT`'s. Antora 3.1 (docs.fp.o's, `fixtures/antora`) has no implicit `ROOT`, and a
  `pages/` under a start path there is not a page, so the read cannot misfire on a 3.1 site's built pages.
