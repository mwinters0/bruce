# P5.2 notes: schema 4, and the fresh en-US crawl

Outcome: `targets.answered_by` is `local`, `baseline` or `live` (D51),
`SCHEMA_VERSION` is 4, and every row a crawl writes is `live`. The en-US crawl
for P5.11 runs detached under `runs/p5.2-en-us-20261020/`; its `HANDOFF.md`
names the check, the stop and what to do with the result.

## What the next slices inherit

- **The column has no default.** A path that writes or copies a target says
  which route answered it: P5.4's copy of a baseline's rows into a CI run
  writes `baseline`, P5.5's local fetcher `local`. A test inserting a row by
  SQL names `answered_by` too.
- **`AnsweredBy` is `store`'s**, exported beside `Outcome`, and the writer
  binds it last in both of its paths (the appender and the redirect
  statement), so a column added after it moves both.
- **P4.10's DBs are schema 3** and `bruce` refuses them; the baseline for
  P5.3's gzip measurement and P5.11 is this crawl's, not those.

## Negative results

- None. The change touched no view: `link_verdicts` and the `report_*`
  views never read the column, and their tests passed unchanged apart from
  the raw `INSERT INTO targets` statements now naming it.
