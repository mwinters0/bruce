//! The inverse check: what a change removed that pages outside the build link
//! to (D4, D9; `roadmap-P5-ci.md`, "The inverse check").
//!
//! A **touched** page is a baseline page in a component-version the build
//! holds, which Antora built whole (D22). What the CI run holds of the build
//! is read back from its rows, never from the process that wrote them: its
//! built pages carry `antora.component` and `antora.version` (D19), and a
//! component-version is held as `crawl::baseline::Held` has it, a URL below
//! the component's segment of a built page's URL and, for a named version,
//! below that version's segment too (I16, D56). A baseline link is
//! **inverse breakage** when it is on a crawled page outside the build, was
//! `ok`, and the run removed what it reached:
//!
//! - `page`: the target, or the page it redirected to, is in a held
//!   component-version, is a page in the baseline and is not built; a
//!   target that redirected to a landing in no held one (a page-alias another
//!   component's page owns) is that owner's, and removed only with its landing;
//! - `file`: the same of a target that was no page (an image, an attachment),
//!   every file of the build being a target of the run (`ci`, D56);
//! - `anchor`: the target is built and its page lacks the anchor the link's
//!   fragment named.
//!
//! A link on a page in a held component-version is the gate's, or gone with
//! its page.

use std::collections::BTreeSet;

use duckdb::Connection;
use percent_encoding::percent_decode_str;
use url::{Position, Url};

use super::error::Error;
use super::request::Request;
use super::sql::{Cols, Sql};
use super::table::Table;
use crate::store::RunId;

/// The columns of a baseline link's crawled page.
const LINK: Cols = Cols {
    run: "v.run_id",
    repo: "ps.source_repo",
    page: "v.src_url_id",
};

/// The URL prefixes the build holds: for each built page's component-version,
/// the URL through the component's segment, or through its version's where
/// the version is named.
fn held(conn: &Connection, run: RunId) -> Result<BTreeSet<String>, Error> {
    let mut stmt = conn.prepare(
        "SELECT t.url, c.value, v.value FROM pages p \
         JOIN targets t ON t.run_id = p.run_id AND t.url_id = p.url_id \
         JOIN page_facts c ON c.run_id = p.run_id AND c.url_id = p.url_id \
            AND c.key = 'antora.component' \
         LEFT JOIN page_facts v ON v.run_id = p.run_id AND v.url_id = p.url_id \
            AND v.key = 'antora.version' \
         WHERE p.run_id = ?",
    )?;
    let mut rows = stmt.query([run.0])?;
    let mut prefixes = BTreeSet::new();
    while let Some(row) = rows.next()? {
        let (url, component, version): (String, String, Option<String>) =
            (row.get(0)?, row.get(1)?, row.get(2)?);
        prefixes.extend(prefix(&url, &component, version.as_deref()));
    }
    Ok(prefixes)
}

/// The prefix of `url`, a page of `component` at `version`: its URL through
/// the first segment that is the component, and a named version's after it.
/// The placeholders Antora writes for a version its URLs omit, `""` and
/// `master`, and no version, name none (D19).
fn prefix(url: &str, component: &str, version: Option<&str>) -> Option<String> {
    let named = version.filter(|v| !matches!(*v, "" | "master"));
    let parsed = Url::parse(url).ok()?;
    let segments: Vec<&str> = parsed.path().split('/').collect();
    let decoded = |s: &str| percent_decode_str(s).decode_utf8_lossy().into_owned();
    let at = (1..segments.len()).find(|&i| {
        decoded(segments[i]) == component
            && named.is_none_or(|v| segments.get(i + 1).is_some_and(|s| decoded(s) == v))
    })?;
    let end = at + usize::from(named.is_some());
    Some(format!(
        "{}{}/",
        &parsed[..Position::BeforePath],
        segments[..=end].join("/")
    ))
}

/// The baseline links the change left broken on pages outside the build, a
/// row for each page, target and fragment, grouped by the linking page's
/// component, else its repo.
pub(super) fn inverse(
    conn: &Connection,
    run: RunId,
    base: RunId,
    request: &Request,
) -> Result<Table, Error> {
    let held = held(conn, run)?;
    let (ci, base) = (run.0, base.0);
    let mut sql = Sql::new("WITH held(prefix) AS (VALUES ");
    if held.is_empty() {
        sql.push("(CAST(NULL AS VARCHAR))");
    }
    for (i, prefix) in held.iter().enumerate() {
        sql.push(if i > 0 { ", (" } else { "(" })
            .bind_text(prefix)
            .push(")");
    }
    // The run ids are numbers, written into the text; the user's values are bound.
    sql.push(
        "), cand AS (SELECT v.src_url_id, v.target_url_id, v.fragment, v.raw_href, v.kind, \
         s.url AS page, t.url AS target, lnd.url AS landing_url, \
         coalesce(lnd.url_id, t.url_id) AS landed_id, \
         coalesce(try(url_decode(v.fragment)), v.fragment) AS decoded, \
         ps.source_repo, ps.source_path \
         FROM link_verdicts v \
         JOIN pages ps ON ps.run_id = v.run_id AND ps.url_id = v.src_url_id \
            AND ps.root IS NOT NULL \
         JOIN targets s ON s.run_id = v.run_id AND s.url_id = v.src_url_id \
         JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id \
         LEFT JOIN targets lnd ON lnd.run_id = t.run_id AND lnd.url = t.final_url \
         WHERE ",
    );
    sql.scope(RunId(base), request, LINK)
        .push(" AND v.outcome = 'ok' AND ")
        .kinds("v.kind", request)
        .push(&format!(
            " AND NOT EXISTS (SELECT 1 FROM held h WHERE starts_with(s.url, h.prefix)) \
             AND NOT EXISTS (SELECT 1 FROM targets c WHERE c.run_id = {ci} \
                AND c.url_id = v.src_url_id AND c.answered_by = 'local')), \
             judged AS (SELECT c.*, \
             EXISTS (SELECT 1 FROM targets x WHERE x.run_id = {ci} AND x.url_id = c.target_url_id \
                AND x.answered_by = 'local' AND x.outcome = 'ok') AS built, \
             EXISTS (SELECT 1 FROM pages bp WHERE bp.run_id = {base} \
                AND bp.url_id = c.landed_id) AS is_page, \
             EXISTS (SELECT 1 FROM held h WHERE starts_with(c.target, h.prefix)) AS held_target, \
             c.landing_url IS NOT NULL \
                AND NOT EXISTS (SELECT 1 FROM held h WHERE starts_with(c.landing_url, h.prefix)) \
                AS lands_outside, \
             c.landing_url IS NOT NULL \
                AND EXISTS (SELECT 1 FROM held h WHERE starts_with(c.landing_url, h.prefix)) \
                AND NOT EXISTS (SELECT 1 FROM targets x WHERE x.run_id = {ci} \
                    AND x.url_id = c.landed_id AND x.answered_by = 'local' \
                    AND x.outcome = 'ok') AS landing_gone, \
             coalesce(c.fragment, '') <> '' AND lower(c.fragment) <> 'top' \
                AND NOT contains(c.fragment, ':~:') AS real_fragment, \
             (SELECT coalesce(cl.url_id, ct.url_id) FROM targets ct \
                LEFT JOIN targets cl ON cl.run_id = ct.run_id AND cl.url = ct.final_url \
                WHERE ct.run_id = {ci} AND ct.url_id = c.target_url_id) AS ci_landed \
             FROM cand c), \
             typed AS (SELECT j.*, CASE \
             WHEN NOT built AND is_page AND ((held_target AND NOT lands_outside) OR landing_gone) \
                THEN 'page' \
             WHEN NOT built AND NOT is_page AND held_target THEN 'file' \
             WHEN built AND real_fragment \
                AND EXISTS (SELECT 1 FROM pages cp WHERE cp.run_id = {ci} \
                    AND cp.url_id = j.ci_landed) \
                AND NOT EXISTS (SELECT 1 FROM anchors a WHERE a.run_id = {ci} \
                    AND a.url_id = j.ci_landed \
                    AND (a.anchor = j.fragment OR a.anchor = j.decoded)) THEN 'anchor' \
             END AS removed FROM judged j) \
             SELECT page, raw_href, target, fragment, kind, removed, component, source_repo, \
             source_path FROM (SELECT t.page, t.raw_href, t.target, t.fragment, t.kind, \
             t.removed, (SELECT max(f.value) FROM page_facts f WHERE f.run_id = {base} \
                AND f.url_id = t.src_url_id AND f.key = 'antora.component') AS component, \
             t.source_repo, t.source_path FROM typed t WHERE t.removed IS NOT NULL \
             QUALIFY row_number() OVER (PARTITION BY t.src_url_id, t.target, \
                coalesce(t.fragment, '') ORDER BY t.raw_href, t.kind) = 1) \
             ORDER BY coalesce(component, source_repo), page, raw_href, fragment"
        ))
        .limit(request);
    sql.fetch(conn)
}

/// How many links, linking pages and groups of them (a component, else a
/// repo, else none) `table` holds, for D9's "N inbound pages across M
/// components".
pub(super) fn counts(table: &Table) -> (usize, usize) {
    let column = |name: &str| table.columns.iter().position(|c| c == name);
    let (Some(page), Some(component), Some(repo)) =
        (column("page"), column("component"), column("source_repo"))
    else {
        return (0, 0);
    };
    let mut pages = BTreeSet::new();
    let mut groups = BTreeSet::new();
    for row in &table.rows {
        pages.insert(row[page].text());
        groups.insert(row[component].text().or(row[repo].text()));
    }
    (pages.len(), groups.len())
}

#[cfg(test)]
mod tests;
