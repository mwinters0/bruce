# P7.3.2 — A remapped status keeps the crawl's reason: notes

What the next slice inherits. The decision is D79 (amended).

## What landed

- **`link_verdicts_with`'s `answer`** gives a remapped status the stored
  reason when the outcome the map gives equals the stored outcome, else none
  (`store::views`). A `{ 404 = "broken_url" }` therefore erases no
  `wrong_trailing_slash`; a 4xx mapped to `error`, or a 403 to `ok`, drops it.
  The test is the "kept" row of
  `a_status_the_policy_judges_is_read_over_the_outcome_the_crawl_stored`.
- The manual's `status` paragraph (`docs/manual/config.md`) says so.

## For P7.4 and P7.5

- Nothing else reads the reason before the macro: the digest's queries that read
  `targets.outcome` and `targets.reason` directly (P7.3's notes) are unchanged,
  so a status remapped at read time still does not reach them.
