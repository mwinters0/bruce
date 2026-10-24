//! The `broken` and `errors` reports: a row for each link of a `report_*`
//! link view, which the gate reads too for its errors. A broken link has a
//! `hint` an error has not (D65).

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, Sql};
use super::table::Table;
use crate::store::RunId;

/// A row per broken link, `broken_url` first, with the nearest page ID to an
/// unresolved xref's refspec as its `hint` (D65).
pub(super) fn broken(conn: &Connection, run: RunId, request: &Request) -> Result<Table, Error> {
    links(conn, run, request, "report_broken", "reason, hint")
}

/// A row per link that could not be asked, apart from the broken ones (D16).
pub(super) fn errors(conn: &Connection, run: RunId, request: &Request) -> Result<Table, Error> {
    links(conn, run, request, "report_errors", "reason")
}

/// A row per link of a `report_*` link view, the worst outcome first, with
/// `after_outcome` the columns between it and the status.
fn links(
    conn: &Connection,
    run: RunId,
    request: &Request,
    view: &str,
    after_outcome: &str,
) -> Result<Table, Error> {
    let mut sql = Sql::new("SELECT page, raw_href, target, fragment, kind, outcome, ");
    sql.push(after_outcome)
        .push(", status::INTEGER AS status, source_repo, source_path FROM ")
        .push(view)
        .push(" WHERE ")
        .kinds("kind", request)
        .push(" AND ");
    sql.scope(
        run,
        request,
        Cols {
            run: "run_id",
            repo: "source_repo",
            page: "src_url_id",
        },
    );
    sql.push(" ORDER BY outcome DESC, page, raw_href, fragment")
        .limit(request);
    sql.fetch(conn)
}
