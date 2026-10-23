//! The queries behind the digest's sections: each reads one run's crawled
//! pages in every kind (D64), an allowlisted page holding no links (D3), so a
//! sitemap file's entries are no row of any but the redirects' and the
//! housekeeping's own list of them.

use duckdb::Connection;

use super::document::{Run, Totals};
use crate::extract::Kind;
use crate::report::error::Error;
use crate::report::reach;
use crate::report::redirects;
use crate::report::request::{Report, Request};
use crate::report::sql::Sql;
use crate::report::table::{Cell, Table};
use crate::store::{Mode, RunId};

/// A link of `view`, a `report_*` link view, on a crawled page.
const ON_CRAWLED: &str = " JOIN report_pages pg ON pg.run_id = b.run_id \
                          AND pg.url_id = b.src_url_id AND pg.crawled ";

/// A page's link counts over every kind.
const PAGE_COUNTS: &str = "WITH s AS (SELECT url_id, sum(links)::BIGINT AS links, \
     sum(broken_url)::BIGINT AS broken_url, sum(broken_anchor)::BIGINT AS broken_anchor, \
     sum(error)::BIGINT AS error FROM report_stats WHERE run_id = ";

/// The run's context, or why it has no digest.
pub(super) fn run(conn: &Connection, run: RunId) -> Result<Run, Error> {
    let mut sql = Sql::new(
        "SELECT mode, tool_version, epoch(started_at)::BIGINT, epoch(finished_at)::BIGINT, roots \
         FROM runs WHERE run_id = ",
    );
    sql.bind(run.0);
    let table = sql.fetch(conn)?;
    match table.rows.first().map(Vec::as_slice) {
        Some(
            [
                Cell::Text(mode),
                Cell::Text(tool_version),
                Cell::Int(started),
                finished,
                Cell::List(roots),
            ],
        ) => {
            if mode != Mode::Crawl.as_str() {
                return Err(Error::NotCrawl(run));
            }
            Ok(Run {
                id: run,
                tool_version: tool_version.clone(),
                started: *started,
                finished: match finished {
                    Cell::Int(at) => Some(*at),
                    _ => None,
                },
                roots: roots.clone(),
            })
        }
        _ => Err(Error::NoSuchRun(run)),
    }
}

/// The run's pages and links, and the links by outcome.
pub(super) fn totals(conn: &Connection, run: RunId) -> Result<Totals, Error> {
    let mut sql = Sql::new("SELECT (SELECT count(*) FROM report_pages WHERE run_id = ");
    sql.bind(run.0)
        .push(
            " AND crawled)::BIGINT, coalesce(sum(s.links), 0)::BIGINT, \
             coalesce(sum(s.broken_url), 0)::BIGINT, coalesce(sum(s.broken_anchor), 0)::BIGINT, \
             coalesce(sum(s.error), 0)::BIGINT, coalesce(sum(s.unchecked), 0)::BIGINT \
             FROM report_stats s JOIN report_pages pg ON pg.run_id = s.run_id \
             AND pg.url_id = s.url_id AND pg.crawled WHERE s.run_id = ",
        )
        .bind(run.0);
    let table = sql.fetch(conn)?;
    match table.rows.first().map(Vec::as_slice) {
        Some(
            [
                Cell::Int(pages),
                Cell::Int(links),
                Cell::Int(broken_url),
                Cell::Int(broken_anchor),
                Cell::Int(error),
                Cell::Int(unchecked),
            ],
        ) => Ok(Totals {
            pages: *pages,
            links: *links,
            broken_url: *broken_url,
            broken_anchor: *broken_anchor,
            error: *error,
            unchecked: *unchecked,
        }),
        _ => unreachable!("an aggregate over no rows is still one row"),
    }
}

/// The `unchecked` links by reason, and the distinct targets they name.
pub(super) fn unchecked(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT v.reason, count(*)::BIGINT AS links, \
         count(DISTINCT v.target_url_id)::BIGINT AS targets \
         FROM link_verdicts v JOIN report_pages pg ON pg.run_id = v.run_id \
         AND pg.url_id = v.src_url_id AND pg.crawled \
         WHERE v.outcome = 'unchecked' AND v.run_id = ",
    );
    sql.bind(run.0)
        .push(" GROUP BY v.reason ORDER BY 2 DESC, 1");
    sql.fetch(conn)
}

/// The crawled pages holding a broken link, the most first.
pub(super) fn pages(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(PAGE_COUNTS);
    sql.bind(run.0).push(
        " GROUP BY url_id) \
         SELECT pg.url AS page, pg.title, p.source_repo, p.source_path, p.source_url, \
         s.links, s.broken_url, s.broken_anchor, s.error \
         FROM report_pages pg JOIN pages p ON p.run_id = pg.run_id AND p.url_id = pg.url_id \
         JOIN s ON s.url_id = pg.url_id \
         WHERE pg.crawled AND s.broken_url + s.broken_anchor > 0 AND pg.run_id = ",
    );
    sql.bind(run.0)
        .push(" ORDER BY s.broken_url + s.broken_anchor DESC, pg.url");
    sql.fetch(conn)
}

/// Every broken link on a crawled page, a page's together; the caller puts
/// the pages in `pages`' order.
pub(super) fn broken(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT b.page, b.raw_href, b.target, b.fragment, b.kind, b.outcome, b.reason, \
         b.status::INTEGER AS status FROM report_broken b",
    );
    sql.push(ON_CRAWLED)
        .push("WHERE b.run_id = ")
        .bind(run.0)
        .push(" ORDER BY b.page, b.outcome DESC, b.raw_href, b.fragment");
    sql.fetch(conn)
}

/// The broken targets, the one named by the most links first and then by the
/// most pages, each with the source of the page it names where the run holds
/// it: the page a redirect landed on, for a link to a page that moved.
pub(super) fn broken_targets(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT b.target, b.fragment, max(b.outcome) AS outcome, max(b.reason) AS reason, \
         max(b.status)::INTEGER AS status, count(*)::BIGINT AS links, \
         count(DISTINCT b.src_url_id)::BIGINT AS pages, max(tp.source_repo) AS source_repo, \
         max(tp.source_path) AS source_path, max(tp.source_url) AS source_url \
         FROM report_broken b",
    );
    sql.push(ON_CRAWLED).push(
        "LEFT JOIN targets t ON t.run_id = b.run_id AND t.url = b.target \
         LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url \
         LEFT JOIN pages tp ON tp.run_id = b.run_id \
         AND tp.url_id = coalesce(landing.url_id, t.url_id) \
         WHERE b.run_id = ",
    );
    sql.bind(run.0)
        .push(" GROUP BY b.target, b.fragment ORDER BY 6 DESC, 7 DESC, 1, 2");
    sql.fetch(conn)
}

/// The groups of crawled pages by `group`, each with its link counts, the
/// most broken first. `join` may end in a key `key` binds.
fn grouped(
    conn: &Connection,
    run: RunId,
    group: &str,
    column: &str,
    join: &str,
    key: Option<&str>,
) -> Result<Table, Error> {
    let mut sql = Sql::new(PAGE_COUNTS);
    sql.bind(run.0)
        .push(" GROUP BY url_id) SELECT * FROM (SELECT ")
        .push(group)
        .push(" AS ")
        .push(column)
        .push(
            ", count(*)::BIGINT AS pages, \
             (count(*) FILTER (WHERE s.broken_url + s.broken_anchor > 0))::BIGINT AS broken_pages, \
             coalesce(sum(s.links), 0)::BIGINT AS links, \
             coalesce(sum(s.broken_url), 0)::BIGINT AS broken_url, \
             coalesce(sum(s.broken_anchor), 0)::BIGINT AS broken_anchor, \
             coalesce(sum(s.error), 0)::BIGINT AS error \
             FROM report_pages pg LEFT JOIN s ON s.url_id = pg.url_id ",
        )
        .push(join);
    if let Some(key) = key {
        sql.bind_text(key);
    }
    sql.push(" WHERE pg.crawled AND pg.run_id = ")
        .bind(run.0)
        .push(" GROUP BY 1) ORDER BY broken_url + broken_anchor DESC, error DESC, 1");
    sql.fetch(conn)
}

/// The repos the crawled pages were built from, the most broken first; a
/// page with no source is the row with no repo.
pub(super) fn by_repo(conn: &Connection, run: RunId) -> Result<Table, Error> {
    grouped(conn, run, "pg.source_repo", "repo", "", None)
}

/// The Antora components of the crawled pages, the most broken first. A site
/// of no Antora pages has none.
pub(super) fn by_component(conn: &Connection, run: RunId) -> Result<Table, Error> {
    grouped(
        conn,
        run,
        "f.value",
        "component",
        "JOIN page_facts f ON f.run_id = pg.run_id AND f.url_id = pg.url_id AND f.key = ",
        Some("antora.component"),
    )
}

/// Every link whose outcome is `error` on a crawled page (D16), a page's
/// together, with the page's source.
pub(super) fn errors(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT b.page, p.source_url, b.raw_href, b.target, b.fragment, b.kind, b.reason, \
         b.status::INTEGER AS status FROM report_errors b",
    );
    sql.push(ON_CRAWLED)
        .push(
            "LEFT JOIN pages p ON p.run_id = b.run_id AND p.url_id = b.src_url_id \
             WHERE b.run_id = ",
        )
        .bind(run.0)
        .push(" ORDER BY b.page, b.raw_href, b.fragment");
    sql.fetch(conn)
}

/// A request for one of the reports the digest's housekeeping reuses: every
/// kind, every row.
fn every(report: Report) -> Request {
    let mut request = Request::new(report);
    request.kinds = Kind::ALL.to_vec();
    request
}

/// The redirected links by landing URL, with the pages they are on.
pub(super) fn redirects(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut request = every(Report::Redirects);
    request.linking_pages = true;
    redirects::redirects(conn, run, &request)
}

/// The sitemap's pages no page links to.
pub(super) fn orphans(conn: &Connection, run: RunId) -> Result<Table, Error> {
    reach::orphans(conn, run, &every(Report::Orphans))
}

/// The pages absent from the sitemap of a root that read one.
pub(super) fn unlisted(conn: &Connection, run: RunId) -> Result<Table, Error> {
    Ok(reach::unlisted(conn, run, &every(Report::Unlisted))?.0)
}

/// The sitemap's entries no page was fetched for, with what each answered,
/// those that did not answer `ok` first (`report sitemap`'s rows without a
/// page, `report_sitemap`).
pub(super) fn sitemap_entries(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT page, outcome, reason, status::INTEGER AS status FROM report_sitemap \
         WHERE via_links IS NULL AND run_id = ",
    );
    sql.bind(run.0).push(" ORDER BY outcome = 'ok', page");
    sql.fetch(conn)
}

/// The roots that read no sitemap, which have no unlisted pages for that.
pub(super) fn roots_without_sitemap(conn: &Connection, run: RunId) -> Result<Vec<String>, Error> {
    let mut sql = Sql::new("SELECT root FROM report_roots WHERE NOT has_sitemap AND run_id = ");
    sql.bind(run.0).push(" ORDER BY root");
    Ok(sql
        .fetch(conn)?
        .rows
        .iter()
        .filter_map(|row| row[0].text().map(str::to_owned))
        .collect())
}
