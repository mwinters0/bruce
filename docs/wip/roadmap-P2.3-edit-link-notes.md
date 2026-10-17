# P2.3 The edit link and `forge`: notes

What the next slices inherit, and what was tried and refused. The intent is
[`roadmap-P2-extractor.md`](roadmap-P2-extractor.md), "The edit link" and "The
forge parse"; what exists is the code (`src/extract/edit_link.rs`,
`src/forge.rs`) and [`../status/STATUS.md`](../status/STATUS.md).

## What the next slices inherit

- **`Settled::edit_link` is a `forge::EditLink`**, the URL and an
  `Option<Source>` (`repo`, `git_ref`, `path`). The type lives in `forge`, not
  `extract`, because `profile` sits between them (D39) and a finish step must
  be able to name it. A profile's preference through the sink is already folded
  in: the finish step sees one settled choice.
- **`Source::path` keeps the start path.** On docs.fp.o it is
  `arm-sbc/modules/ROOT/pages/index.adoc` or `docs/modules/…`, so P2.4 finds
  the `modules/<m>/pages/<p>.adoc` tail by searching for a `modules` segment
  followed by `pages`, not by anchoring at the start. The path is as the URL
  spells it, percent-encoding included.
- **The edit link is one of the page's links**: a candidate holds the index of
  its `Link`, so it is resolved against `<base href>` like every other href.
  Only `a[href]` is a candidate, and only a web page (`http` or `https`) can be
  chosen. A profile's preferred href is resolved the same way, and one that
  names no web page is ignored in favour of the ranking.
- **Class matches the anchor itself.** docs.fp.o's `edit-this-page` is on the
  wrapping `<div>`, which no handler sees; its anchors are found by their
  `title`. The three known classes (I12) are for Docsy and Docusaurus, whose
  anchors carry them.
- **A phrase is the whole of a title or the link's text**, split into words
  with case, punctuation, icons and entities ignored: "edit this page", "edit
  page" or "edit on …". "Edit this page history" and "Improve this page" are
  not. Text is kept to 200 bytes per anchor.
- **Pagure is tried before GitHub**, so a GitHub path whose first directory is
  literally `f` reads as Pagure's. The first marker from the left decides.
- **Fixture facts.** All 29 non-404 docs.fp.o pages yield a forge-shaped edit
  link to a `.adoc` file, on three hosts: forge.fedoraproject.org (Forgejo),
  github.com and gitlab.com. The 404 and the generic `main-landmark` and
  `role-main` pages have none. Each generator fixture's link is found by the
  signal its theme gives (`tests/extract.rs`).
- **Tests.** The ranking's cases are in `src/extract/tests.rs`, against the
  stub profile, which also prefers a link; the phrase matcher's are in
  `src/extract/edit_link/tests.rs` and the shapes' in `src/forge/tests.rs`.
  Each fixture's snapshot gained an `edit:` line.

## Negative results

- **A class substring match** (`class~=edit`) was not built:
  `generic/edit-candidates` has an `a.edit` "Improve this page" before the
  `rel=edit` winner, and a substring would give it a tier the spec does not.
- **Scanning the forge marker from the right** was not taken: a file path may
  hold an `edit` or `blob` directory, and a repo is less likely to be named
  for a marker than a path is to hold one.
