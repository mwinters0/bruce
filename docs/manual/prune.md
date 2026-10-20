# `bruce prune`

```sh
bruce prune (--keep N | --older-than AGE) [--db FILE]
```

Deletes old **runs** from a database, with every row they hold, so a
database that gains a run a day does not grow without end. A run is never
changed, only kept or deleted whole.

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `--keep N` | none | Keep the `N` latest finished runs and delete every older run, finished or not. `N` is at least 1. |
| `--older-than AGE` | none | Delete the runs started longer ago than `AGE`: a whole number and a unit, `s`, `m`, `h`, `d` or `w`, as in `90d` or `12h`. |
| `--db FILE` | `bruce.duckdb` | The database to prune. It must exist; `prune` never creates one. |

Give exactly one of `--keep` and `--older-than`; neither, or both, is a
command-line error.

## What is kept

- **The last attempt.** A run that never finished (a crawl stopped by
  Ctrl-C, or one that died) and is newer than the latest finished run is
  kept, whatever the rule says, so it stays there to inspect
  (`bruce report --run N`). Once a finished run passes it, it is deleted
  like any other.
- With `--keep N` and fewer than `N` finished runs, only the runs before the
  oldest finished one go; with no finished run, none does.
- `--older-than` is not protected beyond the last attempt: it deletes the
  latest finished run too if that started longer ago than `AGE`, which can
  leave no run to report on. The next crawl numbers its run after the
  highest number left, or from 1 if none is.

When `prune` deletes a run it checkpoints the database, so the file gives
back the space, and then prints the numbers of the runs it deleted; otherwise
it says there was none.

## Not beside a crawl

DuckDB lets one process write a file. `bruce prune` on a database a crawl
has open fails at once, and a database no process has open holds no crawl,
so an unfinished run `prune` finds is dead.

## Exit status

| Status | Meaning |
|---|---|
| 0 | The database was pruned, or held nothing to delete. |
| 1 | The database does not exist, could not be opened or written, or is of another schema version. |
| 2 | The command line was not understood. |
