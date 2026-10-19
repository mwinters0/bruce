# P4.6 `bruce report`: notes

`bruce report broken|errors|stats` is `cli::report` calling `report::run`.
The views are `report`'s (`src/report/views.rs`) and `cli::db::open` installs
them beside the store's and the profiles' at every open, `crawl`'s included;
D6 says why a view holds every run and the query binds one.

## What the next slice inherits

- **The views.** `report_pages` (a run's pages with `url`, `source_repo`,
  `source_path`, `root`, `crawled`, `in_sitemap`, `via_links`, `profile`),
  `report_broken` and `report_errors` (a row per link, `src_url_id` and the
  columns the CLI prints) and `report_stats` (counts per page and kind; a page
  with no links has no row, so `stats` joins it from `report_pages`). 4.7's
  reports add views in `views.rs` and a `Report` variant, a query in `query.rs`
  and a name in `Report::ALL`; the filters, the run and `--top` come with
  `Sql::scope`, `Sql::kinds` and `Sql::limit`.
- **A filter names the linking page.** `--repo` and `--fact` filter on the
  page a link is on, as `Cols` says. `inbound`, `orphans` and the rest filter
  on the page a row is about, which is the same `Cols` with a different page
  column; `--kind` is a link's, and for a page report it is the kind of the
  links counted.
- **`crawled` is `pages.root IS NOT NULL`.** `stats` lists only crawled pages;
  allowlisted pages, fetched for their anchors, have no links of their own and
  would fill the table with zeros (P4.4's DB: 4108 of 6425). `unlisted` and
  `orphans` want the same line.
- **`--repo` is the value `pages.source_repo` stores**, a URL
  (`https://forge.fedoraproject.org/atomic/docs`), not `group/repo`.
- **`Cell` and `Table`** carry the result as typed cells, so JSON keeps
  numbers and booleans; a type `Cell::from` does not name prints as its debug
  form rather than failing. `NULL` is `-` in text, `null` in JSON.
- **`stats`' totals are text only.** JSON is the page rows alone, so the
  unchecked-by-reason counts are not machine-readable; `report_stats` and
  `link_verdicts` hold them for `duckdb`. A report that wants totals in JSON
  needs an output shape other than one array of objects (D6, the spec's
  "Filters and output").
- **`cli` has a `db` child** (`DEFAULT_DB`, `path`, `open`): the module graph
  forbids a child calling its parent, and both subcommands open the database.
  `report` shares `request` (the types) and `error` as leaf children for the
  same reason: the parent calls `run` and `query`, so neither may name it.

## On P4.4's DB (a copy of `runs/p4.4-en-us-20261019-1633/en-us.duckdb`)

`broken` (anchor links) is 208 `broken_url` and 610 `broken_anchor`, 0.6 s
wall including opening; `stats` 1.0 s, 2317 pages, 552256 links, unchecked
184931 (157623 `out_of_scope`, 25039 `host_not_allowlisted`, 1955 `skipped`,
280 `scheme`, 34 `robots`). `errors` has no row. `--repo
fedora/docs/fedora-docs-ui` matched nothing, the stored value being the URL.

## Negative results

- `ORDER BY` on aliases that are also the joined view's columns (`links`,
  `error`) is ambiguous in DuckDB, so `stats` sorts an outer select.
- A page report cannot take its filter on the view alone: `report_stats` has
  no row for a page with no links, so the `kind` filter sits in the join's
  `ON`, not the `WHERE`.
- `serde_json` is not a dependency; JSON is written by hand (`table.rs`),
  which escapes the quote, the backslash and control characters and passes
  the rest through.
- No `prune`, no `diff`: 4.8 and 4.9.
