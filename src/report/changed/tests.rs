use duckdb::params;

use super::super::tests::{ROOT, column, fact, page, registry, url};
use crate::report::{Cell, Output, Report, Request, Store, VIEWS, run};
use crate::store::RunId;

/// Runs 1 and 2 are finished. Both record one root, `ROOT`, whose `hash_root`
/// is `article`. Pages by `(scope, hash)` in run 1 and run 2:
///
/// - 1: `(region, a)` then `(region, b)`: `changed`.
/// - 2: `(region, a)` then `(region, a)`: no row.
/// - 3: `(region, a)` then `(whole, a)`: `scope_changed`, and missed.
/// - 4: `(whole, a)` in both: `root_missed`, its hash unmoved.
/// - 5: `(whole, a)` then `(whole, b)`: `changed`, and missed.
/// - 6: no root in either, its hash moved: no row.
/// - 7: only in run 2, `(region, a)`: no row.
/// - 8: only in run 2, `(whole, a)`: `root_missed`, with no scope before.
/// - 9: only in run 1: no row.
/// - 10: `(root, a)` then `(root, b)`, repo `r/two`, component `other`:
///   `changed`.
fn store() -> Store {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    for run in [1, 2] {
        store
            .connection()
            .execute(
                &format!(
                    "INSERT INTO runs VALUES (?, now(), now(), 'crawl', 'x', 'h', NULL, ['{ROOT}'], \
                     [{{'root': '{ROOT}', 'hash_root': ['article'], \
                     'hash_exclude': []::VARCHAR[]}}])"
                ),
                [run],
            )
            .unwrap();
    }
    let pages = [
        (1, 1, "region", "a", Some(ROOT), Some("r/one")),
        (2, 1, "region", "b", Some(ROOT), Some("r/one")),
        (1, 2, "region", "a", Some(ROOT), Some("r/one")),
        (2, 2, "region", "a", Some(ROOT), Some("r/one")),
        (1, 3, "region", "a", Some(ROOT), Some("r/one")),
        (2, 3, "whole", "a", Some(ROOT), Some("r/one")),
        (1, 4, "whole", "a", Some(ROOT), Some("r/one")),
        (2, 4, "whole", "a", Some(ROOT), Some("r/one")),
        (1, 5, "whole", "a", Some(ROOT), Some("r/one")),
        (2, 5, "whole", "b", Some(ROOT), Some("r/one")),
        (1, 6, "region", "a", None, None),
        (2, 6, "region", "b", None, None),
        (2, 7, "region", "a", Some(ROOT), Some("r/one")),
        (2, 8, "whole", "a", Some(ROOT), Some("r/one")),
    ];
    let more = [
        (1, 9, "region", "a", Some(ROOT), Some("r/one")),
        (1, 10, "root", "a", Some(ROOT), Some("r/two")),
        (2, 10, "root", "b", Some(ROOT), Some("r/two")),
    ];
    for (run, id, scope, hash, root, repo) in pages.into_iter().chain(more) {
        page(&store, run, id, repo, root);
        store
            .connection()
            .execute(
                "UPDATE pages SET hash_scope = ?, content_hash = encode(?), title = ? \
                 WHERE run_id = ? AND url_id = ?",
                params![scope, hash, format!("page {id}"), run, id],
            )
            .unwrap();
    }
    fact(&store, 2, 10, "antora.component", "other");
    store
}

fn changed(store: &Store, edit: impl FnOnce(&mut Request)) -> Output {
    let mut request = Request::new(Report::Changed);
    edit(&mut request);
    run(store, &request).unwrap()
}

/// `change page` for each row, the page by its id.
fn rows(out: &Output) -> Vec<String> {
    let change = column(&out.table, "change");
    let page = column(&out.table, "page");
    change
        .iter()
        .zip(&page)
        .map(|(c, p)| format!("{c} {}", p.rsplit('/').next().unwrap()))
        .collect()
}

#[test]
fn a_moved_hash_a_changed_scope_and_a_missed_root_are_rows_in_that_order() {
    let out = changed(&store(), |_| {});
    assert_eq!((out.from, out.run), (Some((RunId(1), false)), RunId(2)));
    assert_eq!(
        out.table.columns,
        [
            "change",
            "page",
            "scope_from",
            "scope_to",
            "root_missed",
            "title",
            "source_repo",
            "source_path"
        ]
    );
    assert_eq!(
        rows(&out),
        [
            "changed 1",
            "changed 10",
            "changed 5",
            "scope_changed 3",
            "root_missed 4",
            "root_missed 8"
        ]
    );
    assert_eq!(out.notes, ["from run 1 to run 2"]);
}

#[test]
fn a_row_says_the_scopes_it_was_hashed_under_and_whether_its_root_missed() {
    let out = changed(&store(), |_| {});
    let flag = |id: u64| -> Cell {
        let at = column(&out.table, "page")
            .iter()
            .position(|p| *p == url(id))
            .unwrap();
        out.table.rows[at][4].clone()
    };
    assert_eq!(flag(1), Cell::Bool(false));
    assert_eq!(flag(5), Cell::Bool(true));
    assert_eq!(flag(3), Cell::Bool(true));
    let scopes = |id: u64| -> (Cell, Cell) {
        let at = column(&out.table, "page")
            .iter()
            .position(|p| *p == url(id))
            .unwrap();
        (out.table.rows[at][2].clone(), out.table.rows[at][3].clone())
    };
    let text = |s: &str| Cell::Text(s.to_owned());
    assert_eq!(scopes(3), (text("region"), text("whole")));
    assert_eq!(scopes(8), (Cell::Null, text("whole")));
    assert_eq!(column(&out.table, "title")[0], "page 1");
}

#[test]
fn a_root_whose_keys_changed_between_the_runs_changes_every_pages_scope() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE runs SET hash_scopes = [{'root': ?, 'hash_root': ['article'], \
             'hash_exclude': ['nav']}] WHERE run_id = 1",
            [ROOT],
        )
        .unwrap();
    let out = changed(&store, |_| {});
    // Pages 2, 4 and 6 had the same hash and 1, 5 and 10 a moved one; none is
    // `changed` now, the hashes not being comparable.
    assert_eq!(
        rows(&out),
        [
            "scope_changed 1",
            "scope_changed 10",
            "scope_changed 2",
            "scope_changed 3",
            "scope_changed 4",
            "scope_changed 5",
            "root_missed 8"
        ]
    );
}

#[test]
fn a_root_without_hash_root_misses_nothing() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE runs SET hash_scopes = [{'root': ?, 'hash_root': []::VARCHAR[], \
             'hash_exclude': []::VARCHAR[]}]",
            [ROOT],
        )
        .unwrap();
    let out = changed(&store, |_| {});
    // The keys are the same in both runs, so the rows are the moved hashes
    // and the changed scope, and the whole pages are not missed.
    assert_eq!(
        rows(&out),
        ["changed 1", "changed 10", "changed 5", "scope_changed 3"]
    );
    assert!(out.table.rows.iter().all(|r| r[4] == Cell::Bool(false)));
}

#[test]
fn the_filters_apply_to_the_page_in_to_and_top_keeps_the_first_rows() {
    let store = store();
    let out = changed(&store, |r| r.repo = Some("r/two".into()));
    assert_eq!(rows(&out), ["changed 10"]);
    let out = changed(&store, |r| {
        r.facts = vec![("antora.component".into(), "other".into())];
    });
    assert_eq!(rows(&out), ["changed 10"]);
    let out = changed(&store, |r| r.top = Some(2));
    assert_eq!(rows(&out), ["changed 1", "changed 10"]);
}

#[test]
fn from_and_to_name_the_runs() {
    let store = store();
    // Run 2 against run 1: page 1 went back from `b` to `a`, still a change;
    // page 8 is not in run 1, and page 3 is `region` there and not missed.
    let out = changed(&store, |r| {
        r.from = Some(RunId(2));
        r.to = Some(RunId(1));
    });
    assert_eq!(
        rows(&out),
        [
            "changed 1",
            "changed 10",
            "changed 5",
            "scope_changed 3",
            "root_missed 4"
        ]
    );
    assert_eq!(out.notes, ["from run 2 to run 1"]);
}
