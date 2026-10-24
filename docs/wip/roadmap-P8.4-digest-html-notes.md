# P8.4 — The digest's HTML: notes

What landed, per the P8 spec's "How it looks, and what proves it" and "What
the document holds": `Digest::html` (`report/digest/html.rs`, its stylesheet
`html/style.css` inlined by `include_str!`), `bruce report digest --format
json|html|both` writing `DIR/STEM-YYYY-MM-DD.html` beside the `.json`, a test
that the document holds no `<script>`, `<link>`, `src=` or `url(`, and the
golden file `fixtures/antora/digest.html`, which `mise run e2e` compares with
the digest of its crawl. D64 says why.

## What the sections are

The header, the figures, the pages as `<details>` ranked as `pages` is, the
repos that hold breakage (each a `<details>` of in-page links to its pages),
"Fix once, fix many" (the targets with `links > 1`, and the count of the
rest), the errors, and the housekeeping, each after the figures closed
([`roadmap-P8.4.3-digest-sections-notes.md`](roadmap-P8.4.3-digest-sections-notes.md)).
There are no section numbers: 3 and 7
are the two-run sections 8.7 and 8.8 add, and a number skipped reads as a
fault. The header says `Compared with: no earlier run`, the figures carry no
change; 8.8 fills both.

## What 8.5 to 8.8 inherit

- **A section is a `fn` over `&Digest` writing into the `String`**, its rows
  read through `Row`, which finds a cell by column name and panics on a name
  the table lacks, so a renamed column fails every HTML test. 8.5's `hint`
  is one more column in "Problem" or the targets' table; 8.7's `anchors-removed`
  and 8.8's "since the last run" are two more sections beside `repos`.
- **`by_repo`'s rows with no broken page are not listed**, the section saying
  how many repos there are beside the ones it shows (on en-US, 21 of 62).
- **The golden file is the normalised HTML**: `e2e.normalise_digest` replaces
  the times, duration, date and versions, and the file is rewritten by
  `cd scripts && uv run e2e.py --save-digest`. A change to the renderer is a diff to
  read in that file, and a new section is one more.
- **The en-US digest** (`runs/p8.1-en-us-20261023-2004/en-us.duckdb`, run 1)
  renders to 661 KB of HTML beside 671 KB of JSON, in 6 s; 324 page
  drill-downs, 49 targets in "Fix once", holding 383 of the 828 links, the
  266-link anchor first.

## Negative results

- **`mise run e2e` fails at its gate before this slice and after it**: the
  planted build's `xref:ops::disks.adoc`, through the page-alias, is reported
  introduced as `unresolved_xref` and absent from `redirected`, where
  `scripts/e2e.py` (`PLANTED_INTRODUCED`, `PLANTED_REDIRECTED`) expects it
  resolved by D57. Checked with this slice stashed, at `c4caf99`. The digest
  step runs before the gate and passes. The cause is not sought:
  `docs/status/history/2026-10-23.md`, "e2e's gate", is the pickup.
- **Printing opens every `<details>` only where the browser supports
  `::details-content`** (Chromium 131, Firefox 143, Safari 18.4); an older one
  prints them closed, there being no CSS that opens a closed `<details>`
  otherwise and no script allowed. Not seen in a browser here: none is
  installed, so the look is read from the CSS and the structure from tests.
- **No Rust golden file**: the structure tests over the synthetic store assert
  the sections, the escaping and the links, and the e2e golden holds the
  whole page, so a second whole-page copy would be a second thing to bless.
- **A page's own markup is text**: only an `http` or `https` URL becomes a
  link, and everything else is escaped (`text_a_page_wrote_is_escaped_…`).
