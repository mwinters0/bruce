use duckdb::params;
use serde_json::Value;

use super::super::tests::{ROOT, column, fact, link, page, registry, target, url};
use super::document::utc;
use super::*;
use crate::report::{Cell, VIEWS};

const OTHER: &str = "https://other.test/";

/// An allowlisted host no root starts on.
const SLOW: &str = "https://slow.test/12";

/// 2026-10-23T20:04:11Z, the start of run 1.
const START: i64 = 1_792_785_851;

/// Run 1 is a finished crawl of two roots, `ROOT`, whose sitemap (101) lists
/// pages 1, 2 and 3 and two entries no page was fetched for, a 404 (20) and
/// a URL robots.txt forbade (21), and `OTHER`, which read none. Pages 1 (`r/one`,
/// component `handbook`), 2 (`r/two`, `other`), 3 (in the sitemap, linked from
/// nowhere), 4 (`ROOT`, in no sitemap) and 5 (`OTHER`) are crawled; 6 is
/// allowlisted. Page 1 links to a 404 (10), by image to another (11), to page
/// 2 at a missing fragment and a present one, to a 503 on the allowlisted
/// `SLOW` (12) and a skipped URL (13); page 2 to 10, to itself and to 15, a redirect to page 2; 4 to page 1;
/// and the allowlisted 6 to 10. Run 2 is a finished `check-dir` run, run 3 a
/// crawl that never finished.
pub(super) fn store() -> Store {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    conn.execute(
        "INSERT INTO runs VALUES (1, to_timestamp(?), to_timestamp(? + 4233), 'crawl', '0.0.9', \
         'h', NULL, [?, ?], [])",
        params![START, START, ROOT, OTHER],
    )
    .unwrap();
    for (run, mode, finished) in [(2, "local", true), (3, "crawl", false)] {
        conn.execute(
            "INSERT INTO runs VALUES (?, now(), CASE WHEN ? THEN now() END, ?, 'x', 'h', NULL, \
             [], [])",
            params![run, finished, mode],
        )
        .unwrap();
    }
    for (id, repo, root, listed) in [
        (1, Some("r/one"), Some(ROOT), true),
        (2, Some("r/two"), Some(ROOT), true),
        (3, None, Some(ROOT), true),
        (4, None, Some(ROOT), false),
        (5, None, Some(OTHER), false),
        (6, None, None, false),
    ] {
        page(&store, 1, id, repo, root);
        conn.execute(
            "UPDATE pages SET in_sitemap = ?, title = ?, source_url = ? WHERE url_id = ?",
            params![
                listed,
                format!("Page {id}"),
                repo.map(|r| format!("https://forge.test/{r}/{id}.adoc")),
                id
            ],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO anchors VALUES (1, 2, 'sec', 'h2', 'content')",
        [],
    )
    .unwrap();
    for (id, status, outcome, reason) in [
        (10, 404, "broken_url", None),
        (11, 404, "broken_url", None),
        (12, 503, "error", Some("server_error")),
        (13, 0, "unchecked", Some("skipped")),
        (15, 200, "ok", None),
        (20, 404, "broken_url", None),
        (21, 0, "unchecked", Some("robots")),
    ] {
        target(&store, 1, id, status, outcome, reason);
    }
    conn.execute(
        "UPDATE targets SET final_url = ? WHERE url_id = 15",
        params![url(2)],
    )
    .unwrap();
    conn.execute(
        "UPDATE targets SET url = ? WHERE url_id = 12",
        params![SLOW],
    )
    .unwrap();
    for (id, target_url) in [(100, ROOT), (101, "https://e.test/sitemap.xml")] {
        conn.execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, status, outcome) \
             VALUES ('live', 1, ?, ?, 200, 'ok')",
            params![id, target_url],
        )
        .unwrap();
    }
    link(&store, 1, 100, 101, None, "sitemap_file");
    for (src, to, fragment, kind) in [
        (1, 10, None, "anchor"),
        (1, 2, Some("nope"), "anchor"),
        (1, 2, Some("sec"), "anchor"),
        (1, 12, None, "anchor"),
        (1, 11, None, "image"),
        (1, 13, None, "anchor"),
        (2, 10, None, "anchor"),
        (2, 2, None, "anchor"),
        (2, 15, None, "anchor"),
        (4, 1, None, "anchor"),
        (6, 10, None, "anchor"),
        (101, 1, None, "sitemap"),
        (101, 2, None, "sitemap"),
        (101, 3, None, "sitemap"),
        (101, 20, None, "sitemap"),
        (101, 21, None, "sitemap"),
    ] {
        link(&store, 1, src, to, fragment, kind);
    }
    fact(&store, 1, 1, "antora.component", "handbook");
    fact(&store, 1, 2, "antora.component", "other");
    store
}

/// [`store`] with pages 2, 3 and 4 ranked by severity: page 3 holds two links
/// that errored, page 4 a broken URL and a missing fragment, page 2 a broken
/// URL; page 1 holds three red links and one amber.
pub(super) fn ranked() -> Store {
    let store = store();
    link(&store, 1, 3, 12, None, "anchor");
    link(&store, 1, 3, 12, None, "image");
    link(&store, 1, 4, 10, None, "anchor");
    link(&store, 1, 4, 2, Some("nope"), "anchor");
    store
}

/// [`store`] and run 4, a finished crawl of the same root that held two pages:
/// page 1, linking to the 404 (10), and page 2, with the anchor `nope` that
/// run 1 lost and page 1 links to.
pub(super) fn compared() -> Store {
    let store = store();
    let conn = store.connection();
    conn.execute(
        "INSERT INTO runs VALUES (4, to_timestamp(? - 86400), to_timestamp(? - 86400 + 60), \
         'crawl', '0.0.8', 'h', NULL, [?], [])",
        params![START, START, ROOT],
    )
    .unwrap();
    page(&store, 4, 1, Some("r/one"), Some(ROOT));
    page(&store, 4, 2, Some("r/two"), Some(ROOT));
    target(&store, 4, 10, 404, "broken_url", None);
    link(&store, 4, 1, 10, None, "anchor");
    conn.execute(
        "INSERT INTO anchors VALUES (4, 2, 'nope', 'h2', 'content'), (4, 2, 'sec', 'h2', 'content')",
        [],
    )
    .unwrap();
    store
}

/// [`compared`] and, in run 4, a link of page 1 to a 404 (17) that run 1's page
/// 1 does not hold: broken then, mended since.
pub(super) fn compared_with_a_fix() -> Store {
    let store = compared();
    target(&store, 4, 17, 404, "broken_url", None);
    link(&store, 4, 1, 17, None, "anchor");
    store
}

pub(super) fn of(store: &Store) -> Digest {
    digest(store, Some(RunId(1)), Against::Previous).unwrap()
}

/// [`store`] with page 5 linking to an unchecked target of each reason the
/// frontier gives, and twice to `forge.test`, an allowlisted host.
pub(super) fn with_unchecked() -> Store {
    let store = store();
    let conn = store.connection();
    for (id, at, outcome, reason, links) in [
        (
            30,
            "https://e.test/de/a",
            "unchecked",
            Some("out_of_scope"),
            2,
        ),
        (
            31,
            "https://e.test/de/b",
            "unchecked",
            Some("out_of_scope"),
            1,
        ),
        (
            32,
            "https://e.test/fr",
            "unchecked",
            Some("out_of_scope"),
            1,
        ),
        (
            33,
            "http://e.test/x/y",
            "unchecked",
            Some("out_of_scope"),
            1,
        ),
        (
            34,
            "https://github.test/a/b",
            "unchecked",
            Some("host_not_allowlisted"),
            2,
        ),
        (
            35,
            "https://github.test/c",
            "unchecked",
            Some("host_not_allowlisted"),
            1,
        ),
        (
            36,
            "https://spdx.test/l",
            "unchecked",
            Some("host_not_allowlisted"),
            1,
        ),
        (37, "mailto:a@e.test", "unchecked", Some("scheme"), 1),
        (
            38,
            "https://forge.test/o/r/issues/new?t=x",
            "unchecked",
            Some("skipped"),
            1,
        ),
        (
            39,
            "https://forge.test/user/login",
            "unchecked",
            Some("skipped"),
            1,
        ),
        (40, "https://forge.test/o/r", "ok", None, 2),
    ] {
        target(&store, 1, id, 0, outcome, reason);
        conn.execute(
            "UPDATE targets SET url = ? WHERE url_id = ?",
            params![at, id],
        )
        .unwrap();
        for n in 0..links {
            link(&store, 1, 5, id, Some(&format!("f{n}")), "anchor");
        }
    }
    store
}

#[test]
fn a_time_is_formatted_in_utc() {
    let at = |s| utc(s).1;
    assert_eq!(at(0), "1970-01-01T00:00:00Z");
    assert_eq!(at(-1), "1969-12-31T23:59:59Z");
    assert_eq!(at(START), "2026-10-23T20:04:11Z");
    assert_eq!(at(1_835_481_599), "2028-02-29T23:59:59Z");
    assert_eq!(utc(START).0, "2026-10-23");
}

#[test]
fn the_digest_is_named_for_the_day_the_run_started() {
    let digest = of(&store());
    assert_eq!(digest.date(), "2026-10-23");
    assert_eq!(digest.run.id, RunId(1));
    assert_eq!(digest.run.finished, Some(START + 4233));
    assert_eq!(digest.run.roots, [ROOT, OTHER]);
    assert_eq!(digest.run.tool_version, "0.0.9");
}

#[test]
fn the_totals_count_every_kind_on_the_pages_a_root_covers() {
    let t = of(&store()).totals;
    // Page 6 is allowlisted, and the sitemap's links are on no page.
    assert_eq!(
        (
            t.pages,
            t.links,
            t.broken_url,
            t.broken_anchor,
            t.error,
            t.unchecked
        ),
        (5, 10, 3, 1, 1, 1)
    );
}

#[test]
fn pages_are_ranked_by_their_broken_links_and_their_links_follow_them() {
    let d = of(&store());
    assert_eq!(column(&d.pages, "page"), [url(1), url(2)]);
    assert_eq!(
        d.pages.rows[0][5..],
        [
            Cell::Int(6),
            Cell::Int(2),
            Cell::Int(1),
            Cell::Int(1),
            Cell::Int(0)
        ]
    );
    assert_eq!(
        column(&d.pages, "source_url")[1],
        "https://forge.test/r/two/2.adoc"
    );
    assert_eq!(
        column(&d.broken, "raw_href"),
        ["1>10#", "1>11#", "1>2#nope", "2>10#"]
    );
    assert_eq!(
        column(&d.broken, "kind"),
        ["anchor", "image", "anchor", "anchor"]
    );
}

#[test]
fn pages_and_repos_are_ranked_by_their_red_links_then_their_amber_ones() {
    let d = of(&ranked());
    // Red is `broken_url` and `error`: page 3's two errors outrank page 4, whose
    // amber link outranks page 2 though its URL sorts after.
    assert_eq!(column(&d.pages, "page"), [url(1), url(3), url(4), url(2)]);
    // Pages 3 and 4 hold no repo: 3 red and 1 amber, as r/one's page 1 does, and
    // the row of no repo sorts last on the name.
    assert_eq!(column(&d.by_repo, "repo"), ["r/one", "Null", "r/two"]);
    assert_eq!(d.by_repo.rows[1][2], Cell::Int(2));
}

#[test]
fn a_broken_target_carries_the_source_of_the_page_it_names() {
    let d = of(&store());
    let t = &d.broken_targets;
    assert_eq!(column(t, "target"), [url(10), url(11), url(2)]);
    assert_eq!(column(t, "fragment")[2], "nope");
    assert_eq!(t.rows[0][7..9], [Cell::Int(2), Cell::Int(2)]);
    assert_eq!(column(t, "outcome")[2], "broken_anchor");
    // A missing anchor is the target page's repo to restore.
    assert_eq!(column(t, "source_repo")[2], "r/two");
    assert_eq!(column(t, "source_repo")[0], "Null");
}

#[test]
fn broken_targets_are_ranked_by_their_links_then_their_pages() {
    let store = store();
    // Target 11 now has four links on one page, and `2#nope` two on one.
    for (to, fragment) in [(11, None), (11, None), (11, None), (2, Some("nope"))] {
        link(&store, 1, 1, to, fragment, "anchor");
    }
    let t = of(&store).broken_targets;
    assert_eq!(column(&t, "target"), [url(11), url(10), url(2)]);
    assert_eq!(t.rows[0][7..9], [Cell::Int(4), Cell::Int(1)]);
    // 10 has two links on two pages, `2#nope` two on one: the pages break the tie.
    assert_eq!(t.rows[1][7..9], [Cell::Int(2), Cell::Int(2)]);
    assert_eq!(t.rows[2][7..9], [Cell::Int(2), Cell::Int(1)]);
}

#[test]
fn repos_and_components_are_ranked_and_a_page_without_either_is_kept() {
    let d = of(&store());
    assert_eq!(column(&d.by_repo, "repo"), ["r/one", "r/two", "Null"]);
    assert_eq!(
        d.by_repo.rows[0][1..],
        [
            Cell::Int(1),
            Cell::Int(1),
            Cell::Int(6),
            Cell::Int(2),
            Cell::Int(1),
            Cell::Int(1),
            Cell::Int(0)
        ]
    );
    assert_eq!(d.by_repo.rows[2][1..3], [Cell::Int(3), Cell::Int(0)]);
    assert_eq!(column(&d.by_component, "component"), ["handbook", "other"]);
}

/// A link from `src` to `to` written as `href`.
fn written(store: &Store, src: u64, to: u64, href: &str) {
    store
        .connection()
        .execute(
            "INSERT INTO links VALUES (1, ?, ?, NULL, ?, 'anchor', NULL)",
            params![src, to, href],
        )
        .unwrap();
}

/// [`store`] with a page (7) linking to `cc`, a 302 to `cc//`, which is page
/// 51; 51 links relatively to the doubled-slash page 52 (`cc//a/`) and to a
/// 404, and absolutely to another; 52 links relatively to a 404 and back to
/// 51. Page 60 doubles a slash too, and page 61 is its twin.
pub(super) fn with_doubled_slashes() -> Store {
    let store = store();
    let conn = store.connection();
    for (id, at) in [
        (7, "https://e.test/seven"),
        (51, "https://e.test/cc//"),
        (52, "https://e.test/cc//a/"),
        (60, "https://e.test/tw//x"),
        (61, "https://e.test/tw/x"),
    ] {
        page(&store, 1, id, Some("r/seven"), Some(ROOT));
        conn.execute(
            "UPDATE targets SET url = ? WHERE url_id = ?",
            params![at, id],
        )
        .unwrap();
    }
    target(&store, 1, 50, 302, "ok", None);
    conn.execute(
        "UPDATE targets SET url = 'https://e.test/cc', final_url = 'https://e.test/cc//' \
         WHERE url_id = 50",
        [],
    )
    .unwrap();
    for id in [53, 54, 55] {
        target(&store, 1, id, 404, "broken_url", None);
    }
    for (src, to, href) in [
        (7, 50, "https://e.test/cc"),
        (51, 52, "a/"),
        (51, 53, "gone"),
        (51, 54, "https://e.test/abs"),
        (52, 55, "gone2"),
        (52, 51, "../"),
        (60, 54, "https://e.test/abs"),
        (61, 1, "https://e.test/1"),
    ] {
        written(&store, src, to, href);
    }
    store
}

#[test]
fn the_links_broken_through_a_doubled_slash_count_on_the_linking_page() {
    let d = of(&with_doubled_slashes());
    // Page 7's one link reaches 51 by the redirect, then 52, whose relative
    // links, and 51's, break: 51's `a/` and `gone`, 52's `gone2` and `../`.
    // They rank with page 7's red: 1 and 4 against page 1's 3.
    assert_eq!(
        column(&d.pages, "page")[..2],
        ["https://e.test/seven".to_owned(), url(1)]
    );
    let row = |page: &str, name: &str| {
        let at = column(&d.pages, "page")
            .iter()
            .position(|p| p == page)
            .unwrap();
        let col = d.pages.columns.iter().position(|c| c == name).unwrap();
        d.pages.rows[at][col].clone()
    };
    assert_eq!(row("https://e.test/seven", "broken_url"), Cell::Int(1));
    assert_eq!(row("https://e.test/seven", "through"), Cell::Int(4));
    // Page 51 reached only in its doubled form counts its absolute link, and
    // the relative ones it holds are page 7's; 52 holds only those; 60 has a
    // twin, which holds its links.
    assert_eq!(row("https://e.test/cc//", "broken_url"), Cell::Int(1));
    assert_eq!(row("https://e.test/cc//", "through"), Cell::Int(0));
    let pages = column(&d.pages, "page");
    assert!(
        !pages.contains(&"https://e.test/cc//a/".to_owned()),
        "{pages:?}"
    );
    assert!(
        !pages.contains(&"https://e.test/tw//x".to_owned()),
        "{pages:?}"
    );
    let broken = column(&d.broken, "raw_href");
    assert!(!broken.contains(&"gone".to_owned()) && !broken.contains(&"gone2".to_owned()));
    assert_eq!(
        broken.iter().filter(|h| *h == "https://e.test/abs").count(),
        1
    );
    // The repo adds its pages': 1 + 1 red and 4 through.
    let repo = d
        .by_repo
        .rows
        .iter()
        .find(|r| r[0] == Cell::Text("r/seven".to_owned()))
        .unwrap();
    assert_eq!(repo.last(), Some(&Cell::Int(4)));
    assert_eq!(
        d.totals.links,
        10 + 1 + 1 + 1,
        "the uncounted links are none"
    );
}

#[test]
fn a_doubled_slash_a_redirect_lands_on_is_the_links_landing() {
    let store = with_doubled_slashes();
    // A link naming a doubled-slash URL itself has none: the URL shows it.
    target(&store, 1, 70, 200, "ok", None);
    store
        .connection()
        .execute(
            "UPDATE targets SET url = 'https://e.test/own//x' WHERE url_id = 70",
            [],
        )
        .unwrap();
    written(&store, 7, 70, "https://e.test/own//x");
    let d = of(&store);
    let landing = |t: &crate::report::table::Table, target: &str| {
        let at = column(t, "target")
            .iter()
            .position(|u| u == target)
            .unwrap_or_else(|| panic!("{target} in {:?}", column(t, "target")));
        t.rows[at][t.columns.iter().position(|c| c == "landing").unwrap()].clone()
    };
    let cc = Cell::Text("https://e.test/cc//".to_owned());
    assert_eq!(landing(&d.broken, "https://e.test/cc"), cc);
    assert_eq!(landing(&d.broken_targets, "https://e.test/cc"), cc);
    assert_eq!(landing(&d.broken, "https://e.test/own//x"), Cell::Null);
    assert_eq!(
        landing(&d.broken_targets, "https://e.test/own//x"),
        Cell::Null
    );
    // A target broken for another reason has none.
    assert_eq!(landing(&d.broken, &url(54)), Cell::Null);
}

#[test]
fn a_wrong_trailing_slash_names_the_url_that_answers_as_the_fix() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE targets SET reason = 'wrong_trailing_slash' WHERE url_id = 10",
            [],
        )
        .unwrap();
    store
        .connection()
        .execute(
            "UPDATE targets SET url = ? || '/' WHERE url_id = 11",
            [url(11)],
        )
        .unwrap();
    store
        .connection()
        .execute(
            "UPDATE targets SET reason = 'wrong_trailing_slash' WHERE url_id = 11",
            [],
        )
        .unwrap();
    let d = of(&store);
    let fix = |t: &crate::report::table::Table, target: &str| {
        let at = column(t, "target")
            .iter()
            .position(|u| u == target)
            .unwrap_or_else(|| panic!("{target} in {:?}", column(t, "target")));
        let columns = &t.columns;
        let position = |name: &str| columns.iter().position(|c| c == name).unwrap();
        assert_eq!(position("fix"), position("landing") + 1);
        t.rows[at][position("fix")].clone()
    };
    let ten = Cell::Text(format!("{}/", url(10)));
    let eleven = Cell::Text(url(11));
    assert_eq!(fix(&d.broken, &url(10)), ten);
    assert_eq!(fix(&d.broken_targets, &url(10)), ten);
    assert_eq!(fix(&d.broken, &format!("{}/", url(11))), eleven);
    assert_eq!(fix(&d.broken_targets, &format!("{}/", url(11))), eleven);
    // A target broken for another reason has none.
    assert_eq!(fix(&d.broken, &url(2)), Cell::Null);
}

#[test]
fn a_redirect_onto_a_wrong_trailing_slash_names_the_landing_toggled_as_the_fix() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE targets SET url = 'https://e.test/old', final_url = 'https://e.test/new/', \
             reason = 'wrong_trailing_slash' WHERE url_id = 10",
            [],
        )
        .unwrap();
    let d = of(&store);
    let cell = |t: &crate::report::table::Table, name: &str| {
        let at = column(t, "target")
            .iter()
            .position(|u| u == "https://e.test/old")
            .unwrap_or_else(|| panic!("in {:?}", column(t, "target")));
        t.rows[at][t.columns.iter().position(|c| c == name).unwrap()].clone()
    };
    for t in [&d.broken, &d.broken_targets] {
        assert_eq!(cell(t, "landing"), Cell::Text("https://e.test/new/".into()));
        assert_eq!(cell(t, "fix"), Cell::Text("https://e.test/new".into()));
    }
}

/// The count in column `name` of `page`'s row of the pages section.
fn pages_count(d: &Digest, page: &str, name: &str) -> Cell {
    let at = column(&d.pages, "page")
        .iter()
        .position(|p| p == page)
        .unwrap();
    let col = d.pages.columns.iter().position(|c| c == name).unwrap();
    d.pages.rows[at][col].clone()
}

#[test]
fn a_link_broken_on_the_twin_too_is_not_broken_through_it_and_a_query_link_is() {
    let store = with_doubled_slashes();
    let conn = store.connection();
    target(&store, 1, 62, 404, "broken_url", None);
    conn.execute(
        "UPDATE targets SET url = 'https://e.test/tw//x?q=1' WHERE url_id = 62",
        [],
    )
    .unwrap();
    for (src, to, href) in [
        // Page 7 now names page 60, which has its twin, 61.
        (7, 60, "https://e.test/tw//x"),
        // Broken on both: the twin's to count.
        (60, 53, "gone-both"),
        (61, 53, "gone-both"),
        // Broken here only: the linking page's.
        (60, 53, "gone-here"),
        (61, 1, "gone-here"),
        // A query keeps the doubled path, so it breaks here alone.
        (60, 62, "?q=1"),
        (61, 1, "?q=1"),
    ] {
        written(&store, src, to, href);
    }
    let d = of(&store);
    let seven = "https://e.test/seven";
    // The four through the redirect to 51, and `gone-here` and `?q=1`.
    assert_eq!(pages_count(&d, seven, "through"), Cell::Int(6));
    assert_eq!(pages_count(&d, seven, "broken_url"), Cell::Int(2));
    assert_eq!(
        pages_count(&d, "https://e.test/tw/x", "broken_url"),
        Cell::Int(1),
        "the twin counts the link broken on both"
    );
}

#[test]
fn errors_are_apart_from_breakage_and_name_the_pages_source() {
    let d = of(&store());
    assert_eq!(column(&d.errors, "raw_href"), ["1>12#"]);
    assert_eq!(column(&d.errors, "reason"), ["server_error"]);
    assert_eq!(
        column(&d.errors, "source_url"),
        ["https://forge.test/r/one/1.adoc"]
    );
    assert_eq!(column(&d.unchecked, "reason"), ["skipped"]);
}

#[test]
fn the_unchecked_links_are_grouped_by_where_they_point() {
    let d = of(&with_unchecked());
    let g = &d.unchecked_groups;
    assert_eq!(g.columns, ["reason", "group", "links", "targets"]);
    // The reasons in `unchecked`'s order, the most links first; on a root's
    // host the URL to the path's first segment, for `skipped` the host and
    // the path's last two segments, a scheme for a URL that is not `http(s)`,
    // and elsewhere the host.
    assert_eq!(
        column(&d.unchecked, "reason"),
        [
            "out_of_scope",
            "host_not_allowlisted",
            "skipped",
            "no_anchors",
            "scheme"
        ]
    );
    let rows: Vec<(String, String, String, String)> = g
        .rows
        .iter()
        .map(|r| (shown(&r[0]), shown(&r[1]), shown(&r[2]), shown(&r[3])))
        .collect();
    let want = [
        ("out_of_scope", "https://e.test/de/", "3", "2"),
        ("out_of_scope", "http://e.test/x/", "1", "1"),
        ("out_of_scope", "https://e.test/fr", "1", "1"),
        ("host_not_allowlisted", "github.test", "3", "2"),
        ("host_not_allowlisted", "spdx.test", "1", "1"),
        ("skipped", "e.test/13", "1", "1"),
        ("skipped", "forge.test/user/login", "1", "1"),
        ("skipped", "forge.test/\u{2026}/issues/new", "1", "1"),
        ("no_anchors", "forge.test", "2", "1"),
        ("scheme", "mailto:", "1", "1"),
    ];
    let want: Vec<(String, String, String, String)> = want
        .iter()
        .map(|(a, b, c, d)| (a.to_string(), b.to_string(), c.to_string(), d.to_string()))
        .collect();
    assert_eq!(rows, want);
}

#[test]
fn the_external_hosts_checked_are_those_whose_links_were_asked() {
    // forge.test's skipped links are no check of it, its asked one is; e.test
    // is a root's host.
    let d = of(&with_unchecked());
    assert_eq!(d.checked_hosts.columns, ["host", "links", "targets"]);
    assert_eq!(
        column(&d.checked_hosts, "host"),
        ["forge.test", "slow.test"]
    );
    let counts: Vec<String> = d
        .checked_hosts
        .rows
        .iter()
        .map(|r| format!("{} {}", shown(&r[1]), shown(&r[2])))
        .collect();
    assert_eq!(counts, ["2 1", "1 1"]);
}

/// A cell as text, a number as its digits.
fn shown(cell: &Cell) -> String {
    match cell {
        Cell::Int(n) => n.to_string(),
        other => other.text().unwrap_or("").to_owned(),
    }
}

#[test]
fn housekeeping_holds_the_redirects_orphans_and_unlisted_pages() {
    let d = of(&store());
    assert_eq!(column(&d.redirects, "landing"), [url(2)]);
    assert_eq!(
        d.redirects.rows[0][5],
        Cell::List(vec![url(2)]),
        "the pages whose links they are"
    );
    assert_eq!(column(&d.orphans, "page"), [url(3)]);
    assert_eq!(column(&d.unlisted, "page"), [url(4)]);
    // The entries no page was fetched for, the one that failed first; the
    // sitemap's pages are in no list of entries.
    let entries = &d.sitemap_entries;
    assert_eq!(entries.columns, ["page", "outcome", "reason", "status"]);
    assert_eq!(column(entries, "page"), [url(20), url(21)]);
    assert_eq!(column(entries, "outcome"), ["broken_url", "unchecked"]);
    assert_eq!(column(entries, "reason")[1], "robots");
    assert_eq!(entries.rows[0][3], Cell::Int(404));
    assert_eq!(d.roots_without_sitemap, [OTHER]);
}

#[test]
fn a_run_is_named_and_a_check_dir_run_has_no_digest() {
    let store = store();
    assert_eq!(
        digest(&store, Some(RunId(1)), Against::Previous)
            .unwrap()
            .run
            .id,
        RunId(1)
    );
    let d = digest(&store, Some(RunId(3)), Against::Previous).unwrap();
    assert!(d.unfinished && d.run.finished.is_none());
    assert!(matches!(
        digest(&store, Some(RunId(2)), Against::Previous),
        Err(Error::NotCrawl(RunId(2)))
    ));
    assert!(matches!(
        digest(&store, Some(RunId(9)), Against::Previous),
        Err(Error::NoSuchRun(RunId(9)))
    ));
    // The default is the latest finished run, which here is the `check-dir` one.
    assert!(matches!(
        digest(&store, None, Against::Previous),
        Err(Error::NotCrawl(RunId(2)))
    ));
}

#[test]
fn the_default_names_a_later_unfinished_run() {
    let store = store();
    store
        .connection()
        .execute("DELETE FROM runs WHERE run_id = 2", [])
        .unwrap();
    let d = digest(&store, None, Against::Previous).unwrap();
    assert_eq!((d.run.id, d.later_unfinished), (RunId(1), Some(RunId(3))));
}

#[test]
fn a_run_with_no_pages_has_empty_sections() {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    let id = store
        .begin_run(crate::store::Mode::Crawl, "h", None, &[])
        .unwrap();
    store.finish_run(id).unwrap();
    let d = digest(&store, None, Against::Previous).unwrap();
    assert_eq!((d.totals.pages, d.totals.links), (0, 0));
    assert!(d.pages.rows.is_empty() && d.by_repo.rows.is_empty());
    let json: Value = serde_json::from_str(&d.json()).unwrap();
    assert_eq!(json["pages"], Value::Array(Vec::new()));
}

#[test]
fn the_json_is_valid_and_carries_its_version_and_context() {
    let json: Value = serde_json::from_str(&of(&store()).json()).unwrap();
    assert_eq!(json["format_version"], FORMAT_VERSION);
    assert_eq!(json["tool_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(json["compared_with"], Value::Null);
    let run = &json["run"];
    assert_eq!(run["started_at"], "2026-10-23T20:04:11Z");
    assert_eq!(run["finished_at"], "2026-10-23T21:14:44Z");
    assert_eq!(run["duration_seconds"], 4233);
    assert_eq!(run["roots"], serde_json::json!([ROOT, OTHER]));
    assert_eq!(json["totals"]["links"], 10);
    assert_eq!(json["pages"][0]["page"], url(1));
    assert_eq!(json["broken"].as_array().unwrap().len(), 4);
    assert_eq!(json["housekeeping"]["unlisted"][0]["page"], url(4));
    assert_eq!(json["housekeeping"]["sitemap_entries"][0]["page"], url(20));
}

#[test]
fn an_unfinished_runs_digest_has_no_finish() {
    let store = store();
    let json: Value = serde_json::from_str(
        &digest(&store, Some(RunId(3)), Against::Previous)
            .unwrap()
            .json(),
    )
    .unwrap();
    assert_eq!(json["run"]["finished_at"], Value::Null);
    assert_eq!(json["run"]["duration_seconds"], Value::Null);
}

/// The first fenced `json` block of the manual's page for the digest, whose
/// text begins with the digest's own `format_version`.
fn documented() -> Value {
    let manual = include_str!("../../../docs/manual/digest.md");
    let start = manual
        .find("```json\n{\n  \"format_version\"")
        .expect("the manual shows the digest's shape")
        + "```json\n".len();
    let end = start + manual[start..].find("\n```").unwrap();
    serde_json::from_str(&manual[start..end]).expect("the manual's example is JSON")
}

/// Fails unless `actual` has the fields `documented` shows, no more and no
/// fewer, at every depth. A list is shown by one element, which each of
/// `actual`'s must match; a `null` on either side matches any value, being
/// what a nullable field holds.
fn same_shape(documented: &Value, actual: &Value, at: &str) {
    match (documented, actual) {
        (Value::Null, _) | (_, Value::Null) => {}
        (Value::Object(want), Value::Object(got)) => {
            let names = |m: &serde_json::Map<String, Value>| {
                let mut names: Vec<String> = m.keys().cloned().collect();
                names.sort();
                names
            };
            assert_eq!(names(want), names(got), "the fields of {at}");
            for (name, value) in want {
                same_shape(value, &got[name], &format!("{at}.{name}"));
            }
        }
        (Value::Array(want), Value::Array(got)) => {
            assert_eq!(want.is_empty(), got.is_empty(), "{at} is listed or empty");
            for item in got {
                same_shape(&want[0], item, &format!("{at}[]"));
            }
        }
        (want, got) => assert_eq!(
            std::mem::discriminant(want),
            std::mem::discriminant(got),
            "the type of {at}: {want} against {got}"
        ),
    }
}

#[test]
fn the_json_has_the_fields_the_manual_documents() {
    let documented = documented();
    assert_eq!(documented["format_version"], FORMAT_VERSION);
    // A comparison fills `compared_with`, the breakage new and fixed and
    // `anchors_removed`, which a digest
    // of one run leaves null and empty.
    let compared = digest(
        &compared_with_a_fix(),
        Some(RunId(1)),
        Against::Run(RunId(4)),
    )
    .unwrap();
    let json: Value = serde_json::from_str(&compared.json()).unwrap();
    same_shape(&documented, &json, "$");
}
