# P7.5 — The severity map, and the policy stated: notes

What the next slice inherits. The decision is D82. The row landed without the
gate's `fail` list, which is P7.5.1 (below).

## What landed

- **`[severity]`**: `config::policy::severity` holds `Severity` (`none`,
  `pedantic`, `attention`, `broken`, ordered) and `SeverityMap`, the default
  map with a file's laid over it, recorded whole as `severity.<key>` rows.
  `SeverityMap::of_link(outcome, reason)` is the one reading: the reason's
  level, else the outcome's, else none; `of_redirect` is `redirected` through
  it. `SEVERITY_REASONS` is the list of reasons a key may name, and
  `crawl::outcome::tests::every_reason_the_crawl_writes_takes_a_severity`
  holds it to what the crawl writes.
- **`Digest.policy` and `Digest.severity`**: the run's rows as recorded, and the
  map read off them (`sections::policy`). `report::digest::html::severity`
  (`Level` is gone, `Severity` is the one type) draws the dots, the figures'
  `bad`/`warn` classes and the legend, each level with the kinds at it.
- **The policy stated**: the JSON's `policy` (every row, `host` null for a
  run-wide one), Housekeeping's "Settings off their defaults"
  (`html/policy.rs`; a row not among `Policy::default().entries()`, written by
  `Entry::written_as` as the file writes it), and the pedantic guide entry
  naming how such a redirect is marked. The manual's two pages carry them.

## For P7.5.1 and P7.6

- **The `fail` list is one more key.** `Policy::entries()` writes a `gate.fail`
  row per outcome; `Entry::written_as` needs its arm (an unknown key prints as
  `default.<key>` today), and Housekeeping lists it with no other change. The
  gate reads the CI run's rows (D81: the guard already names that run).
- **A trial policy (P7.6) must carry the map.** `Digest.severity` comes from
  `policy` rows, so `--config` hands the digest rows from a file's
  `Policy::entries()` and nothing else changes on the report's side.
- **`none` keeps the rows.** A kind at `none` loses its dot, and stays in every
  list, count and ranking: an outcome-ranked SQL order (`pages`, `by_repo`)
  cannot move with a level without a second ordering in the JSON too (D82's
  Rejected).

## Negative results

- Nothing measured: the digest's queries are unchanged and the one added reads
  the policy table's rows for the run.
- A reason key applies to any link carrying the reason whatever its outcome, so
  `too_large = "attention"` dots an `ok` link; no digest list shows such a
  link today.
