use duckdb::params;

use super::{CAUSES, fixed_breakage, new_breakage};
use crate::report::VIEWS;
use crate::report::table::Table;
use crate::report::tests::{ROOT, column, link, page, registry, target, url};
use crate::store::{RunId, Store};

/// Run 4, the comparison, and run 1, the digest's: pages 1 (`r/one`) and 2
/// (`r/two`) in both, page 5 in run 1 only and page 6 in run 4 only. Page 1
/// links to a target of each kind of change, page 2 to the 404 (40) that page
/// 5 links to:
///
/// - 30 answered in run 4 and is a 404 now; page 30 was `r/thirty`'s;
/// - 31 held `#frag` in run 4 and redirects to a page without it now;
/// - page 2 held `#gone` in run 4;
/// - 41 answered `ok` in run 4 and its URL doubles a slash now;
/// - 42 errored in run 4 and is a 404 now;
/// - 10 is a 404 in both, and 52 errors now where it was a 404, 53 is gone
///   from page 1, 50 answers now where it was a 404, 51 likewise on page 6.
fn compared() -> Store {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    for run in [4, 1] {
        conn.execute(
            "INSERT INTO runs VALUES (?, now(), now(), 'crawl', 'x', 'h', NULL, [?], [])",
            params![run, ROOT],
        )
        .unwrap();
    }
    for (run, id, repo) in [
        (4, 1, "r/one"),
        (4, 2, "r/two"),
        (1, 1, "r/one"),
        (1, 2, "r/two"),
        (1, 5, "r/five"),
        (4, 6, "r/six"),
    ] {
        page(&store, run, id, Some(repo), Some(ROOT));
    }
    // Pages the links name, as the comparison recorded them.
    page(&store, 4, 30, Some("r/thirty"), None);
    page(&store, 4, 31, Some("r/thirtyone"), None);
    page(&store, 1, 33, Some("r/thirtythree"), None);
    page(&store, 4, 41, None, None);
    conn.execute(
        "INSERT INTO anchors VALUES (4, 2, 'gone', 'h2', 'content'), \
         (4, 31, 'frag', 'h2', 'content')",
        [],
    )
    .unwrap();
    for (run, id, status, outcome, reason) in [
        (4, 10, 404, "broken_url", None),
        (1, 10, 404, "broken_url", None),
        (1, 30, 404, "broken_url", None),
        (1, 40, 404, "broken_url", None),
        (4, 42, 503, "error", Some("server_error")),
        (1, 42, 404, "broken_url", None),
        (4, 50, 404, "broken_url", None),
        (1, 50, 200, "ok", None),
        (4, 51, 404, "broken_url", None),
        (4, 52, 404, "broken_url", None),
        (1, 52, 503, "error", Some("server_error")),
        (4, 53, 404, "broken_url", None),
    ] {
        target(&store, run, id, status, outcome, reason);
    }
    // Page 31 redirects to page 33 now, and 41 is spelt with a doubled slash.
    target(&store, 1, 31, 301, "ok", None);
    conn.execute(
        "UPDATE targets SET final_url = ? WHERE run_id = 1 AND url_id = 31",
        params![url(33)],
    )
    .unwrap();
    target(&store, 1, 41, 200, "ok", None);
    conn.execute(
        "UPDATE targets SET url = 'https://e.test//41' WHERE run_id = 1 AND url_id = 41",
        [],
    )
    .unwrap();
    for (run, src, to, fragment) in [
        (4, 1, 10, None),
        (1, 1, 10, None),
        (4, 1, 30, None),
        (1, 1, 30, None),
        (4, 1, 31, Some("frag")),
        (1, 1, 31, Some("frag")),
        (4, 1, 2, Some("gone")),
        (1, 1, 2, Some("gone")),
        (1, 2, 40, None),
        (1, 5, 40, None),
        (4, 1, 41, None),
        (1, 1, 41, None),
        (4, 1, 42, None),
        (1, 1, 42, None),
        (4, 1, 50, None),
        (1, 1, 50, None),
        (4, 6, 51, None),
        (4, 1, 52, None),
        (1, 1, 52, None),
        (4, 1, 53, None),
    ] {
        link(&store, run, src, to, fragment, "anchor");
    }
    store
}

fn new(store: &Store) -> Table {
    new_breakage(store.connection(), RunId(4), RunId(1)).unwrap()
}

/// `column` of the row whose `page` and `raw_href` are those of the link from
/// `src` to `to` at `fragment`.
fn of(t: &Table, src: u64, to: u64, fragment: Option<&str>, name: &str) -> String {
    let href = format!("{src}>{to}#{}", fragment.unwrap_or(""));
    let hrefs = column(t, "raw_href");
    let at = hrefs
        .iter()
        .position(|h| *h == href)
        .unwrap_or_else(|| panic!("no row for {href} in {hrefs:?}"));
    column(t, name).swap_remove(at)
}

fn text(s: impl std::fmt::Display) -> String {
    s.to_string()
}

#[test]
fn each_new_break_says_what_the_comparison_held() {
    let t = new(&compared());
    let cause = |src, to, fragment| of(&t, src, to, fragment, "cause");
    assert_eq!(cause(1, 30, None), text("target_removed"));
    assert_eq!(cause(1, 31, Some("frag")), text("target_moved"));
    assert_eq!(cause(1, 2, Some("gone")), text("anchor_removed"));
    assert_eq!(cause(2, 40, None), text("link_added"));
    assert_eq!(cause(5, 40, None), text("new_page"));
    assert_eq!(cause(1, 41, None), text("unexplained"));
    assert_eq!(cause(1, 42, None), text("first_judged"));
    assert_eq!(t.rows.len(), 7, "{t:?}");
}

#[test]
fn a_link_broken_in_both_runs_is_no_new_break() {
    let t = new(&compared());
    assert!(!column(&t, "raw_href").contains(&"1>10#".to_owned()));
}

#[test]
fn the_causes_are_listed_in_the_order_they_are_read() {
    let t = new(&compared());
    let found = column(&t, "cause");
    let ranks: Vec<usize> = found
        .iter()
        .map(|c| CAUSES.iter().position(|k| *c == text(k)).unwrap())
        .collect();
    assert!(ranks.is_sorted(), "{found:?}");
}

#[test]
fn the_source_shown_is_the_targets_where_the_target_changed_else_the_linking_pages() {
    let t = new(&compared());
    let repo = |src, to, fragment| of(&t, src, to, fragment, "source_repo");
    // The target as the comparison recorded it, not as it is now (no page).
    assert_eq!(repo(1, 30, None), text("r/thirty"));
    assert_eq!(repo(1, 31, Some("frag")), text("r/thirtyone"));
    assert_eq!(repo(1, 2, Some("gone")), text("r/two"));
    // The linking page.
    assert_eq!(repo(2, 40, None), text("r/two"));
    assert_eq!(repo(5, 40, None), text("r/five"));
    assert_eq!(repo(1, 42, None), text("r/one"));
    assert_eq!(of(&t, 1, 30, None, "source_path"), text("30.adoc"), "{t:?}");
}

#[test]
fn a_link_fixed_says_what_it_became() {
    let store = compared();
    let t = fixed_breakage(store.connection(), RunId(4), RunId(1)).unwrap();
    let became = |src, to| of(&t, src, to, None, "became");
    assert_eq!(became(1, 50), text("ok"));
    assert_eq!(became(1, 53), text("link_removed"));
    assert_eq!(became(6, 51), text("page_removed"));
    assert_eq!(became(1, 52), text("error"));
    // 1>10 is broken in both; 1>30 and 1>31 were ok.
    assert_eq!(t.rows.len(), 4, "{t:?}");
    assert_eq!(of(&t, 1, 50, None, "source_repo"), text("r/one"));
    assert_eq!(of(&t, 6, 51, None, "source_repo"), text("r/six"));
}

#[test]
fn a_run_compared_with_itself_has_neither_new_nor_fixed_breakage() {
    let store = compared();
    let conn = store.connection();
    assert!(
        new_breakage(conn, RunId(1), RunId(1))
            .unwrap()
            .rows
            .is_empty()
    );
    assert!(
        fixed_breakage(conn, RunId(1), RunId(1))
            .unwrap()
            .rows
            .is_empty()
    );
}
