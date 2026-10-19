//! The `diff` report: the links broken in one run and not in another
//! (`roadmap-P4-reports.md`, "What each report holds").
//!
//! A link is keyed by its page, its target and its fragment, which are the
//! same in every run (`url_id` is a hash of the canonical form, D26), and is
//! broken where `report_broken` has it. A key broken in `to` and not in
//! `from` is `new`; broken in `from` and not in `to` is `fixed`. Not broken
//! includes absent, so a broken link on a page the run did not reach is
//! `fixed`, one on a page that is new `new`: the run lost or gained the
//! breakage either way. Several links sharing a key are one row.

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, Sql};
use super::table::Table;
use crate::store::RunId;

/// The columns of both halves, in the order `broken` prints them after
/// `change`.
const COLUMNS: &str = "b.page, b.raw_href, b.target, b.fragment, b.kind, b.outcome, b.reason, \
                       b.status::INTEGER AS status, b.source_repo, b.source_path";

/// The links of `to` and of `from` that changed, `new` first, and a note
/// naming the runs. `--repo` and `--fact` filter the page a row is on in the
/// run the row comes from.
pub(super) fn diff(
    conn: &Connection,
    from: RunId,
    to: RunId,
    request: &Request,
) -> Result<(Table, Vec<String>), Error> {
    let mut sql = Sql::new("SELECT * FROM (");
    half(&mut sql, "new", to, from, request);
    sql.push(" UNION ALL ");
    half(&mut sql, "fixed", from, to, request);
    sql.push(") ORDER BY change = 'fixed', outcome DESC, page, raw_href, fragment")
        .limit(request);
    let note = format!("from run {} to run {}", from.0, to.0);
    Ok((sql.fetch(conn)?, vec![note]))
}

/// The links broken in `run` and not in `other`, labelled `change`.
fn half(sql: &mut Sql, change: &str, run: RunId, other: RunId, request: &Request) {
    sql.push("SELECT '")
        .push(change)
        .push("' AS change, ")
        .push(COLUMNS)
        .push(" FROM report_broken b WHERE ")
        .kinds("b.kind", request)
        .push(" AND ");
    sql.scope(
        run,
        request,
        Cols {
            run: "b.run_id",
            repo: "b.source_repo",
            page: "b.src_url_id",
        },
    );
    sql.push(" AND NOT EXISTS (SELECT 1 FROM report_broken o WHERE o.run_id = ")
        .bind(other.0)
        .push(
            " AND o.src_url_id = b.src_url_id AND o.target IS NOT DISTINCT FROM b.target \
             AND coalesce(o.fragment, '') = coalesce(b.fragment, '') AND ",
        )
        .kinds("o.kind", request)
        .push(
            ") QUALIFY row_number() OVER (PARTITION BY b.src_url_id, b.target, \
             coalesce(b.fragment, '') ORDER BY b.raw_href, b.kind) = 1",
        );
}

#[cfg(test)]
mod tests;
