//! The views reports read, installed at every open beside the store's and the
//! profiles' (D6). Each carries every run, as a view takes no parameter, and
//! the columns a filter needs: `run_id` and, but for `report_roots`, the
//! page's `url_id`.

use crate::profile::View;

/// Every view the reports ship.
pub const VIEWS: &[View] = &[
    PAGES, BROKEN, ERRORS, STATS, INBOUND, ROOTS, SITEMAP, REDIRECTS,
];

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

/// The links of one outcome set that count (D71), with the page they are on and
/// what their target answered. `$outcomes` is the `IN` list; `$columns` and `$joins` add
/// to the select list and the `FROM`.
macro_rules! link_rows {
    ($columns:literal, $joins:literal, $outcomes:literal) => {
        concat!(
            "\
SELECT v.run_id, v.src_url_id, s.url AS page, v.raw_href, t.url AS target, v.fragment, v.kind,
    v.outcome, v.reason, t.status, p.source_repo, p.source_path",
            $columns,
            "
FROM link_verdicts v
LEFT JOIN targets s ON s.run_id = v.run_id AND s.url_id = v.src_url_id
LEFT JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id
LEFT JOIN pages p ON p.run_id = v.run_id AND p.url_id = v.src_url_id",
            $joins,
            "
WHERE v.counted AND v.outcome IN ",
            $outcomes
        )
    };
}

/// A link whose outcome is `broken_url` or `broken_anchor`. An unresolved
/// xref carries the page ID `antora_hints` finds nearest to its refspec (D65).
const BROKEN: View = View {
    name: "report_broken",
    select: link_rows!(
        ",\n    h.hint",
        "\nLEFT JOIN antora_hints h ON h.run_id = v.run_id AND h.src_url_id = v.src_url_id
    AND h.xref_refspec = v.xref_refspec",
        "('broken_url', 'broken_anchor')"
    ),
};

/// A link whose outcome is `error`, apart from broken (D16).
const ERRORS: View = View {
    name: "report_errors",
    select: link_rows!("", "", "('error')"),
};

/// A page's counted links (D71) by kind, one row for each kind it links by. A
/// page with no links has no row; `bruce report stats` joins this to
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
WHERE counted
GROUP BY run_id, src_url_id, kind",
};

/// A link from one page to another, by the page it lands on, so a link to a
/// URL that redirected counts for the page it reached. A page's link to
/// itself is no row, and nor is a sitemap's, which only lists.
const INBOUND: View = View {
    name: "report_inbound",
    select: "\
SELECT v.run_id, v.src_url_id, coalesce(landing.url_id, v.target_url_id) AS url_id, v.kind
FROM link_verdicts v
LEFT JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id
LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url
WHERE v.kind NOT IN ('sitemap', 'sitemap_file')
    AND v.src_url_id <> coalesce(landing.url_id, v.target_url_id)",
};

/// A root of a run, and whether it read a sitemap: a file reached from the
/// root's page by `sitemap_file` links, an index's children included (D47),
/// answered `ok` with no reason. A file that failed, was blocked or ran past
/// the cap listed nothing. The root's page is the target whose URL is the
/// root's, `runs.roots` being canonical.
const ROOTS: View = View {
    name: "report_roots",
    select: "\
WITH RECURSIVE r AS (SELECT run_id, unnest(roots) AS root FROM runs),
files AS (SELECT run_id, src_url_id, target_url_id FROM links WHERE kind = 'sitemap_file'),
reached(run_id, root, url_id) AS (
    SELECT r.run_id, r.root, f.target_url_id
    FROM r
    JOIN targets s ON s.run_id = r.run_id AND s.url = r.root
    JOIN files f ON f.run_id = s.run_id AND f.src_url_id = s.url_id
    UNION
    SELECT d.run_id, d.root, f.target_url_id
    FROM reached d
    JOIN files f ON f.run_id = d.run_id AND f.src_url_id = d.url_id
)
SELECT r.run_id, r.root,
    EXISTS (SELECT 1 FROM reached d
        JOIN targets t ON t.run_id = d.run_id AND t.url_id = d.url_id
        WHERE d.run_id = r.run_id AND d.root = r.root
            AND t.outcome = 'ok' AND t.reason IS NULL) AS has_sitemap
FROM r",
};

/// The pages a sitemap report lists: every crawled page, and every sitemap
/// entry no page was fetched for (`via_links` is NULL for those, which have
/// no page row).
const SITEMAP: View = View {
    name: "report_sitemap",
    select: "\
SELECT pg.run_id, pg.url_id, pg.url AS page, pg.title, pg.in_sitemap, pg.via_links,
    t.outcome, t.reason, t.status, pg.source_repo, pg.source_path
FROM report_pages pg
JOIN targets t ON t.run_id = pg.run_id AND t.url_id = pg.url_id
WHERE pg.crawled
UNION ALL
SELECT DISTINCT e.run_id, e.target_url_id, t.url, NULL, true, NULL,
    t.outcome, t.reason, t.status, NULL, NULL
FROM links e
JOIN targets t ON t.run_id = e.run_id AND t.url_id = e.target_url_id
LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url
WHERE e.kind = 'sitemap' AND NOT EXISTS (SELECT 1 FROM pages p
    WHERE p.run_id = t.run_id AND p.url_id = coalesce(landing.url_id, t.url_id))",
};

/// A link whose target was redirected: where it landed, the hops it took
/// (a 3xx or a refresh, D7), the link's verdict `reason` (D70) and the page it
/// is on.
const REDIRECTS: View = View {
    name: "report_redirects",
    select: "\
SELECT v.run_id, v.src_url_id, s.url AS page, v.raw_href, t.url AS target, v.fragment, v.kind,
    t.final_url AS landing, v.reason,
    array_to_string(list_transform(t.redirect_chain,
        h -> CASE WHEN h.refresh THEN 'refresh' ELSE h.status::VARCHAR END || ' ' || h.url),
        ' > ') AS chain,
    p.source_repo, p.source_path
FROM link_verdicts v
JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id
LEFT JOIN targets s ON s.run_id = v.run_id AND s.url_id = v.src_url_id
LEFT JOIN pages p ON p.run_id = v.run_id AND p.url_id = v.src_url_id
WHERE v.counted AND t.final_url IS NOT NULL",
};
