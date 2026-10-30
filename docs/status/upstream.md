# Upstream fixes we are waiting on

A defect or limit whose fix belongs to a dependency (a crate, Antora, the sites
we check), what we do until the fix ships, and what to change once it does.
What an entry holds, when one is added and when it is struck are
[`.claude/skills/upstream-issue/SKILL.md`](../../.claude/skills/upstream-issue/SKILL.md),
and every dependency upgrade reads this file. Each site to revisit carries an
`upstream: UF<k>` comment.

`cd scripts && uv run upstream.py` resolves every entry to its markers and
back. Numbers up to the watermark are allocated, and none is reused.
<!-- upstream-watermark: UF2 -->

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
  target goes through the appender with a NULL chain. `runs.roots` and
  `runs.hash_scopes` are lists too, bound the same way inside list literals
  (`Store::begin_run`). Rejected: `append_record_batch`
  (the `appender-arrow` feature), an Arrow dependency for one column; a `hops`
  table, which departs from the spec's list and from `report_redirects`
  rendering the chain whole; a chain rendered to text and cast, which needs
  escaping of a URL.
- **Watch.** `store::tests::the_appender_still_takes_no_list` asserts the
  error and fails at the release that lifts it.
- **When it lands.** Delete `Tables::insert_redirected` and send a chain
  through the appender as a `Value::List` of `Value::Struct`, and bind
  `runs.roots` as a `Value::List`; invert the watch
  test into one appending a chain; the `a_targets_chain_keeps_each_hops_url_status_and_kind`
  test stays as it is.

## UF2 — DuckDB asserts on a macro parameter in a join condition

- **The issue.** In DuckDB 1.5.6 (duckdb 1.10506.1) built with assertions, a
  table macro whose parameter stands in a `LEFT JOIN`'s `ON` condition aborts the
  process when the macro is planned, `joinside.cpp:53`, `GetJoinSide`:
  `right_bindings.find(table_binding) != right_bindings.end()`. `cargo test`
  builds DuckDB that way; the release CLI plans the same query and answers
  correctly. A view over the macro is planned when it is created, so opening a
  store dies.
- **Upstream.** Nothing found: searched duckdb/duckdb on GitHub for the assertion's
  text, "table macro parameter left join" and "joinside" on 2026-10-30. The nearest
  are [#5435](https://github.com/duckdb/duckdb/issues/5435), a fuzzer report of
  the same assertion from an `UPDATE ... FROM` with a row-value `ON`, and
  [discussion #16012](https://github.com/duckdb/duckdb/discussions/16012), a
  correlated parameter in a non-inner join, answered by using inner joins only.
  No issue is opened (the maintainer's call).
- **Fixed when.** `store::views::tests::a_macro_parameter_in_a_join_condition_still_aborts_a_debug_duckdb`
  fails: a debug-built DuckDB plans the query below.
- **Workaround.** `link_verdicts_with`'s `policy_run` is read in one place, the
  `pr` CTE mapping each run to the run whose policy judges it, and every join to
  `policy` is on `pr.prun` (`store/views.rs`). Rejected: `policy_run` in each
  condition, the plainer SQL, which fails every test; a view per policy.
  The query, in `macro_parameter_in_a_join_condition_child`: `CREATE MACRO m(r :=
  NULL) AS TABLE SELECT a.x, b.v FROM a LEFT JOIN b ON b.x = coalesce(r, a.x)`,
  then `CREATE VIEW vv AS SELECT * FROM m()` and a select from it.
- **Watch.** `store::views::tests::a_macro_parameter_in_a_join_condition_still_aborts_a_debug_duckdb`
  runs the query in a child process and asserts that it aborts; it asserts
  nothing in a build without debug assertions.
- **When it lands.** Drop the `pr` CTE and use `coalesce(policy_run, <alias>.run_id)`
  in the joins; delete the watch test and its child.
