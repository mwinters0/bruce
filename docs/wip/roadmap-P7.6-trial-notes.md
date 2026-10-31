# P7.6 — `bruce report --config`, judging by a trial policy: notes

What the next slice inherits. The decision is D84.

## What landed

- **`Store::try_policy(&Policy)`** (`store::trial`) creates a temporary table
  `policy_in_force` holding the policy's `entries()` once for each run of the
  database. `policy_in_force` is a view over `policy`, created with the other
  views, and every reader of the policy asks for it: the macro, the digest's
  `judged`, `policy` and `recorded_reach`, and the gate's `recorded_fail`. The
  temporary table shadows the view for the connection, and nothing is written.
- **`--config FILE`** on every report and on `digest`: `cli::report::open`
  loads the file with `Config::load`, calls `try_policy`, and says on stderr
  that it judges by a trial. `Digest.trial` carries the path, set by the
  caller; the HTML opens with a line naming it and Housekeeping states the
  file's settings, the JSON has `trial` (`null` otherwise) and `policy` is the
  file's rows.
- A comparison needs nothing more: `JudgedBy` names the newer run, and its
  rows in the temporary table are the trial's. A baseline file imported by
  `--since` has no temporary rows and is never the newer run.

## For P7.7 and P7.8

- **A new reader of the policy reads `policy_in_force`**, never `policy`, or a
  trial does not reach it. Writers (`begin_run`, the import, `prune`) keep the
  stored `policy`. Nothing enforces the first: a grep for `FROM policy` in
  `src` outside tests and `store` finds a reader that bypasses the view.
- P7.7's asking settings are recorded in `policy` and are not judging, so a
  trial ignores them as the spec says; they must not be read through
  `policy_in_force`, or a trial would answer for a crawl that asked otherwise.
- P7.8's `[digest]` `group_by` is the report's: a trial's file would set it, so
  its row is read through the same name.

## Negative results

- **Shadowing `policy` itself failed in a test**: a qualified `main.policy`,
  which the import and `delete_runs` write through, finds the temporary table
  as well, so the temporary table's rows were counted as the stored ones. The
  view is the fix; D84 records it.
- Not measured: a view over a table is inlined, so the macro's reads cost what
  they did (D77). The 2026-10-28 crawl's database is schema 5 and no longer
  opens, so the evidence is the tests alone.
- The trial reads the file's whole `Config` (roots validated, as any command
  reads it) and uses its `policy`; its roots and asking settings are not used.
