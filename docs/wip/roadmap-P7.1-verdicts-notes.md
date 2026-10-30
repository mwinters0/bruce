# P7.1 — Verdicts judged through a policy, defaults only: notes

What the next slice inherits. The decision is D77.

## What landed

`link_verdicts` is `SELECT * FROM link_verdicts_with()`, the table macro holding
the verdict SQL, in `store/views.rs`. The macro has one parameter,
`pedantic_trailing_slash := true`; every other constant stays a literal in its
body. `views::install` creates the `doubled_slash` view, the macro, then the view
calling it, a macro binding what it names when it is created.

## Evidence

On a copy of the 2026-10-25 crawl's DB (`runs/en-us-20261025-1301/en-us.duckdb`,
runs 1 and 2), `runs/p7.1-verdicts/capture.sh` runs every report that run can
answer, as text and JSON, and the digest three times. The binary before and the
binary after wrote byte-identical files, the digest's HTML and JSON included
(`timings.txt` beside the script has the times). The digest, alternated three
times each way on a machine with nothing else running: 44.1 to 44.6 s of CPU
before, 44.0 to 44.4 after, 7.0 to 7.8 s of wall clock either way. The macro
costs nothing measurable.

`introduced` and `inverse` could not run: that DB has no `check-dir` run. Their
SQL is unchanged and reads the view; `report::gate::tests` and
`report::inverse::tests` cover them, and a digest compares runs 1 and 2 through
the same view.

## For P7.2 and P7.3

- **A scalar parameter fits a setting that is one value for the run**, as
  `[judge]`'s `pedantic_trailing_slash` is. `doubled_slash` and a status's outcome
  are per host, so they cannot be scalars: the macro will read the run's recorded
  policy (P7.2's table) rather than take them as parameters. P7.3 decides the
  shape; the one parameter here proves the mechanism and nothing more.
- **`doubled_slash` has no switch yet.** Its reason feeds rule 3 and `counted`
  through the `doubled_slash` view, so turning it off per host touches both, and
  the guide's entry with them.
- **Time**: the scratchpad's first digest timing on this machine doubled while
  an unrelated job loaded it (load average 45); compare CPU time (`%U`), not wall
  clock, and alternate the binaries.
