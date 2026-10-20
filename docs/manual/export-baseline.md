# `bruce export-baseline`

```sh
bruce export-baseline --out FILE [--run N] [--db FILE] [--force]
```

Writes one finished **run** of a database that `bruce crawl` wrote
([`crawl.md`](crawl.md)) as a database file of its own, the **baseline** a
content repo's CI checks its build against. A scheduled full crawl is the
usual run to export; where the file is published is the deployment's choice.

The file is an ordinary `bruce` database holding that one run, so
[`bruce report`](report.md) reads it as it reads any other. It carries every
table the run wrote but the saved page bodies, which a crawl keeps only on
request, and it records no earlier run as the one it was compared against.
The run keeps its number.

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `--out FILE` | required | The file to write. It must not exist unless `--force` is given. |
| `--run N` | the latest finished run | The run to export. It must have finished. |
| `--db FILE` | `bruce.duckdb` | The database to read. It must exist; `export-baseline` never creates one. |
| `--force` | off | Replace `--out` if it exists. The old file stays until the new one is whole, so a refused run leaves it. |

## What is refused

- **A run that never finished**, whether it was stopped by Ctrl-C or died: it
  holds only the targets it reached, and each one missing would become a live
  request in every CI job reading the file. Name a finished run, or crawl
  again.
- A database with no finished run, when `--run` names none.
- A `--out` that is the database itself.

## What it prints

One line on stdout: the run, the file, the counts of targets, pages, anchors
and links it holds, and the file's size in bytes and in MiB, then its size
gzipped, so a deployer can weigh publishing it compressed. A full en-US
crawl of docs.fedoraproject.org (172,452 targets, 6,435 pages, 588,552 links)
exports to about 25.8 MiB, 5.9 MiB gzipped.

## Not beside a crawl

DuckDB lets one process write a file. `bruce export-baseline` on a database a
crawl has open fails at once.

## Exit status

| Status | Meaning |
|---|---|
| 0 | The baseline was written. |
| 1 | The database does not exist, could not be opened or is of another schema version; the run is unfinished or missing; or `--out` exists without `--force`, is the database, or could not be written. |
| 2 | The command line was not understood. |
