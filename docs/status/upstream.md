# Upstream fixes we are waiting on

A defect or limit whose fix belongs to a dependency (a crate, Antora, the sites
we check), what we do until the fix ships, and what to change once it does.
What an entry holds, when one is added and when it is struck are
[`.claude/skills/upstream-issue/SKILL.md`](../../.claude/skills/upstream-issue/SKILL.md),
and every dependency upgrade reads this file. Each site to revisit carries an
`upstream: UF<k>` comment.

`cd scripts && uv run upstream.py` resolves every entry to its markers and
back. Numbers up to the watermark are allocated, and none is reused.
<!-- upstream-watermark: UF1 -->

## UF1 — duckdb-rs binds and appends no list or struct

- **The issue.** In duckdb 1.10506.1 neither `Appender::append_row` nor a bound
  statement parameter takes a `Value::List` or `Value::Struct`; both return
  "not yet supported". `targets.redirect_chain` is a list of structs (D43's
  schema 2), so the writer cannot append a target with a chain.
- **Upstream.** [duckdb-rs#885](https://github.com/duckdb/duckdb-rs/pull/885),
  "Implement bind owned List, Array, Struct, and Map values", merged to `main`
  on 2026-10-06 (commit 3a3a950) with appender tests; read 2026-10-18, no release
  named. The crate's latest release, 1.10506.1 (2026-10-16), still carries the
  error in its tests. Older: [#680](https://github.com/duckdb/duckdb-rs/issues/680)
  (container binding, closed) and [#422](https://github.com/duckdb/duckdb-rs/issues/422)
  (appender arrays, closed 2025-07-08 pointing at the `appender-arrow` feature).
- **Fixed when.** A duckdb release containing 3a3a950: `Appender::append_row`
  accepts an owned `Value::List` of `Value::Struct`.
- **Workaround.** A target with a redirect chain is inserted by a cached
  statement, one per chain length, with the chain as a list literal of struct
  literals whose fields are bound scalars (`store/writer.rs`); every other
  target goes through the appender with a NULL chain. `runs.roots` is a list
  too, bound the same way inside a list literal (`Store::begin_run`). Rejected: `append_record_batch`
  (the `appender-arrow` feature), an Arrow dependency for one column; a `hops`
  table, which departs from the spec's list and from every report reading
  `len(redirect_chain)`; a chain rendered to text and cast, which needs
  escaping of a URL.
- **Watch.** `store::tests::the_appender_still_takes_no_list` asserts the
  error and fails at the release that lifts it.
- **When it lands.** Delete `Tables::insert_redirected` and send a chain
  through the appender as a `Value::List` of `Value::Struct`, and bind
  `runs.roots` as a `Value::List`; invert the watch
  test into one appending a chain; the `a_targets_chain_keeps_each_hops_url_status_and_kind`
  test stays as it is.
