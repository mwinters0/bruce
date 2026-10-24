use super::super::tests::{ROOT, column, fact, link, page, registry, target, url};
use crate::extract::Kind;
use crate::report::{Cell, Output, Report, Request, Store, VIEWS, run};
use crate::store::RunId;

/// Runs 1 and 2 are finished. Page 2 (repo `r/two`, component `other`) has
/// anchors `a`, `b`, `c` and `d` in run 1, and `a` and `d` in run 2; page 3
/// (`r/three`) has `x` in run 1 and none in run 2; page 4 is new in run 2.
/// Run 2's links:
///
/// - pages 1 and 5 to 2 at `#b`: the anchor was lost, two links on two pages.
/// - page 1 to 2 at `#b` again, by image: another link, of another kind.
/// - page 1 to 2 at `#c`: lost, one link.
/// - page 1 to 2 at `#nope`: never there, so no row.
/// - page 1 to 2 at `#a`: kept, ok.
/// - page 1 to 3 at `#x`: lost.
/// - page 1 to 4 at `#new`: broken, but page 4 had no anchors before.
/// - page 5 to 6 (a redirect to page 3) at `#x`: lost, on the page it landed
///   on.
///
/// Anchor `d` is lost and no link names it: no row.
fn store() -> Store {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    for run in [1, 2] {
        conn.execute(
            "INSERT INTO runs VALUES (?, now(), now(), 'crawl', 'x', 'h', NULL, [], [])",
            [run],
        )
        .unwrap();
        page(&store, run, 1, Some("r/one"), Some(ROOT));
        page(&store, run, 2, Some("r/two"), Some(ROOT));
        page(&store, run, 3, Some("r/three"), Some(ROOT));
        page(&store, run, 5, Some("r/five"), Some(ROOT));
    }
    page(&store, 2, 4, Some("r/four"), Some(ROOT));
    for (run, id, anchor) in [
        (1, 2, "a"),
        (1, 2, "b"),
        (1, 2, "c"),
        (1, 2, "d"),
        (1, 3, "x"),
        (2, 2, "a"),
        (2, 2, "d"),
    ] {
        conn.execute(
            "INSERT INTO anchors VALUES (?, ?, ?, 'h2', 'content')",
            duckdb::params![run, id, anchor],
        )
        .unwrap();
    }
    target(&store, 2, 6, 301, "ok", None);
    conn.execute(
        "UPDATE targets SET final_url = ? WHERE run_id = 2 AND url_id = 6",
        [url(3)],
    )
    .unwrap();
    link(&store, 2, 1, 2, Some("b"), "anchor");
    link(&store, 2, 5, 2, Some("b"), "anchor");
    link(&store, 2, 1, 2, Some("b"), "image");
    link(&store, 2, 1, 2, Some("c"), "anchor");
    link(&store, 2, 1, 2, Some("nope"), "anchor");
    link(&store, 2, 1, 2, Some("a"), "anchor");
    link(&store, 2, 1, 3, Some("x"), "anchor");
    link(&store, 2, 1, 4, Some("new"), "anchor");
    link(&store, 2, 5, 6, Some("x"), "anchor");
    fact(&store, 2, 2, "antora.component", "other");
    store
}

fn removed(store: &Store, edit: impl FnOnce(&mut Request)) -> Output {
    let mut request = Request::new(Report::AnchorsRemoved);
    edit(&mut request);
    run(store, &request).unwrap()
}

/// `page anchor links pages` for each row, the page by its id.
fn rows(out: &Output) -> Vec<String> {
    let page = column(&out.table, "page");
    let anchor = column(&out.table, "anchor");
    out.table
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let n = |at: usize| match row[at] {
                Cell::Int(n) => n,
                _ => panic!("a count"),
            };
            format!(
                "{} {} {} {}",
                page[i].rsplit('/').next().unwrap(),
                anchor[i],
                n(2),
                n(3)
            )
        })
        .collect()
}

#[test]
fn a_lost_anchor_links_still_name_is_a_row_the_most_linked_first() {
    let out = removed(&store(), |_| {});
    assert_eq!((out.from, out.run), (Some((RunId(1), false)), RunId(2)));
    assert_eq!(
        out.table.columns,
        [
            "page",
            "anchor",
            "links",
            "pages",
            "source_repo",
            "source_path"
        ]
    );
    // `x` on page 3 is named from page 1 directly and from page 5 through a
    // redirect; `b` by two pages, its image link not counted.
    assert_eq!(rows(&out), ["2 b 2 2", "3 x 2 2", "2 c 1 1"]);
    assert_eq!(
        column(&out.table, "source_repo"),
        ["r/two", "r/three", "r/two"]
    );
    assert_eq!(out.notes, ["from run 1 to run 2"]);
}

#[test]
fn kind_chooses_the_links_that_count() {
    let out = removed(&store(), |r| r.kinds = vec![Kind::Image]);
    assert_eq!(rows(&out), ["2 b 1 1"]);
    let out = removed(&store(), |r| r.kinds = Kind::ALL.to_vec());
    assert_eq!(rows(&out), ["2 b 3 2", "3 x 2 2", "2 c 1 1"]);
}

#[test]
fn the_filters_apply_to_the_page_that_lost_the_anchor() {
    let store = store();
    let out = removed(&store, |r| r.repo = Some("r/three".into()));
    assert_eq!(rows(&out), ["3 x 2 2"]);
    let out = removed(&store, |r| {
        r.facts = vec![("antora.component".into(), "other".into())];
    });
    assert_eq!(rows(&out), ["2 b 2 2", "2 c 1 1"]);
    let out = removed(&store, |r| r.top = Some(1));
    assert_eq!(rows(&out), ["2 b 2 2"]);
}

#[test]
fn json_lists_the_linking_pages() {
    let out = removed(&store(), |r| r.linking_pages = true);
    assert_eq!(out.table.columns[4], "linking_pages");
    assert_eq!(
        out.table.rows[1][4],
        Cell::List(vec![url(1), url(5)]),
        "{:?}",
        out.table.rows
    );
}

#[test]
fn an_anchor_a_run_before_the_other_gained_is_no_row() {
    let store = store();
    // Compared the other way round, page 2 gained `b` and `c`: nothing lost.
    let out = removed(&store, |r| {
        r.from = Some(RunId(2));
        r.to = Some(RunId(1));
    });
    assert_eq!(rows(&out), Vec::<String>::new());
}

#[test]
fn a_percent_encoded_fragment_names_the_anchor_it_decodes_to() {
    let store = store();
    store
        .connection()
        .execute(
            "INSERT INTO anchors VALUES (1, 2, 'é', 'h2', 'content')",
            [],
        )
        .unwrap();
    link(&store, 2, 1, 2, Some("%C3%A9"), "anchor");
    let out = removed(&store, |_| {});
    assert!(
        rows(&out).contains(&"2 %C3%A9 1 1".to_owned()),
        "{:?}",
        rows(&out)
    );
}
