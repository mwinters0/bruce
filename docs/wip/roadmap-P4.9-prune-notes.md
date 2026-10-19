# P4.9 `prune`: notes

`bruce prune (--keep N | --older-than AGE) [--db f]` is `cli/prune` calling
`Store::prune(Rule)`, which lives in `store/prune.rs`. It selects the runs to
delete, deletes their number from every table in one transaction, `runs`
last, and runs `CHECKPOINT`. The manual is `docs/manual/prune.md`.

## What the next slice inherits

- **The rules as built.** `--keep N` deletes every run numbered below the
  `N`th latest finished run, so an unfinished run between finished ones goes
  and one after the latest finished run stays. `--older-than` deletes by
  `started_at`, with the same exception for the last attempt, and deletes the
  latest finished run too when it is old enough (the spec says "the runs
  started before the cutoff"). With no finished run both rules keep every
  unfinished run: nothing anchors "newer than the latest finished".
- **Judgement calls beyond the spec**, none binding on a design: `--keep`
  takes at least 1 (`--keep 0` would be deleting the file); `--keep` and
  `--older-than` are exclusive, as `historical/initial.md`'s "D27" draws them
  (`--keep N | --older-than DUR`); `DUR` is a whole number with `s`, `m`,
  `h`, `d` or `w`, no dependency being added for it and a fractional or
  compound age having no use here.
- **Run numbers.** `begin_run` numbers from `max(run_id) + 1`. Both rules
  delete a prefix of the numbers (the last attempt is newer than every
  finished run), so a number is not reused unless `--older-than` deletes
  every run, when the next crawl is run 1 again. `runs.baseline_run` of a
  kept run may then name a deleted run; nothing reads it yet (P6, baselines).

## Negative results

- The bundled DuckDB has no ICU, so a `TIMESTAMPTZ` takes no interval
  arithmetic (`now() - to_seconds(?)` fails to bind). The cutoff compares
  `epoch_us(started_at)` with `epoch_us(now()) - ?` instead; the tests set
  `started_at` through `rows::timestamp`.
- Run on a copy of P4.4's DB (one run, 31 MB): `--keep 1` deleted nothing in
  0.13 s, and `--older-than 0s` deleted run 1 in 0.19 s and left a 5 MB file.
  That is one run, so how `CHECKPOINT` fares on a database of many is
  unmeasured. A crawl or `duckdb` session holding the file makes `prune`
  fail at open with DuckDB's lock error, exit 1.
