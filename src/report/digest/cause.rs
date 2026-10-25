//! Sections 3 and 4, "broken since" and "fixed since" the compared run: the
//! links broken in the digest's run and not in the one it is compared with,
//! each saying why by what the comparison held, and the links the comparison
//! had broken that are not.
//!
//! A link is keyed as `diff` keys it, by its page, its target and its
//! fragment, several links sharing a key being a key's, but a row is a link.
//! The cause is read off the comparison, never off a hash, and the first that
//! holds is the one named (D74): [`CAUSES`] lists them in the order a
//! maintainer reads.

use duckdb::Connection;

use super::sections::ON_CRAWLED;
use crate::report::error::Error;
use crate::report::sql::Sql;
use crate::report::table::Table;
use crate::store::RunId;

/// The causes of a new break, the order they are listed in. The first three
/// are a change in what a link names: the target page is gone, moved or lost
/// the anchor, and the source shown is the target's, as the comparison
/// recorded it; the rest are the linking page's own.
pub const CAUSES: [&str; 7] = [
    "target_removed",
    "target_moved",
    "anchor_removed",
    "link_added",
    "new_page",
    "unexplained",
    "first_judged",
];

/// The causes whose source shown is the target page's.
const TARGET_CAUSES: &str = "'target_removed', 'target_moved', 'anchor_removed'";

/// `list_position` of the cause, as an SQL list of [`CAUSES`].
fn cause_order() -> String {
    let list: Vec<String> = CAUSES.iter().map(|c| format!("'{c}'")).collect();
    format!("list_position([{}], cause)", list.join(", "))
}

/// The links of `to` that are broken and are not in `from`, each with its
/// `cause` (one of [`CAUSES`]) and the `source_repo`, `source_path` and
/// `source_url` of the page that answers for it: the target's, as `from`
/// recorded it, where the target changed, else the linking page's. The causes
/// in [`CAUSES`]' order, then `broken_url` before `broken_anchor`. The run ids
/// are written into the text, being numbers.
///
/// What `from` held of a key is read off every link of its run, counted or
/// not (D71), and a key it held broken is no new break.
pub(super) fn new_breakage(conn: &Connection, from: RunId, to: RunId) -> Result<Table, Error> {
    let (c, r) = (from.0, to.0);
    let order = cause_order();
    let mut sql = Sql::new(&format!(
        "WITH prior AS (SELECT src_url_id, target_url_id, coalesce(fragment, '') AS frag, \
         bool_or(outcome = 'ok') AS was_ok, \
         bool_or(outcome IN ('error', 'unchecked')) AS was_unjudged \
         FROM link_verdicts WHERE run_id = {c} GROUP BY ALL), \
         held AS (SELECT DISTINCT src_url_id, target, coalesce(fragment, '') AS frag \
         FROM report_broken WHERE run_id = {c}), \
         x AS (SELECT b.page, b.raw_href, b.target, b.fragment, b.kind, b.outcome, b.reason, \
         b.status::INTEGER AS status, \
         CASE WHEN cs.url_id IS NULL THEN 'new_page' \
         WHEN pr.src_url_id IS NULL THEN 'link_added' \
         WHEN NOT pr.was_ok AND pr.was_unjudged THEN 'first_judged' \
         WHEN NOT pr.was_ok THEN 'unexplained' \
         WHEN t.outcome = 'broken_url' THEN 'target_removed' \
         WHEN t.final_url IS NOT NULL AND ct.final_url IS DISTINCT FROM t.final_url \
         THEN 'target_moved' \
         WHEN b.outcome = 'broken_anchor' AND EXISTS (SELECT 1 FROM anchors a \
         WHERE a.run_id = {c} AND a.url_id = coalesce(cl.url_id, ct.url_id) \
         AND a.anchor IN (b.fragment, coalesce(try(url_decode(b.fragment)), b.fragment))) \
         THEN 'anchor_removed' \
         ELSE 'unexplained' END AS cause, \
         sp.source_repo AS l_repo, sp.source_path AS l_path, sp.source_url AS l_url, \
         coalesce(cp.source_repo, tp.source_repo) AS t_repo, \
         coalesce(cp.source_path, tp.source_path) AS t_path, \
         coalesce(cp.source_url, tp.source_url) AS t_url \
         FROM report_broken b"
    ));
    sql.push(ON_CRAWLED).push(&format!(
        "LEFT JOIN targets t ON t.run_id = b.run_id AND t.url = b.target \
         LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url \
         LEFT JOIN pages tp ON tp.run_id = b.run_id \
         AND tp.url_id = coalesce(landing.url_id, t.url_id) \
         LEFT JOIN pages sp ON sp.run_id = b.run_id AND sp.url_id = b.src_url_id \
         LEFT JOIN pages cs ON cs.run_id = {c} AND cs.url_id = b.src_url_id \
         AND cs.root IS NOT NULL \
         LEFT JOIN prior pr ON pr.src_url_id = b.src_url_id AND pr.target_url_id = t.url_id \
         AND pr.frag = coalesce(b.fragment, '') \
         LEFT JOIN targets ct ON ct.run_id = {c} AND ct.url_id = t.url_id \
         LEFT JOIN targets cl ON cl.run_id = {c} AND cl.url = ct.final_url \
         LEFT JOIN pages cp ON cp.run_id = {c} AND cp.url_id = coalesce(cl.url_id, ct.url_id) \
         WHERE b.run_id = {r} AND NOT EXISTS (SELECT 1 FROM held h \
         WHERE h.src_url_id = b.src_url_id AND h.target IS NOT DISTINCT FROM b.target \
         AND h.frag = coalesce(b.fragment, ''))) \
         SELECT page, raw_href, target, fragment, kind, outcome, reason, status, cause, \
         CASE WHEN cause IN ({TARGET_CAUSES}) THEN t_repo ELSE l_repo END AS source_repo, \
         CASE WHEN cause IN ({TARGET_CAUSES}) THEN t_path ELSE l_path END AS source_path, \
         CASE WHEN cause IN ({TARGET_CAUSES}) THEN t_url ELSE l_url END AS source_url \
         FROM x ORDER BY {order}, outcome DESC, page, raw_href, fragment"
    ));
    sql.fetch(conn)
}

/// The links `from` had broken that are not in `to`, with what each is there
/// in `became`: `ok`, `error` or `unchecked`; `link_removed`, the page no
/// longer holding it; `page_removed`, `to` not reaching the page; or
/// `uncounted`, a link `to` holds on a page whose twin answers for it (D71).
/// The page's source is `to`'s where it holds the page, else `from`'s. Ranked
/// as [`new_breakage`]'s rows are, without a cause.
pub(super) fn fixed_breakage(conn: &Connection, from: RunId, to: RunId) -> Result<Table, Error> {
    let (c, r) = (from.0, to.0);
    let mut sql = Sql::new(&format!(
        "WITH now AS (SELECT src_url_id, target_url_id, coalesce(fragment, '') AS frag, \
         bool_or(outcome = 'ok') AS any_ok, bool_or(outcome = 'error') AS any_error, \
         bool_or(outcome = 'unchecked') AS any_unchecked \
         FROM link_verdicts WHERE run_id = {r} GROUP BY ALL), \
         held AS (SELECT DISTINCT src_url_id, target, coalesce(fragment, '') AS frag \
         FROM report_broken WHERE run_id = {r}) \
         SELECT b.page, b.raw_href, b.target, b.fragment, b.kind, b.outcome, b.reason, \
         b.status::INTEGER AS status, \
         CASE WHEN np.url_id IS NULL THEN 'page_removed' \
         WHEN n.src_url_id IS NULL THEN 'link_removed' WHEN n.any_ok THEN 'ok' \
         WHEN n.any_error THEN 'error' WHEN n.any_unchecked THEN 'unchecked' \
         ELSE 'uncounted' END AS became, \
         coalesce(np.source_repo, cp.source_repo) AS source_repo, \
         coalesce(np.source_path, cp.source_path) AS source_path, \
         coalesce(np.source_url, cp.source_url) AS source_url \
         FROM report_broken b"
    ));
    sql.push(ON_CRAWLED).push(&format!(
        "LEFT JOIN targets t ON t.run_id = b.run_id AND t.url = b.target \
         LEFT JOIN pages cp ON cp.run_id = b.run_id AND cp.url_id = b.src_url_id \
         LEFT JOIN pages np ON np.run_id = {r} AND np.url_id = b.src_url_id \
         AND np.root IS NOT NULL \
         LEFT JOIN now n ON n.src_url_id = b.src_url_id AND n.target_url_id = t.url_id \
         AND n.frag = coalesce(b.fragment, '') \
         WHERE b.run_id = {c} AND NOT EXISTS (SELECT 1 FROM held h \
         WHERE h.src_url_id = b.src_url_id AND h.target IS NOT DISTINCT FROM b.target \
         AND h.frag = coalesce(b.fragment, '')) \
         ORDER BY b.outcome DESC, b.page, b.raw_href, b.fragment"
    ));
    sql.fetch(conn)
}

#[cfg(test)]
mod tests;
