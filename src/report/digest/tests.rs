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

pub(super) fn of(store: &Store) -> Digest {
    digest(store, Some(RunId(1))).unwrap()
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
        [Cell::Int(6), Cell::Int(2), Cell::Int(1), Cell::Int(1)]
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
    assert_eq!(t.rows[0][5..7], [Cell::Int(2), Cell::Int(2)]);
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
    assert_eq!(t.rows[0][5..7], [Cell::Int(4), Cell::Int(1)]);
    // 10 has two links on two pages, `2#nope` two on one: the pages break the tie.
    assert_eq!(t.rows[1][5..7], [Cell::Int(2), Cell::Int(2)]);
    assert_eq!(t.rows[2][5..7], [Cell::Int(2), Cell::Int(1)]);
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
            Cell::Int(1)
        ]
    );
    assert_eq!(d.by_repo.rows[2][1..3], [Cell::Int(3), Cell::Int(0)]);
    assert_eq!(column(&d.by_component, "component"), ["handbook", "other"]);
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
    assert_eq!(digest(&store, Some(RunId(1))).unwrap().run.id, RunId(1));
    let d = digest(&store, Some(RunId(3))).unwrap();
    assert!(d.unfinished && d.run.finished.is_none());
    assert!(matches!(
        digest(&store, Some(RunId(2))),
        Err(Error::NotCrawl(RunId(2)))
    ));
    assert!(matches!(
        digest(&store, Some(RunId(9))),
        Err(Error::NoSuchRun(RunId(9)))
    ));
    // The default is the latest finished run, which here is the `check-dir` one.
    assert!(matches!(
        digest(&store, None),
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
    let d = digest(&store, None).unwrap();
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
    let d = digest(&store, None).unwrap();
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
    let json: Value =
        serde_json::from_str(&digest(&store, Some(RunId(3))).unwrap().json()).unwrap();
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
    let json: Value = serde_json::from_str(&of(&store()).json()).unwrap();
    same_shape(&documented, &json, "$");
}
