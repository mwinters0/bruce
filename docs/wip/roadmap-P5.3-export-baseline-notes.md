# P5.3 notes: `export-baseline`

Outcome: `bruce export-baseline --out f [--run N] [--db f] [--force]` writes a
finished run into a new database of the current schema (`Store::export_baseline`,
`src/store/baseline.rs`; the command, `src/cli/export_baseline.rs`), and prints
its size and its gzipped size. The manual page is
[`../manual/export-baseline.md`](../manual/export-baseline.md).

## What the next slices inherit

- **The file keeps the run's number**, so P5.4's import into a CI database
  has a `run_id` to renumber (`begin_run` allocates the next), and matches a
  baseline already imported on the run's `started_at` and `config_hash`, which
  the file keeps verbatim. `runs.baseline_run` is `NULL` in the file: it named
  a run the file lacks.
- **Six tables are copied**: `runs`, `targets`, `pages`, `page_facts`,
  `anchors`, `links`. `bodies` are not (D15). The copy is `INSERT ... SELECT`
  across an `ATTACH` in one transaction, so P5.4's copy between runs in one
  database can share the table list and the statement's shape
  (`store::baseline::TABLES`).
- **The file holds the store's and the profiles' views, not the reports'**:
  `Store::open` installs those, and `cli::db::open` adds the reports' at every
  open, so `check-dir` and `report` get them on attach with nothing to export.
- **`--force` renames over `--out`** from `<out>.partial`, so a refused run
  leaves the file it would have replaced.
- **The measurement** is under "Size", below; the P6 inbox carries the
  recommendation.

## Size

P5.2's en-US run (`runs/p5.2-en-us-20261020/en-us.duckdb`, 33042432 bytes
holding the same run) exports to `runs/p5.3-baseline/baseline.duckdb`:
172452 targets, 6435 pages, 93531 anchors, 588552 links, 27013120 bytes
(25.8 MiB); gzipped 6137905 bytes with `flate2`'s default level, 6103638 with
`gzip`. The export took 2.3 s and 207 MB resident. Reproduce with
`target/release/bruce export-baseline --db runs/p5.2-en-us-20261020/en-us.duckdb
--out <file>`. `bruce report stats` and `broken` read the file and give the
source's rows (828 lines of `broken` from each).

## Negative results

- **Deleting `--out` before exporting** loses it when the run is refused,
  the choice of run happening inside the export: the file is written as
  `<out>.partial` and renamed.
- **Anything about trimming**: the gzip size, 5.9 MiB, leaves no reason to
  drop the chrome nav links from the copy, the later change the spec holds
  open.
