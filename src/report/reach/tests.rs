use duckdb::params;

use super::super::tests::{ROOT, column, link, page, target, url};
use super::*;
use crate::extract::Kind;
use crate::profile::Registry;
use crate::profile::generic::Generic;
use crate::report::{Cell, Report, Store, VIEWS, run};

const OTHER: &str = "https://other.test/";

/// Run 1, finished, with roots `ROOT`, which read a sitemap, and `OTHER`,
/// which read none. The sitemap file (101) lists pages 1, 2, 3 and 7, the
/// redirect 20 (to page 3) and 30, which robots.txt forbade. Pages 1 to 4
/// and 7 are in `ROOT`, 5 in `OTHER`, 6 is allowlisted; 4 and 5 are in no
/// sitemap. Page 1 links to 2 and, by image, to 3; 3 to 2; 2 to itself;
/// 4 to 1 and, through the redirect 20, to 3.
fn store() -> Store {
    let store = Store::open_in_memory(&Registry::new(Box::new(Generic))).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    conn.execute(
        "INSERT INTO runs VALUES (1, now(), now(), 'crawl', 'x', 'h', NULL, ?)",
        params![format!("['{ROOT}', '{OTHER}']")],
    )
    .unwrap();
    for (id, repo, root, listed) in [
        (1, Some("r/one"), ROOT, true),
        (2, None, ROOT, true),
        (3, None, ROOT, true),
        (4, None, ROOT, false),
        (5, None, OTHER, false),
        (7, None, ROOT, true),
    ] {
        page(&store, 1, id, repo, Some(root));
        conn.execute(
            "UPDATE pages SET in_sitemap = ?, title = ? WHERE url_id = ?",
            params![listed, format!("Page {id}"), id],
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
            "INSERT INTO targets (run_id, url_id, url, status, outcome) \
             VALUES (1, ?, ?, 200, 'ok')",
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

#[test]
fn sitemap_lists_what_was_not_fetched_then_what_it_does_not_list() {
    let out = report(&store(), Report::Sitemap, |_| {});
    // Entry 20 redirected to page 3, so it was fetched; 30 was not.
    assert_eq!(
        column(&out.table, "page"),
        [url(30), url(4), url(5), url(1), url(2), url(3), url(7)]
    );
    assert_eq!(column(&out.table, "outcome")[..2], ["unchecked", "ok"]);
    assert_eq!(column(&out.table, "reason")[0], "robots");
    assert_eq!(out.table.rows[0][3], Cell::Null);
    assert_eq!(out.table.rows[1][2], Cell::Bool(false));
    assert_eq!(out.table.rows[3][2], Cell::Bool(true));
}

#[test]
fn sitemap_filters_by_the_page_so_an_entry_without_one_drops_out() {
    let out = report(&store(), Report::Sitemap, |r| r.repo = Some("r/one".into()));
    assert_eq!(column(&out.table, "page"), [url(1)]);
}
