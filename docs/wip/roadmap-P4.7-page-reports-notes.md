# P4.7 The other single-run reports: notes

`inbound`, `orphans`, `sitemap` and `unlisted` are `report/reach.rs`,
`redirects` is `report/redirects.rs`, and the statement builder both share with
`query.rs` moved to `report/sql.rs`. Four views joined `report_*`:
`report_inbound`, `report_roots`, `report_sitemap` and `report_redirects`
(`views.rs`). The code is not run on a real DB: P4.4's is not in the tree
(`runs/p4.4-en-us-20261019-1633/` holds only its log), so the tests build the
shapes by hand.

## What the next slice inherits

- **`diff` and `prune` need nothing from here.** `Sql`, `Cols` and `PAGE` are
  `report/sql.rs`'s; 4.8's two-run query binds both runs through `Sql::bind`.
  `Request` has one `run`, so `diff` adds `--from`/`--to` beside it.
- **A page report filters on the page a row lists** (`PAGE`), a link report on
  the page the link is on; `redirects` is a link report grouped.
- **`report_inbound` resolves a link to the page it landed on**, through
  `targets.final_url`, the way `link_verdicts` finds anchors. Without it a page
  reached only by a redirect would be an orphan.
- **`Cell::List`** carries an array: JSON array, joined by commas in text.
  `Request.linking_pages` is set by `cli` for `--format json`; it is the only
  field of a request that depends on the format.
- **A root's sitemap is read off the root's own `sitemap_file` link** (D47),
  matched by the root's string against `targets.url`. Roots here are plain
  URLs; one with an escape that canonicalisation rewrites would match nothing
  and read as having none.
- **A skipped sitemap entry is in no report**: `write_sitemap` drops an entry
  whose plan is not a page without writing a link (`src/crawl.rs`), `skipped`
  included, so only robots-blocked, failed and invalid entries show in
  `sitemap`.

## Negative results

- **`redirects` shows one chain per landing URL**, the first of the URLs by
  name. Several origins to one landing share the shape on the forge; a group
  of different chains would need a row per chain, which the spec's grouping
  by landing rules out.
- **The sitemap report cannot take `--repo` or `--fact` on an unfetched entry**:
  it has no page, so it drops out.
- **No timing on the real DB**, for the reason above; `link_verdicts` over
  P4.4's DB took 0.1 s and every new view is a join or a count over it.
