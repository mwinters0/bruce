# P7.3 — The judging keys: notes

What the next slice inherits. The decisions are D77 (amended), D78 (amended) and
D79; the DuckDB defect met on the way is UF2. The row landed without
`doubled_slash_reach`, which is P7.3.1 (below).

## What landed

- **The keys**: `[judge]` `pedantic_trailing_slash` and `schemes`;
  `doubled_slash` and `status` in `[default]` and in `[host."<h>"]`
  (`config/policy`, `docs/manual/config.md`, "Judging"). `Policy` holds every
  setting whole, `Policy::entries()` its rows, `begin_run` writes them with the
  host column D78 reserved; no schema change.
- **`link_verdicts_with(policy_run := NULL)`** reads those rows. The view is its
  call with each link's own run; `policy_run` is how P7.4 will name the newer
  run. A setting with no row is at its default, so a run with no rows is judged
  as P7.2's was.
- **The crawl follows `judge.schemes`** when it writes a `scheme` or
  `bad_scheme` target; `WORKING_SCHEMES` moved to `config::policy`. The status
  map is read time only: the crawl keeps D16's class in the row.
- **The guide follows the policy**: `Digest.judged`, read off the run's rows,
  drops an entry whose reason no host judges, and the legend's and the redirects
  list's ⓘ to the pedantic entry go with it. The doubled-slash entry states
  `doubled_slash_reach` from P7.3.1.

## Evidence

On a copy of the 2026-10-28 crawl's DB made schema 6 by hand
(`runs/p7.3-verdicts/migrate.sh`), `capture.sh` ran every report and the digest
for P7.2's binary and this one, with the `policy` rows P7.2 wrote and with the
whole default policy this slice writes: every output byte-identical, the
digest's HTML and JSON included. The cost (D77): one run's verdicts 0.31 to 0.33
s; the digest 59 to 67 s of CPU before, 65 to 69 after with the machine
loaded (`timings.txt`), its peak memory 2.3 to 2.9 GB. A restructure (verdict
columns on the target rather than a list indexed per link, one join for the
pedantic setting) changed nothing measurable and was not kept.

## For P7.3.1 and P7.4

- **P7.3.1 touches the digest's `through` alone** (`report/digest/sections.rs`,
  `PAGE_COUNTS`, reach `immediate` stopping at the landing page, `off` at none),
  not `counted` (D80). The `doubled_slash` view is policy-free and carries `url`;
  the macro filters it (`dsl`), so `PAGE_COUNTS` and `through` read all doubling
  pages: they agree only because a policy-off host has no `doubled_slash` link
  to start from.
- **Comparisons mix policies until P7.4.** `cause.rs` and the digest's since
  sections judge each run by its own policy; a ⓘ for a reason is written wherever
  a link carries it (`info_of`), so a compared run judged by another policy can
  carry an ⓘ the guide has dropped. P7.4's one policy removes it. Some digest
  queries read `targets.outcome` itself (`cause.rs` `target_removed`, the
  external hosts, the sitemap entries); a status remapped at read time does not
  reach those.
- **`policy_run` is `NULL` for a view and a number for P7.4's call**; the
  baseline run an `import_baseline` renumbers has its rows, as D78 says.
- **A status mapped to a different outcome loses the stored reason**
  (P7.3.2 keeps it where the outcome is the stored one).

## Negative results

- A macro parameter in a join condition aborts a DuckDB with assertions
  (UF2); the first version of the macro did, and every test died. The release
  CLI runs it.
- Judging the crawl's own classification at crawl time (the policy passed to
  `outcome::answered`) was not tried: D79 keeps the row what the crawl saw so a
  trial policy (P7.6) and a comparison need no crawl.
