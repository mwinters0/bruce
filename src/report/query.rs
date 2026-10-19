//! The report to run, and the queries of the link reports and `stats`: a
//! view, the run, the filters, the order and the limit. The page reports are
//! in `reach`, `redirects` and `diff`.

use duckdb::Connection;

use super::error::Error;
use super::request::{Report, Request};
use super::run::Chosen;
use super::sql::{Cols, PAGE, Sql};
use super::table::{Cell, Table};
use super::{diff, reach, redirects};
use crate::store::RunId;

/// Runs the report, returning its table and the notes a text rendering adds.
pub(super) fn run(
    conn: &Connection,
    chosen: &Chosen,
    request: &Request,
) -> Result<(Table, Vec<String>), Error> {
    let run = chosen.run;
    match request.report {
        Report::Broken => Ok((links(conn, run, request, "report_broken")?, Vec::new())),
        Report::Errors => Ok((links(conn, run, request, "report_errors")?, Vec::new())),
        Report::Stats => stats(conn, run, request),
        Report::Inbound => Ok((reach::inbound(conn, run, request)?, Vec::new())),
        Report::Sitemap => Ok((reach::sitemap(conn, run, request)?, Vec::new())),
        Report::Orphans => Ok((reach::orphans(conn, run, request)?, Vec::new())),
        Report::Unlisted => reach::unlisted(conn, run, request),
        Report::Redirects => Ok((redirects::redirects(conn, run, request)?, Vec::new())),
        Report::Diff => {
            let (from, _) = chosen.from.expect("choose gives a diff its base run");
            Ok(diff::diff(conn, from, run, request)?)
        }
    }
}

/// A row per link of a `report_*` link view, the worst outcome first.
fn links(conn: &Connection, run: RunId, request: &Request, view: &str) -> Result<Table, Error> {
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

/// A row per crawled page with its counts, the most broken first, and the run's
/// totals in the notes.
fn stats(conn: &Connection, run: RunId, request: &Request) -> Result<(Table, Vec<String>), Error> {
    let mut sql = Sql::new(
        "SELECT * FROM (SELECT pg.url AS page, coalesce(sum(s.links), 0)::BIGINT AS links, \
         coalesce(sum(s.broken_url), 0)::BIGINT AS broken_url, \
         coalesce(sum(s.broken_anchor), 0)::BIGINT AS broken_anchor, \
         coalesce(sum(s.error), 0)::BIGINT AS error, \
         coalesce(sum(s.unchecked), 0)::BIGINT AS unchecked \
         FROM report_pages pg LEFT JOIN report_stats s \
         ON s.run_id = pg.run_id AND s.url_id = pg.url_id AND ",
    );
    sql.kinds("s.kind", request).push(" WHERE pg.crawled AND ");
    sql.scope(run, request, PAGE).push(
        " GROUP BY pg.url_id, pg.url) \
         ORDER BY broken_url + broken_anchor DESC, error DESC, links DESC, page",
    );
    let mut table = sql.fetch(conn)?;
    let notes = totals(conn, run, request, &table)?;
    if let Some(top) = request.top {
        table.rows.truncate(top);
    }
    Ok((table, notes))
}

/// The run's totals over every page the filters keep, `unchecked` links and
/// the distinct targets they name counted by reason, which no row carries.
fn totals(
    conn: &Connection,
    run: RunId,
    request: &Request,
    pages: &Table,
) -> Result<Vec<String>, Error> {
    let sum = |col: usize| -> i64 {
        pages
            .rows
            .iter()
            .map(|r| match r[col] {
                Cell::Int(n) => n,
                _ => 0,
            })
            .sum()
    };
    let mut line = format!(
        "total: {} pages, {} links, {} broken_url, {} broken_anchor, {} error, {} unchecked",
        pages.rows.len(),
        sum(1),
        sum(2),
        sum(3),
        sum(4),
        sum(5)
    );
    let mut sql = Sql::new(
        "SELECT coalesce(v.reason, '(none)') AS reason, count(*)::BIGINT AS links, \
         count(DISTINCT v.target_url_id)::BIGINT AS targets \
         FROM link_verdicts v JOIN report_pages pg \
         ON pg.run_id = v.run_id AND pg.url_id = v.src_url_id \
         WHERE pg.crawled AND v.outcome = 'unchecked' AND ",
    );
    sql.kinds("v.kind", request).push(" AND ");
    sql.scope(run, request, PAGE)
        .push(" GROUP BY 1 ORDER BY 2 DESC, 1");
    let reasons = sql.fetch(conn)?;
    if reasons.rows.is_empty() {
        return Ok(vec![line]);
    }
    line.push_str("\nunchecked by reason:");
    for row in &reasons.rows {
        if let [Cell::Text(reason), Cell::Int(links), Cell::Int(targets)] = &row[..] {
            line.push_str(&format!("\n  {reason}: {links} links, {targets} targets"));
        }
    }
    Ok(vec![line])
}
