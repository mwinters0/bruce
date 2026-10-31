# Storage and history

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. The DuckDB file: append-only runs, URL identity, the
schema version and its refusal, the verdict view computed at read time, where a target's answer came from, and a
baseline imported as a run. `store` holds the schema, the writer and the views; the reports' own views are `report`'s.

### D2 Runs are append-only: every row but `meta`'s carries a `run_id`, and only a run's `finished_at` is updated
This answers when a link broke and which upstream change did it, and allows run diffs. Old runs are pruned explicitly
(`bruce prune`). A stopped run is never resumed: its rows stay, `finished_at` NULL, and the next starts afresh.
Rejected: latest-state upsert, which cannot date a breakage; a persisted frontier to resume from, much machinery for a
job that fits in a night. Code: `store::schema`.

### D26 A URL's identity is a u64 hash of its canonical form, stable across runs
A link's page then joins across runs by id with no URL comparison; its target joins by URL text. `links` keeps one row
per occurrence, raw href included, so a report can point at the link in its source. An href resolving to no URL is a
target keyed by its raw text (`broken_url`, `invalid_url`), one per text per run. Rejected: deduplicated links with a
count, smaller but losing the raw href; for an unresolvable href, no link row (reported nowhere) or a crawl-time
`links.outcome` and no target. Code: `url::Canonical::id`, `store::schema`.

### D43 A DB records its schema version, and `bruce` refuses a DB of another version
`meta` holds `schema_version`; the refusal names both. A 0.x release migrates no DB: the release workflow names a schema
change in the release's body, comparing the tag's `SCHEMA_VERSION` with the latest published release's, and the user
starts a new DB, the old binary still reading the old one, so migrations wait for a release that promises them. Views
are outside the version (`Store::install`). Rejected: recreating on mismatch, destroying D2's history; migrations for no
user; a rewriting script, a migration in all but name; the schema line left to the notes (D89), which their drafter
forgets. Code: `store::schema`, `release.yml`'s "Write the release body". Evidence: `test_workflows.BodyStep`.

### D28 Fragment verdicts are computed in SQL after the crawl, not during it
The crawler holds no anchor state and need not order fetches around targets. The verdict is a view, `link_verdicts`,
computed at each read, so a stopped run needs no second path and no stored column can disagree with it; the reports and
the gate (D9) read the same join, but for the reads KD23 names. One writer owns the store, DuckDB allowing one writing
process per file (`store::writer`). Rejected: checking fragments in memory as pages arrive; a table written at a run's
end. Reopens: a read slow enough to materialise (0.2 s warm over 587,839 links, run 1 of
2026-10-19's combined en-US database, no longer kept). Code: `store::views`. Evidence:
`store::views::tests`.

### D51 A target records where its answer came from, in a column the writer fills and no view reads
`targets.answered_by` is `local`, `baseline` or `live`, `NOT NULL` with no default, so every path writing a target must
say which. What the build answers, the `not_built` verdict included, is `local`; every other row a crawl writes is
`live`, `unchecked` included: the run decided it, as a copy does not. The verdict views ignore it, so the gate compares
like with like (D28); `report::inverse` alone reads it, `local` meaning built (D56). Rejected: a default of `live`,
which labels a forgotten copy as a request; a table of copies, giving `link_verdicts` a second path. Reopens: a report
wanting a run's answers split by route. Code: `store::AnsweredBy`. Evidence:
`store::tests::a_target_records_where_its_answer_came_from`.

### D52 A baseline file is imported as a run of its own, and a target is copied between runs row by row
`import_baseline` copies the file's one run in under the next number, or reuses the run whose `started_at` and
`config_hash` match: the gate is then a two-run query in one database, as `diff` is. `copy_target` copies a target, its
landing and their pages and anchors as `baseline`, leaving `page_facts` behind so a copied page is never one the build
holds (D22). Rejected: attaching the file, the gate reading across two databases and `baseline_run` naming a run in a
file gone after the job. Reopens: a reuse that misses on a hand-edited file. Code: `store::baseline`. Evidence:
`store::baseline::tests::a_baseline_already_imported_is_reused_and_copies_nothing`.
