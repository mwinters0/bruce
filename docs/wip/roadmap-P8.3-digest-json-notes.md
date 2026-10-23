# P8.3 — The digest's document and JSON: notes

What landed, per the P8 spec's "`bruce report digest`, named by the run's day"
and "What the document holds": `report::digest` builds a crawl run's
`Digest`, `Digest::json` writes it under `FORMAT_VERSION` 1, and `bruce
report digest --out DIR [--name STEM] [--run N]` writes
`DIR/STEM-YYYY-MM-DD.json`. `docs/manual/digest.md` documents the file, and a
test holds the JSON's fields to the manual's example. D64 says why.

## What 8.4 inherits

- **A `Digest` is `Table`s, already ranked**: `broken_targets`, `pages`,
  `broken` (a page's links together, the pages in `pages`' order), `by_repo`,
  `by_component`, `errors`, `redirects`, `orphans`, `unlisted` and `unchecked`,
  with `run`, `totals` and `roots_without_sitemap` beside them. The columns are
  on the struct. A renderer walks them and ranks nothing; it groups `broken`
  by consecutive `page`.
- **`Digest::date()` is the file name's date**, `utc` its formatter. The
  command's `--format` takes `text` or `json` for every report and `json`
  alone for the digest, `cli::report::check` refusing `text`; 8.4 adds the
  HTML's values there and in `cli::report::digest`, which writes the one file.
- **Every URL in the document is absolute**, as the database holds it; the
  forge's `source_url` is the file's view, null where the page records none.
- **A `source_*` column describes the row's subject**: the linking page in
  `pages`, the target page in `broken_targets`.
- **`by_repo` lists every repo, `repo` null for a page recording none**: on
  en-US 168 of 2,320 crawled pages, in 62 rows, 21 with no breakage. Section
  6 may want only the broken ones.

## What 8.7 and 8.8 inherit

- `compared_with` is always `null`, and the document has no section 3 or 7 and
  no change in its figures. Adding them adds fields, so `format_version` stays
  1; `Digest` gains the comparison and `json.rs` renders it.
- Every section's query is in `digest/sections.rs`, over `report_stats`,
  `report_broken`, `report_errors` and `report_pages` joined to the crawled
  pages (`pg.crawled`). A comparison reads the same views for `from`.

## Measured

`bruce report digest --db runs/p8.1-en-us-20261023-2004/en-us.duckdb --out
DIR --name en-us` takes 6 s in a debug build (its statements each read
`link_verdicts` afresh) and writes 671 KB, the spec's estimate being 300 KB: 494 broken targets and 61 redirect landings with their linking pages
are most of the difference. Its figures are P5.2's and 8.1's: 324 pages
holding 828 broken links (217 `broken_url`, 610 `broken_anchor` and one broken
image, which the digest counts for counting every kind), 2,320 pages, 586,175
links and 187,470 `unchecked`. The 266-link anchor ranks third in
`broken_targets`, below two targets named by 6 and 5 pages; 8.3.1 ranks by
links (`../status/history/2026-10-23.md`, "The digest's sections
reviewed").

## Negative results

- **A section is a `Table`, not a type of its own**: the JSON, the HTML and a
  test all want the column names, and `Table` already prints the D6 shape.
- **`broken` is ordered in Rust**, by `pages`' rank: ranking the links in the
  statement would repeat the page counts' query.
- **`serde_json` is a dev-dependency alone**, the test parsing what `quoted`
  and `Table` write; the writer is a few `format!`s, and a production
  dependency would join `THIRD-PARTY-NOTICES.txt` for them.
- **A sitemap's entries are in no section but the redirects**, which reuse
  `report redirects` over every kind: a link in a sitemap file is on no
  crawled page. En-US has none broken (`report broken --kind all`); 8.3.1
  lists them in the housekeeping.
- **`Mode::as_str` is public** now, the digest refusing a `local` run by the
  word the table holds.
