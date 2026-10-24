# P8.5 — An unresolved xref's hint: notes

What landed: `profile/antora/hints.rs`'s view `antora_hints`, `report_broken`'s
`hint` column, `report broken`'s and the digest's `broken` rows carrying it, a
Hint column in a digest page's drill-down when one of its links has one, the
manual (`report.md`, `digest.md`). D65 holds the measure, the floor and why.

## Measured

`runs/p8.1-en-us-20261023-2004/en-us.duckdb`, run 1, `bruce report broken`:
92 `unresolved_xref` links, **47 with a hint, 34 of them the page meant**; 45
with none. The reading is the agent's, by eye, link by link: 13 hints name a
page that is not the one meant (`community-blog.adoc` to `community-polls`,
`getting-help.adoc` to `getting-started`, `release-notes::index.adoc` to
`releases::index.adoc`, `legal/trademarks.adoc` to `marketing:ready:framework`,
the pages under `f/` to the first release directory that sorts), and 8.6's
review is the first by anyone else. 20 of the 45 without are the
`install-guide` and `system-administrators-guide` of ten release versions
(`f27@fedora:…` to `f36@fedora:…`), pages the run does not hold. `report broken` takes 0.17 s on the 32 MB database in a
release build, the hints included.

## What the next slices inherit

- **`hint` is a column of `report_broken`**, so `diff`, the gate's
  `introduced` and the digest's `broken_targets` read the view with it joined
  and do not select it. 8.7's and 8.8's tables that want it select `b.hint`.
- **The report tests' registry has the `antora` profile**
  (`report::tests::registry`), the view `report_broken` joins being its:
  a `report_*` view installed over a registry without it fails to create.
- **`xrefs` in `report/tests.rs` is a store of unresolved xrefs** over four
  pages with coordinates, for a test wanting a hint.

## Negative results

- **Jaro-Winkler weighs the shared prefix**, so it hints a page of the
  right component and the wrong name (`Langpacks` for `WhatCanBePackaged`);
  Levenshtein over the longer ID picked the typo's page in the cases where the
  two differed (D65).
- **Comparing the refspec as written finds another component's page**: 26 of the 92
  name no component or module, and the nearest ID to `WhatCanBePackaged.adoc` is then
  some other component's `index.adoc`. The completion by the linking page's
  component and module is what makes those findable.
- **Ties between release directories are arbitrary**: `f/blocking.adoc`
  hints `releases::f34/blocking.adoc`, every `fNN` being as near. The ID that
  sorts first is a deterministic choice, not a better one.
- **A hint is not a resolution**: the view is not read by Resolve, and a hint
  does not change a verdict or the gate (D20).
