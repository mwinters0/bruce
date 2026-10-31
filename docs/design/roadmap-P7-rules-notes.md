# P7 — Site policy in config: notes

P7 delivered site policy without Rust. A run records the policy it was judged
by (schema 6), and every report judges it at read time through one table
macro. A comparison judges both runs by the newer run's policy, and `report
--config` judges by a trial file. The judging keys are a status's outcome
per host, `doubled_slash` and its reach, `pedantic_trailing_slash`, the
schemes, the severity map and the gate's `fail` list. The asking keys are
the timeouts, the retries, the `Retry-After` ceiling, D72's probe and the
body cap. Antora left the core's output: `[digest]` `group_by`, the profile's
sitemap note and the hints through Views. The decisions are D77 to D87, with
D5, D9, D16, D24, D38, D64, D66, D70 to D73 and D76 amended. The keys are `../manual/config.md`'s, and what a reader
sees is `../manual/digest.md`'s and `../manual/check-dir.md`'s. This keeps
what P9 inherits, the evidence, and what was tried and refused; P11's fact
is in its inbox, `roadmap-P11-prose-inbox.md`.

## The evidence

| Where | What it holds |
|---|---|
| `runs/en-us-20261028-1452/en-us.duckdb` | the 2026-10-28 en-US crawl, schema 5: D77's cost and D79's hosts were read off it |
| `runs/p7.1-verdicts/` | `capture.sh` over a copy of the 2026-10-25 crawl's DB: every report and the digest byte-identical before and after the macro, `timings.txt` |
| `runs/p7.3-verdicts/` | the 2026-10-28 crawl's DB made schema 6 by hand (`migrate.sh`), the same capture for P7.2's binary and P7.3's, `timings.txt` |

- **Time a digest by CPU, not wall clock, and alternate the binaries.** A
  digest's first timing doubled while an unrelated job loaded the machine
  (load average 45). The macro's cost is D77's Why.
- **Only 7.1 and 7.3 were measured on a real crawl.** Every later slice's
  evidence is its tests: the 2026-10-28 crawl's DB is schema 5 and no longer
  opens, and no crawl has run under schema 6. The doubled slash's reach on the
  2026-10-24 crawl (554 pages `full`, 183 `immediate`) is the spec's figure
  and was not measured again. Hand-edited `policy` rows reproduce it on any
  schema-6 DB.
- `introduced` and `inverse` never ran on a real DB in P7, there being no
  `check-dir` run in one; `mise run e2e` and `report::gate::tests` cover them.

## What P9 inherits

- **`[rate]` is `[default]`**, and the loader refuses `[rate]` by name. The
  guide's config section is rewritten under the new name.
- **The guide sets `[digest]` `group_by = "antora.component"` for Fedora**
  (P9's spec, "P9 follows P7"). Without it the digest's JSON has no groups,
  where it listed components before.
- **The guide can set a host's own timeouts and `slash_probe`.** `../manual/config.md`,
  "Asking", is the reference.
- **A run recorded before 7.7 has no asking rows**, and its digest states
  none. A missing row reads as the default, which is what such a run asked
  with.
- **The first schema-6 crawl is the phase's first real evidence** for 7.4
  onwards: a digest compared across a policy change, a trial `--config`, and
  a `none` kind's effect on the ranking.

## What was tried and refused

- **A macro parameter in a join condition aborts a DuckDB with assertions**
  (UF2). The first macro did, and every test died; the release CLI runs it.
- **A restructure of the macro** (verdict columns on the target rather than
  a list indexed per link, one join for the pedantic setting) changed nothing
  measurable on the 2026-10-28 crawl and was not kept.
- **Shadowing `policy` itself with the trial's table** counted the trial's
  rows as the stored ones, `main.policy` finding the temporary table too.
  `policy_in_force` is the fix (D84's Rejected).
- **A count by level alone** ("5 red") was not built. It loses which kind a
  level holds. M25 is to name the reasons in a split count instead.
- **No guard on `changed` or `anchors-removed`.** They read no verdict of the
  older run (a content hash; its anchors).

## Limits left in place

- **A reason key applies to any link carrying the reason, whatever its
  outcome.** `too_large = "attention"` dots an `ok` link. No digest list
  shows such a link today.
- **The `new` marks on a page, a repo and a repo's `new_links` come from SQL
  over all findings.** A page whose only earlier finding was at `none`
  therefore reads as not new. The links a summary counts as new are those
  above `none`.
- **"Introduced breakage not failing" is the baseline lacking the key at a
  left-out outcome.** A link the baseline had at a listed outcome and the run
  now has at a left-out one is listed there. A link broken at a left-out
  outcome in both runs is in no section (D83, Reopens).
- **Nothing enforces that a judging reader reads `policy_in_force`.** A grep
  for `FROM policy` in `src`, outside tests and `store`, finds a reader that
  bypasses the trial (D84, Reopens). Asking rows are read from `policy` on
  purpose (D85).
- **A trial reads the file's whole `Config`**, its roots validated as any
  command reads them, and uses every policy row but the asking ones (D85).
- **`retries` is capped at 10 at load** (a 512 s wait before the last). The
  cap was a choice, not a measurement.
- **Each client built for a host's own connect timeout has its own hickory
  resolver**, and so its own DNS cache. A host is one name, so nothing is
  looked up twice. This was not measured.
- **The sitemap note is per profile, not per root.** A run crawling Antora
  and generic roots prints Antora's sentence once, above all the run's
  entries.
- Not tested end to end: `cli::check_dir` passing the body cap to
  `LocalFetcher` is one line, and `LocalFetcher::with_max_body` has its own
  tests.
