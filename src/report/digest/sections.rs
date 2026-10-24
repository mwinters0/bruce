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

/// The head of a statement over the counted links (D71) of a page, over every
/// kind, as `s`, and the broken links reached *through* a link to a URL that
/// doubles a slash, as `th` (D71).
///
/// A doubled-slash page is one of the run's whose own URL doubles one (`dp`).
/// Its path-relative links resolve against the doubled path, so they break
/// there and nowhere else. A link with reason `doubled_slash` reaches the page
/// it lands on, if that is in `dp`, and from it every page in `dp` its
/// path-relative links reach in turn (`reach`); the broken path-relative
/// links on those pages (`brk`) are the linking page's, each counted once
/// however many links of the page reach the same page. [`page_counts`] binds
/// the run into it.
const PAGE_COUNTS: &str = "WITH RECURSIVE s AS (SELECT url_id, sum(links)::BIGINT AS links, \
     sum(broken_url)::BIGINT AS broken_url, sum(broken_anchor)::BIGINT AS broken_anchor, \
     sum(error)::BIGINT AS error FROM report_stats WHERE run_id = ";

const PAGE_COUNTS_DOUBLED: &str = " GROUP BY url_id), \
     dp AS (SELECT d.url_id FROM doubled_slash d JOIN pages p ON p.run_id = d.run_id \
     AND p.url_id = d.url_id WHERE d.own AND d.run_id = ";

const PAGE_COUNTS_LINKS: &str = "), lv AS (SELECT v.src_url_id AS src, \
     coalesce(landing.url_id, v.target_url_id) AS dst, v.reason, v.relative, v.outcome, \
     v.counted FROM link_verdicts v \
     JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id \
     LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url \
     WHERE v.run_id = ";

const PAGE_COUNTS_THROUGH: &str = " AND (v.reason = 'doubled_slash' OR v.src_url_id IN \
     (SELECT url_id FROM dp))), \
     brk AS (SELECT src AS url_id, count(*)::BIGINT AS n FROM lv WHERE relative \
     AND outcome IN ('broken_url', 'broken_anchor', 'error') \
     AND src IN (SELECT url_id FROM dp) GROUP BY src), \
     reach(page, dst) AS (SELECT src, dst FROM lv WHERE reason = 'doubled_slash' AND counted \
     AND dst IN (SELECT url_id FROM dp) \
     UNION SELECT r.page, e.dst FROM reach r JOIN lv e ON e.src = r.dst \
     WHERE e.relative AND e.dst IN (SELECT url_id FROM dp)), \
     th AS (SELECT r.page AS url_id, sum(b.n)::BIGINT AS through FROM reach r \
     JOIN brk b ON b.url_id = r.dst GROUP BY r.page)";

/// [`PAGE_COUNTS`] over `run`, ready for the statement that reads `s` and `th`.
fn page_counts(run: RunId) -> Sql {
    let mut sql = Sql::new(PAGE_COUNTS);
    sql.bind(run.0)
        .push(PAGE_COUNTS_DOUBLED)
        .bind(run.0)
        .push(PAGE_COUNTS_LINKS)
        .bind(run.0)
        .push(PAGE_COUNTS_THROUGH);
    sql
}

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
         WHERE v.counted AND v.outcome = 'unchecked' AND v.run_id = ",
    );
    sql.bind(run.0)
        .push(" GROUP BY v.reason ORDER BY 2 DESC, 1");
    sql.fetch(conn)
}

/// The head of a statement over the links on the run's crawled pages as
/// `u`, each with its verdict's `outcome` and `reason`, its target's `url`
/// and `target_outcome`, and the URL's `scheme`, `host`, `origin` (scheme,
/// host and port), `path`, path segments `segs` and whether the host is a
/// root's (`root_host`); [`link_urls`] binds the run into it. A URL is split
/// by pattern, a target's being as the `url` crate writes one, or the raw
/// text of one that did not parse (D26).
const LINK_URLS: &str = "WITH r AS (SELECT coalesce(list(h), []) AS hosts FROM (SELECT \
     regexp_extract(unnest(roots), '^[A-Za-z][A-Za-z0-9+.-]*://(?:[^@/?#]*@)?(\\[[^\\]]*\\]|[^:/?#]*)', 1) \
     AS h FROM runs WHERE run_id = ";

const LINK_URLS_THEN: &str = ")), u AS (SELECT v.outcome, v.reason, v.target_url_id, t.url, \
     t.outcome AS target_outcome, \
     lower(regexp_extract(t.url, '^([A-Za-z][A-Za-z0-9+.-]*):', 1)) AS scheme, \
     regexp_extract(t.url, '^[A-Za-z][A-Za-z0-9+.-]*://(?:[^@/?#]*@)?(\\[[^\\]]*\\]|[^:/?#]*)', 1) AS host, \
     regexp_extract(t.url, '^[A-Za-z][A-Za-z0-9+.-]*://[^/?#]*') AS origin, \
     regexp_extract(t.url, '^[A-Za-z][A-Za-z0-9+.-]*://[^/?#]*(/[^?#]*)?', 1) AS path, \
     string_split(trim(path, '/'), '/') AS segs, list_contains(r.hosts, host) AS root_host \
     FROM link_verdicts v JOIN report_pages pg ON pg.run_id = v.run_id \
     AND pg.url_id = v.src_url_id AND pg.crawled \
     LEFT JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id \
     CROSS JOIN r WHERE v.counted AND v.run_id = ";

/// [`LINK_URLS`] over `run`.
fn link_urls(run: RunId) -> Sql {
    let mut sql = Sql::new(LINK_URLS);
    sql.bind(run.0).push(LINK_URLS_THEN).bind(run.0).push(") ");
    sql
}

/// The `unchecked` links by reason and by where they point, the reasons in
/// [`unchecked`]'s order and a reason's groups the most links first. A group
/// is a URL's scheme where it is not `http(s)`; for `skipped`, the host and the
/// path's last two segments, which say what a `skip` glob matched; on a root's
/// host, the URL up to the path's first segment, a root's scope being a URL's
/// prefix; else the host.
pub(super) fn unchecked_groups(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = link_urls(run);
    sql.push(
        "SELECT reason, \"group\", links, targets FROM (SELECT reason, CASE \
         WHEN url IS NULL THEN NULL \
         WHEN scheme = '' THEN url \
         WHEN scheme NOT IN ('http', 'https') THEN scheme || ':' \
         WHEN reason = 'skipped' AND len(segs) > 2 \
         THEN host || '/\u{2026}/' || array_to_string(segs[-2:], '/') \
         WHEN reason = 'skipped' THEN host || '/' || array_to_string(segs, '/') \
         WHEN root_host AND segs[1] = '' THEN origin || '/' \
         WHEN root_host THEN origin || '/' || segs[1] \
         || CASE WHEN len(segs) > 1 OR ends_with(path, '/') THEN '/' ELSE '' END \
         ELSE host END AS \"group\", \
         count(*)::BIGINT AS links, count(DISTINCT target_url_id)::BIGINT AS targets \
         FROM u WHERE outcome = 'unchecked' GROUP BY 1, 2) \
         ORDER BY sum(links) OVER (PARTITION BY reason) DESC, reason, links DESC, targets DESC, \
         \"group\"",
    );
    sql.fetch(conn)
}

/// The hosts no root starts on whose links the run asked about, the most
/// links first: the allowlist (D3) as far as the pages link to it, the run
/// recording no configuration (D64).
pub(super) fn checked_hosts(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = link_urls(run);
    sql.push(
        "SELECT host, count(*)::BIGINT AS links, \
         count(DISTINCT target_url_id)::BIGINT AS targets FROM u \
         WHERE target_outcome <> 'unchecked' AND scheme IN ('http', 'https') AND NOT root_host \
         GROUP BY 1 ORDER BY 2 DESC, 3 DESC, 1",
    );
    sql.fetch(conn)
}

/// The crawled pages holding a broken link or one that errored, ranked by
/// their red links (`broken_url` and `error`) with those broken through them
/// (`through`, D71) and then their amber ones (`broken_anchor`), the severity
/// of the HTML's dots.
pub(super) fn pages(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = page_counts(run);
    sql.push(
        " SELECT pg.url AS page, pg.title, p.source_repo, p.source_path, p.source_url, \
         s.links, s.broken_url, s.broken_anchor, s.error, \
         coalesce(th.through, 0)::BIGINT AS through \
         FROM report_pages pg JOIN pages p ON p.run_id = pg.run_id AND p.url_id = pg.url_id \
         JOIN s ON s.url_id = pg.url_id LEFT JOIN th ON th.url_id = pg.url_id \
         WHERE pg.crawled AND s.broken_url + s.broken_anchor + s.error > 0 AND pg.run_id = ",
    );
    sql.bind(run.0).push(
        " ORDER BY s.broken_url + s.error + coalesce(th.through, 0) DESC, \
         s.broken_anchor DESC, pg.url",
    );
    sql.fetch(conn)
}

/// Every broken link on a crawled page, a page's together, an unresolved
/// xref with its `hint` (D65); the caller puts the pages in `pages`' order.
pub(super) fn broken(conn: &Connection, run: RunId) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT b.page, b.raw_href, b.target, b.fragment, b.kind, b.outcome, b.reason, b.hint, \
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

/// The groups of crawled pages by `group`, each with its link counts, ranked
/// as [`pages`] ranks a page. `join` may end in a key `key` binds.
fn grouped(
    conn: &Connection,
    run: RunId,
    group: &str,
    column: &str,
    join: &str,
    key: Option<&str>,
) -> Result<Table, Error> {
    let mut sql = page_counts(run);
    sql.push(" SELECT * FROM (SELECT ")
        .push(group)
        .push(" AS ")
        .push(column)
        .push(
            ", count(*)::BIGINT AS pages, \
             (count(*) FILTER (WHERE s.broken_url + s.broken_anchor + s.error > 0))::BIGINT \
             AS broken_pages, \
             coalesce(sum(s.links), 0)::BIGINT AS links, \
             coalesce(sum(s.broken_url), 0)::BIGINT AS broken_url, \
             coalesce(sum(s.broken_anchor), 0)::BIGINT AS broken_anchor, \
             coalesce(sum(s.error), 0)::BIGINT AS error, \
             coalesce(sum(th.through), 0)::BIGINT AS through \
             FROM report_pages pg LEFT JOIN s ON s.url_id = pg.url_id \
             LEFT JOIN th ON th.url_id = pg.url_id ",
        )
        .push(join);
    if let Some(key) = key {
        sql.bind_text(key);
    }
    sql.push(" WHERE pg.crawled AND pg.run_id = ")
        .bind(run.0)
        .push(" GROUP BY 1) ORDER BY broken_url + error + through DESC, broken_anchor DESC, 1");
    sql.fetch(conn)
}

/// The repos the crawled pages were built from, ranked as [`pages`] ranks a
/// page; a page with no source is the row with no repo.
pub(super) fn by_repo(conn: &Connection, run: RunId) -> Result<Table, Error> {
    grouped(conn, run, "pg.source_repo", "repo", "", None)
}

/// The Antora components of the crawled pages, ranked as [`pages`] ranks a
/// page. A site
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
