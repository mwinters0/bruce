# P8.4.5 — The pages' and repos' rows counted by severity: notes

What landed, for 8.6's Finding 9
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 9") and P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"What the document holds", 5 and 6): a page's and a repo's summary reads
"● 181 broken links, ● 2 missing fragments (183 issues total)" (`tally` in
`report/digest/html/severity.rs`), red `broken_url` plus `error`, amber
`broken_anchor`, a level with no links left out; `sections::pages` and
`grouped` rank by red, then amber; the manual and the golden file follow. No
`D<k>` entry: the counts and the ranking are the spec's.

## What 8.4.6 and the review inherit

- **The JSON's `pages` and `by_repo` gained rows and changed order, not
  shape**: a page holding only `error` links is now a row of `pages` (35 of
  8.14's 61 pages with an errored link), `broken_pages` counts it, and the
  ranking is `broken_url + error`, then `broken_anchor`. `format_version` stays
  1. `broken` still holds `broken_url` and `broken_anchor` rows alone; the HTML
  joins the page's `errors` rows into its drill-down, red first.
- **On 8.14's database** the first three pages are the code-of-conduct ones, at
  184, 182 and 181 issues, and a page with 3 red and 3 amber ranks below one
  with 4 red (`target/debug/bruce report digest` over a copy of
  `runs/p8.6-review/en-us-p8.14.duckdb`).
- **The summary's dots are the row's, not a heading's**: the sections' own
  `<summary>` still has none (`a_heading_carries_no_dot_and_each_dot_is_named`).

## Negative results

- **The figures still keep errors apart** ("Links that errored", the 828 broken
  links being URLs and fragments), so a summary's "broken links" counts one
  more kind than the figures' total does; the maintainer counted a row's
  errors in its red (Finding 9), and the figures' note says an errored link
  "does not make it broken". Left as it is, filed under STATUS's decisions.
- **Rejected: keeping error-only pages out of the section**, which would leave
  a repo's red count holding links no listed page shows.
