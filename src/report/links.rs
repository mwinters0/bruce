//! The `broken` and `errors` reports: a row for each link of a `report_*`
//! link view, which the gate reads too for its errors.

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, Sql};
use super::table::Table;
use crate::store::RunId;

/// A row per link of a `report_*` link view, the worst outcome first.
pub(super) fn links(
    conn: &Connection,
    run: RunId,
    request: &Request,
    view: &str,
) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT page, raw_href, target, fragment, kind, outcome, reason, status::INTEGER AS status, \
         source_repo, source_path FROM ",
    );
    sql.push(view)
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
