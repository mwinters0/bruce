# P7.3.1 — The doubled slash's reach: notes

What the next slice inherits. The decision is D80.

## What landed

- **`judge.doubled_slash_reach`** (`full`, `immediate`, `off`): `config::Reach`,
  `Policy::doubled_slash_reach`, a run-wide `policy` row of the same name
  (`config/policy`, `docs/manual/config.md`, "Judging"); no schema change.
- **The digest's `through` reads the run's reach**: `sections::page_counts`
  fills `PAGE_COUNTS`'s `$REACH`, so `immediate` is the recursion's first step
  and `off` not even that. `Digest.judged.reach` carries it to the guide, whose
  doubled-slash entry states what is counted on the linking page.

## For P7.4 and P7.5

- **`counted` did not change.** P7.3's notes expected the macro to carry the
  reach; it does not need to. A twinless doubled page's broken relative links are
  already no page's own count (D71), so `off` is `through` zero and `counted`
  the same, which the test asserts (`totals.links` equal under all three).
- **The reach is read from each run's own policy** in `page_counts`, so the
  digest's since sections (`novelty`) count a compared run's `through` by that
  run's reach until P7.4 names one policy. The `Judged.reach` the guide states
  is the digest's run's.
- **P7.5 states the policy in the Housekeeping and JSON**, the reach among it;
  it is read now in `sections::recorded_reach` and `judged`.

## Negative results

- Evidence on the 2026-10-24 crawl's counts (554 `full`, 183 `immediate`) was
  not re-measured: the figures are the spec's, and the test pins the three
  reaches on a small store. Run `bruce report digest` on a DB whose `policy`
  rows are edited by hand to see them.
