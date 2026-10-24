//! The `anchors-removed` report: the anchors a page had in one run and lost
//! in another while links still name them.
//!
//! A row is a page that lost an anchor, one of `from`'s `anchors` that `to`'s
//! page lacks, and the fragment as the links name it. The links are `to`'s
//! `broken_anchor` ones (`report_broken`), which already resolve a redirect
//! to the page it landed on and leave out the links that do not count (D71);
//! one that names an anchor `from`'s page had is a link the loss broke. An
//! anchor no link names is no row, and nor is a link to an anchor the page
//! never had. The page is listed by the URL the links landed on, and
//! `--repo` and `--fact` filter it, the repo being the one to restore the
//! anchor in.

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, Sql};
use super::table::Table;
use crate::store::RunId;

/// A row for each page and fragment, the most linked first: the links naming
/// it and the distinct pages they are on, and with `linking_pages` those
/// pages.
pub(super) fn anchors_removed(
    conn: &Connection,
    from: RunId,
    to: RunId,
    request: &Request,
) -> Result<(Table, Vec<String>), Error> {
    let mut sql = Sql::new(
        "SELECT pg.url AS page, b.fragment AS anchor, count(*)::BIGINT AS links, \
         count(DISTINCT b.src_url_id)::BIGINT AS pages",
    );
    if request.linking_pages {
        sql.push(", list(DISTINCT b.page ORDER BY b.page) AS linking_pages");
    }
    sql.push(
        ", pg.source_repo, pg.source_path FROM report_broken b \
         JOIN targets t ON t.run_id = b.run_id AND t.url = b.target \
         LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url \
         JOIN report_pages pg ON pg.run_id = t.run_id \
         AND pg.url_id = coalesce(landing.url_id, t.url_id) \
         WHERE b.outcome = 'broken_anchor' AND ",
    )
    .kinds("b.kind", request)
    .push(" AND ");
    sql.scope(
        to,
        request,
        Cols {
            run: "b.run_id",
            repo: "pg.source_repo",
            page: "pg.url_id",
        },
    );
    sql.push(" AND EXISTS (SELECT 1 FROM anchors a WHERE a.run_id = ")
        .bind(from.0)
        .push(
            " AND a.url_id = pg.url_id \
             AND a.anchor IN (b.fragment, coalesce(try(url_decode(b.fragment)), b.fragment))) \
             GROUP BY pg.url, b.fragment, pg.source_repo, pg.source_path \
             ORDER BY links DESC, pages DESC, page, anchor",
        )
        .limit(request);
    let note = format!("from run {} to run {}", from.0, to.0);
    Ok((sql.fetch(conn)?, vec![note]))
}

#[cfg(test)]
mod tests;
