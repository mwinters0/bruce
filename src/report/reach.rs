//! How a page is reached: by a link from another page, and by a root's
//! sitemap. The reports are `inbound`, `orphans`, `sitemap` and `unlisted`
//! (D14); each is about a page, so the filters name the page a row lists.
//!
//! A link counts for the page it lands on, so one to a URL that redirected
//! counts for the page it reached (`report_inbound`). The sitemap kinds do not
//! count, and neither does a page's link to itself.

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, PAGE, Sql};
use super::table::Table;
use crate::store::RunId;

/// Crawled pages by the distinct other pages linking to them, most first. A
/// page nothing links to is a row of 0. `--kind` says which links count.
pub(super) fn inbound(conn: &Connection, run: RunId, request: &Request) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT pg.url AS page, count(DISTINCT i.src_url_id)::BIGINT AS inbound, pg.title \
         FROM report_pages pg LEFT JOIN report_inbound i \
         ON i.run_id = pg.run_id AND i.url_id = pg.url_id AND ",
    );
    sql.kinds("i.kind", request).push(" WHERE pg.crawled AND ");
    sql.scope(run, request, PAGE)
        .push(" GROUP BY pg.url_id, pg.url, pg.title ORDER BY 2 DESC, 1")
        .limit(request);
    sql.fetch(conn)
}

/// The pages a root's sitemap lists that no other page links to, by the kinds
/// asked for.
pub(super) fn orphans(conn: &Connection, run: RunId, request: &Request) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT pg.url AS page, pg.title, pg.root, pg.source_repo, pg.source_path \
         FROM report_pages pg WHERE pg.crawled AND pg.in_sitemap AND NOT EXISTS \
         (SELECT 1 FROM report_inbound i WHERE i.run_id = pg.run_id AND i.url_id = pg.url_id AND ",
    );
    sql.kinds("i.kind", request).push(") AND ");
    sql.scope(run, request, PAGE)
        .push(" ORDER BY pg.root, pg.url")
        .limit(request);
    sql.fetch(conn)
}

/// Crawled pages absent from the sitemap of their root, where the root read
/// one (`report_roots`: a file reached from it answered `ok` with no reason);
/// the roots that read none are named in the notes, there being nothing to be
/// absent from.
pub(super) fn unlisted(
    conn: &Connection,
    run: RunId,
    request: &Request,
) -> Result<(Table, Vec<String>), Error> {
    let mut sql = Sql::new(
        "SELECT pg.url AS page, pg.root, pg.title, pg.source_repo, pg.source_path \
         FROM report_pages pg JOIN report_roots r \
         ON r.run_id = pg.run_id AND r.root = pg.root AND r.has_sitemap \
         WHERE pg.crawled AND NOT pg.in_sitemap AND ",
    );
    sql.scope(run, request, PAGE)
        .push(" ORDER BY pg.root, pg.url")
        .limit(request);
    let table = sql.fetch(conn)?;
    let mut bare = Sql::new("SELECT root FROM report_roots WHERE NOT has_sitemap AND run_id = ");
    bare.bind(run.0).push(" ORDER BY root");
    let notes = bare
        .fetch(conn)?
        .rows
        .iter()
        .filter_map(|row| row[0].text())
        .map(|root| format!("{root} read no sitemap, so no page of it is unlisted"))
        .collect();
    Ok((table, notes))
}

/// Every crawled page, and every sitemap entry no page was fetched for, with
/// what its target answered. The entries come first, then the pages the
/// sitemap does not list. `--kind` is no filter here, and an entry has no
/// page to match `--repo` or `--fact` against.
pub(super) fn sitemap(conn: &Connection, run: RunId, request: &Request) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT page, title, in_sitemap, via_links, outcome, reason, status::INTEGER AS status, \
         source_repo, source_path FROM report_sitemap WHERE ",
    );
    sql.scope(
        run,
        request,
        Cols {
            run: "run_id",
            repo: "source_repo",
            page: "url_id",
        },
    )
    .push(" ORDER BY via_links IS NOT NULL, in_sitemap, page")
    .limit(request);
    sql.fetch(conn)
}

#[cfg(test)]
mod tests;
