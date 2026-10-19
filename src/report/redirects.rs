//! The `redirects` report: the links whose target was redirected, grouped by
//! where they landed (D7), so the forge's login page, which a thousand links
//! reach, is one row. Nothing is filtered by host (`roadmap-P4-reports.md`,
//! "The redirects report").

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, Sql};
use super::table::Table;
use crate::store::RunId;

/// A row for each landing URL: the links and the distinct URLs that reached
/// it, and the chain of the first of them. With `linking_pages`, the pages
/// the links are on.
pub(super) fn redirects(conn: &Connection, run: RunId, request: &Request) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT landing, count(*)::BIGINT AS links, count(DISTINCT target)::BIGINT AS urls, \
         arg_min(chain, target) AS chain",
    );
    if request.linking_pages {
        sql.push(", list(DISTINCT page ORDER BY page) AS pages");
    }
    sql.push(" FROM report_redirects WHERE ")
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
    )
    .push(" GROUP BY landing ORDER BY 2 DESC, 1")
    .limit(request);
    sql.fetch(conn)
}

#[cfg(test)]
mod tests;
