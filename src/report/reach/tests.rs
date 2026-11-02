use duckdb::params;

use super::super::tests::{ROOT, column, link, page, registry, target, url};
use super::*;
use crate::extract::Kind;
use crate::report::{Cell, Report, Store, VIEWS, run};

const OTHER: &str = "https://other.test/";

/// Run 1, finished, with roots `ROOT`, which read a sitemap, and `OTHER`,
/// which read none. The sitemap file (101) lists pages 1, 2, 3 and 7, the
/// redirect 20 (to page 3) and 30, which robots.txt forbade. Pages 1 to 4
/// and 7 are in `ROOT`, 5 in `OTHER`, 6 is allowlisted; 4 and 5 are in no
/// sitemap. Page 1 links to 2 and, by image, to 3; 3 to 2; 2 to itself;
/// 4 to 1 and, through the redirect 20, to 3.
fn store() -> Store {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    conn.execute(
        "INSERT INTO runs VALUES (1, now(), now(), 'crawl', 'x', 'h', NULL, ?, [])",
        params![format!("['{ROOT}', '{OTHER}']")],
    )
    .unwrap();
    for (id, repo, root) in [
        (1, Some("r/one"), ROOT),
        (2, None, ROOT),
        (3, None, ROOT),
        (4, None, ROOT),
        (5, None, OTHER),
        (7, None, ROOT),
    ] {
        page(&store, 1, id, repo, Some(root));
        conn.execute(
            "UPDATE pages SET title = ? WHERE url_id = ?",
            params![format!("Page {id}"), id],
        )
        .unwrap();
    }
    page(&store, 1, 6, None, None);
    target(&store, 1, 20, 200, "ok", None);
    conn.execute(
        "UPDATE targets SET final_url = ? WHERE url_id = 20",
        params![url(3)],
    )
    .unwrap();
    target(&store, 1, 30, 0, "unchecked", Some("robots"));
    for (id, target_url) in [(100, ROOT), (101, "https://e.test/sitemap.xml")] {
        conn.execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, status, outcome) \
             VALUES ('live', 1, ?, ?, 200, 'ok')",
            params![id, target_url],
        )
        .unwrap();
    }
    link(&store, 1, 100, 101, None, "sitemap_file");
    for entry in [1, 2, 3, 7, 20, 30] {
        link(&store, 1, 101, entry, None, "sitemap");
    }
    link(&store, 1, 1, 2, None, "anchor");
    link(&store, 1, 1, 3, None, "image");
    link(&store, 1, 3, 2, None, "anchor");
    link(&store, 1, 2, 2, None, "anchor");
    link(&store, 1, 4, 1, None, "anchor");
    link(&store, 1, 4, 20, None, "anchor");
    store
}

fn report(store: &Store, which: Report, edit: impl FnOnce(&mut Request)) -> crate::report::Output {
    let mut request = Request::new(which);
    edit(&mut request);
    run(store, &request).unwrap()
}

#[test]
fn inbound_counts_the_distinct_other_pages_that_link_to_a_page() {
    let out = report(&store(), Report::Inbound, |_| {});
    assert_eq!(out.table.columns, ["page", "inbound", "title"]);
    // 2 has 1 and 3 but not itself; 3 has 4 by way of the redirect; the
    // sitemap's entries count for nothing.
    assert_eq!(
        column(&out.table, "page"),
        [url(2), url(1), url(3), url(4), url(5), url(7)]
    );
    let counts: Vec<&Cell> = out.table.rows.iter().map(|r| &r[1]).collect();
    assert_eq!(
        counts,
        [2, 1, 1, 0, 0, 0].map(Cell::Int).iter().collect::<Vec<_>>()
    );
}

#[test]
fn inbound_counts_the_kinds_it_is_given() {
    let out = report(&store(), Report::Inbound, |r| r.kinds = Kind::ALL.to_vec());
    assert_eq!(out.table.rows[1][..2], [Cell::Text(url(3)), Cell::Int(2)]);
    let out = report(&store(), Report::Inbound, |r| r.kinds = vec![Kind::Sitemap]);
    assert!(out.table.rows.iter().all(|r| r[1] == Cell::Int(0)));
}

#[test]
fn inbound_takes_the_filters_and_top() {
    let out = report(&store(), Report::Inbound, |r| r.repo = Some("r/one".into()));
    assert_eq!(column(&out.table, "page"), [url(1)]);
    let out = report(&store(), Report::Inbound, |r| r.top = Some(2));
    assert_eq!(column(&out.table, "page"), [url(2), url(1)]);
}

#[test]
fn orphans_are_the_sitemap_pages_nothing_links_to() {
    let out = report(&store(), Report::Orphans, |_| {});
    assert_eq!(column(&out.table, "page"), [url(7)]);
    // By image only page 3 has a link.
    let out = report(&store(), Report::Orphans, |r| r.kinds = vec![Kind::Image]);
    assert_eq!(column(&out.table, "page"), [url(1), url(2), url(7)]);
}

#[test]
fn unlisted_pages_are_absent_from_the_sitemap_of_a_root_that_read_one() {
    let out = report(&store(), Report::Unlisted, |_| {});
    assert_eq!(column(&out.table, "page"), [url(4)]);
    assert_eq!(column(&out.table, "root"), [ROOT]);
    assert_eq!(
        out.notes,
        ["https://other.test/ read no sitemap, so no page of it is unlisted"]
    );
}

/// Sets the sitemap file 101's answer, and returns the store.
fn sitemap_answering(outcome: &str, reason: Option<&str>) -> Store {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE targets SET outcome = ?, reason = ? WHERE url_id = 101",
            params![outcome, reason],
        )
        .unwrap();
    store
}

#[test]
fn a_root_read_a_sitemap_only_when_a_file_answered_ok_with_no_reason() {
    for (outcome, reason) in [
        ("broken_url", None),
        ("unchecked", Some("robots")),
        ("ok", Some("too_large")),
    ] {
        let out = report(
            &sitemap_answering(outcome, reason),
            Report::Unlisted,
            |_| {},
        );
        assert!(out.table.rows.is_empty(), "{outcome} {reason:?}");
        assert_eq!(out.notes.len(), 2, "{outcome} {reason:?}");
        assert!(out.notes[0].starts_with(ROOT) && out.notes[1].starts_with(OTHER));
    }
}

#[test]
fn an_index_children_count_for_the_root_that_reached_the_index() {
    // The root links to an index that answered `ok` but was over the cap, and
    // the index to file 101, which answered.
    let store = store();
    let conn = store.connection();
    conn.execute(
        "INSERT INTO targets (answered_by, run_id, url_id, url, status, outcome, reason) \
         VALUES ('live', 1, 102, 'https://e.test/index.xml', 200, 'ok', 'too_large')",
        [],
    )
    .unwrap();
    conn.execute("DELETE FROM links WHERE kind = 'sitemap_file'", [])
        .unwrap();
    link(&store, 1, 100, 102, None, "sitemap_file");
    link(&store, 1, 102, 101, None, "sitemap_file");
    let out = report(&store, Report::Unlisted, |_| {});
    assert_eq!(column(&out.table, "page"), [url(4)]);
    // The same chain with the child failed reads none.
    conn.execute(
        "UPDATE targets SET outcome = 'broken_url' WHERE url_id = 101",
        [],
    )
    .unwrap();
    let out = report(&store, Report::Unlisted, |_| {});
    assert!(out.table.rows.is_empty());
}

#[test]
fn sitemap_lists_what_was_not_fetched_then_what_it_does_not_list_then_what_it_does() {
    let out = report(&store(), Report::Sitemap, |_| {});
    // Entry 20 redirected to page 3, so it was fetched; 30 was not.
    assert_eq!(
        column(&out.table, "page"),
        [url(30), url(4), url(5), url(1), url(2), url(3), url(7)]
    );
    assert_eq!(column(&out.table, "outcome")[..2], ["unchecked", "ok"]);
    assert_eq!(column(&out.table, "reason")[0], "robots");
    assert_eq!(out.table.columns[2..4], ["listed", "linked"]);
    // The entry is listed and linked by nothing; page 4 is neither; page 1 is
    // listed by the sitemap and linked by page 4.
    assert_eq!(out.table.rows[0][2..4], [text("direct"), Cell::Null]);
    assert_eq!(out.table.rows[1][2..4], [Cell::Null, Cell::Null]);
    assert_eq!(out.table.rows[3][2..4], [text("direct"), text("direct")]);
}

fn text(s: &str) -> Cell {
    Cell::Text(s.to_owned())
}

/// [`store`] with page 3 named by no sitemap entry and no link but through
/// the redirect 20, which the sitemap lists and page 4 links to.
fn through_redirects() -> Store {
    let store = store();
    store
        .connection()
        .execute(
            "DELETE FROM links WHERE target_url_id = 3 AND src_url_id IN (1, 3, 101)",
            [],
        )
        .unwrap();
    store
}

#[test]
fn a_page_a_sitemap_entry_redirects_to_is_listed_and_one_a_link_redirects_to_is_linked() {
    let store = through_redirects();
    let out = report(&store, Report::Sitemap, |_| {});
    let row = |n: u64| {
        let at = column(&out.table, "page")
            .iter()
            .position(|p| p == &url(n))
            .unwrap();
        out.table.rows[at][2..4].to_vec()
    };
    assert_eq!(row(3), [text("redirect"), text("redirect")]);
    assert_eq!(row(2), [text("direct"), text("direct")]);
    // Listed, so neither unlisted nor, linked by way of page 4, an orphan.
    let out = report(&store, Report::Unlisted, |_| {});
    assert_eq!(column(&out.table, "page"), [url(4)]);
    let out = report(&store, Report::Orphans, |_| {});
    assert_eq!(column(&out.table, "page"), [url(7)]);
}

#[test]
fn direct_wins_where_a_page_is_reached_both_ways() {
    // Page 3 is listed by entries 3 and 20 and linked by 1 and, through 20, 4.
    let out = report(&store(), Report::Sitemap, |_| {});
    let at = column(&out.table, "page")
        .iter()
        .position(|p| p == &url(3))
        .unwrap();
    assert_eq!(out.table.rows[at][2..4], [text("direct"), text("direct")]);
}

#[test]
fn sitemap_filters_by_the_page_so_an_entry_without_one_drops_out() {
    let out = report(&store(), Report::Sitemap, |r| r.repo = Some("r/one".into()));
    assert_eq!(column(&out.table, "page"), [url(1)]);
}
