use duckdb::params;

use super::super::tests::{column, link, page, target, url};
use crate::extract::Kind;
use crate::profile::Registry;
use crate::profile::generic::Generic;
use crate::report::{Cell, Format, Output, Report, Request, Store, VIEWS, run};

const LOGIN: &str = "https://e.test/login";

fn moved(store: &Store, id: u64, landing: &str, chain: &str) {
    target(store, 1, id, 200, "ok", None);
    store
        .connection()
        .execute(
            &format!(
                "UPDATE targets SET final_url = ?, redirect_chain = [{chain}] WHERE url_id = ?"
            ),
            params![landing, id],
        )
        .unwrap();
}

fn hop(url: String, status: u16, refresh: bool) -> String {
    format!("{{'url': '{url}', 'status': {status}, 'refresh': {refresh}}}")
}

/// Page 3 is reached by 20 (a 301) and 21 (a refresh); the login page by
/// 22, which pages 1, 2 and 3 link to, and 2 links to 22 by image as well.
fn store() -> Store {
    let store = Store::open_in_memory(&Registry::new(Box::new(Generic))).unwrap();
    store.install(VIEWS).unwrap();
    store
        .connection()
        .execute(
            "INSERT INTO runs VALUES (1, now(), now(), 'crawl', 'x', 'h', NULL, [])",
            [],
        )
        .unwrap();
    for id in 1..=4 {
        page(
            &store,
            1,
            id,
            (id == 1).then_some("r/one"),
            Some("https://e.test/"),
        );
    }
    moved(&store, 20, &url(3), &hop(url(20), 301, false));
    moved(&store, 21, &url(3), &hop(url(21), 200, true));
    moved(&store, 22, LOGIN, &hop(url(22), 303, false));
    link(&store, 1, 4, 20, None, "anchor");
    link(&store, 1, 1, 21, None, "anchor");
    for src in [1, 2, 3] {
        link(&store, 1, src, 22, None, "anchor");
    }
    link(&store, 1, 2, 22, None, "image");
    link(&store, 1, 2, 3, None, "anchor");
    store
}

fn report(store: &Store, edit: impl FnOnce(&mut Request)) -> Output {
    let mut request = Request::new(Report::Redirects);
    edit(&mut request);
    run(store, &request).unwrap()
}

#[test]
fn redirects_group_the_links_by_where_they_landed() {
    let out = report(&store(), |_| {});
    assert_eq!(out.table.columns, ["landing", "links", "urls", "chain"]);
    assert_eq!(column(&out.table, "landing"), [LOGIN, &url(3)[..]]);
    assert_eq!(out.table.rows[0][1..3], [Cell::Int(3), Cell::Int(1)]);
    assert_eq!(out.table.rows[1][1..3], [Cell::Int(2), Cell::Int(2)]);
    assert_eq!(
        column(&out.table, "chain"),
        [format!("303 {}", url(22)), format!("301 {}", url(20))]
    );
}

#[test]
fn a_refresh_hop_is_named_as_one() {
    let out = report(&store(), |r| r.repo = Some("r/one".into()));
    assert_eq!(column(&out.table, "landing"), [&url(3)[..], LOGIN]);
    assert_eq!(out.table.rows[0][1], Cell::Int(1));
    assert_eq!(
        column(&out.table, "chain")[0],
        format!("refresh {}", url(21))
    );
}

#[test]
fn redirects_count_the_kinds_they_are_given() {
    let out = report(&store(), |r| r.kinds = vec![Kind::Image]);
    assert_eq!(column(&out.table, "landing"), [LOGIN]);
    assert_eq!(out.table.rows[0][1], Cell::Int(1));
}

#[test]
fn the_linking_pages_are_added_on_request_and_are_an_array_in_json() {
    let out = report(&store(), |r| r.linking_pages = true);
    assert_eq!(out.table.columns[4], "pages");
    assert_eq!(out.table.rows[1][4], Cell::List(vec![url(1), url(4)]));
    let json = out.table.render(Format::Json);
    assert!(
        json.contains(&format!("\"pages\":[\"{}\",\"{}\"]", url(1), url(4))),
        "{json}"
    );
}
