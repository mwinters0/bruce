//! The views reports read, installed at every open beside the store's and the
//! profiles' (D6). Each carries every run, as a view takes no parameter, and
//! the columns a filter needs: `run_id` and the page's `url_id`.

use crate::profile::View;

/// Every view the reports ship.
pub const VIEWS: &[View] = &[PAGES, BROKEN, ERRORS, STATS];

/// A page of a run and where it came from. `crawled` is a page a root's scope
/// claimed (`pages.root`), not an allowlisted one fetched for its anchors.
const PAGES: View = View {
    name: "report_pages",
    select: "\
SELECT p.run_id, p.url_id, t.url, p.title, p.source_repo, p.source_path, p.root,
    p.root IS NOT NULL AS crawled, p.in_sitemap, p.via_links, p.profile
FROM pages p
JOIN targets t ON t.run_id = p.run_id AND t.url_id = p.url_id",
};

/// The links of one outcome set, with the page they are on and what their
/// target answered. `$outcomes` is the `IN` list.
macro_rules! link_rows {
    ($outcomes:literal) => {
        concat!(
            "\
SELECT v.run_id, v.src_url_id, s.url AS page, v.raw_href, t.url AS target, v.fragment, v.kind,
    v.outcome, v.reason, t.status, p.source_repo, p.source_path
FROM link_verdicts v
LEFT JOIN targets s ON s.run_id = v.run_id AND s.url_id = v.src_url_id
LEFT JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id
LEFT JOIN pages p ON p.run_id = v.run_id AND p.url_id = v.src_url_id
WHERE v.outcome IN ",
            $outcomes
        )
    };
}

/// A link whose outcome is `broken_url` or `broken_anchor`.
const BROKEN: View = View {
    name: "report_broken",
    select: link_rows!("('broken_url', 'broken_anchor')"),
};

/// A link whose outcome is `error`, apart from broken (D16).
const ERRORS: View = View {
    name: "report_errors",
    select: link_rows!("('error')"),
};

/// A page's link counts by kind, one row for each kind it links by. A page
/// with no links has no row; `bruce report stats` joins this to
/// `report_pages` to keep it.
const STATS: View = View {
    name: "report_stats",
    select: "\
SELECT run_id, src_url_id AS url_id, kind,
    count(*)::BIGINT AS links,
    (count(*) FILTER (WHERE outcome = 'broken_url'))::BIGINT AS broken_url,
    (count(*) FILTER (WHERE outcome = 'broken_anchor'))::BIGINT AS broken_anchor,
    (count(*) FILTER (WHERE outcome = 'error'))::BIGINT AS error,
    (count(*) FILTER (WHERE outcome = 'unchecked'))::BIGINT AS unchecked
FROM link_verdicts
GROUP BY run_id, src_url_id, kind",
};
