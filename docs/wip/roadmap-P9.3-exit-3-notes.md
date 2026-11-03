# P9.3 — Exit status 3: notes

What 9.4 to 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "Schema changes and
retention". The decision is D102.

## What landed

- **`store::schema::Mismatch`** is the refusal, `store::Error::SchemaVersion`
  holding it; the version comparison is its `check`, so a `--db` and an
  imported baseline are refused by one function. The message is unchanged.
- **`cli::failure::Failure`** is what the five subcommands return in place of a
  `String`: the message and whether the cause was a `Mismatch`. A `String`
  converts into it, so the `?`s over configuration and file errors are as they
  were. `Failure::at` takes a store error, `Failure::report` a report's, whose
  `Baseline` variant holds the `--since` import's. `run` prints and exits
  `failure.status(its_own)`, so `check-dir`'s 2 and the others' 1 stay for every
  other cause.
- **The status** is 3 for `crawl`, `check-dir` (a baseline of another schema
  included), `report` and `report digest` (`--since` included), `prune` and
  `export-baseline`. The manuals' exit tables name it;
  `cli::tests::every_command_exits_3_on_a_database_of_another_schema_version`
  and `a_baseline_of_another_schema_version_exits_3_whoever_reads_it` hold it.
- **`scripts/review.py`** tells a database of another schema by the status, not
  by scraping stderr for "schema version".

## What the next slices inherit

- **9.4's script tests `$? -eq 3`** after `crawl` (S1) and after the digest (S3),
  and `export-baseline` (S2) and `prune` (S5) meet 3 only on a DB S1 did not
  replace. A DB-less nightly never meets it.
- **Only a refusal at open or at a baseline's import is 3.** A DB whose `meta`
  has no `schema_version` row, or none at all, is a failure of 1 (or 2): it is
  not a bruce database of another version.
- **`crawl`'s message does not say what the script does** (move aside, start
  fresh): the guide does.

## Negative results

- The typed error did not need to cross `report`: `report::Error::Baseline`
  already holds the `store::Error`, so `cli` matches it and no variant was added.
