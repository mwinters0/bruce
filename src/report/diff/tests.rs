use duckdb::params;

use super::super::tests::{ROOT, column, link, page, target, url};
use crate::extract::Kind;
use crate::profile::Registry;
use crate::profile::generic::Generic;
use crate::report::{Error, Output, Report, Request, Store, VIEWS, run};
use crate::store::RunId;

/// Runs 1 and 2 are finished, run 3 is not. Page 1 (repo `r/one`) and page 2
/// (`r/two`) are in both; page 3 is only in run 2. Targets 10 and 11 are 404s
/// in run 1; in run 2 11 answers and 12 is a 404.
///
/// - 1 to 10: broken in both, so no row.
/// - 1 to 11: broken in 1 only, so `fixed`.
/// - 2 to 10: broken in 1 only, the link gone from 2, so `fixed`.
/// - 1 to 12: broken in 2 only, so `new`.
/// - 3 to 10: broken in 2 only, on a page that is new, so `new`.
/// - 1 to page 2 at `#nope`: broken in 2 only, so `new`; at `#sec` it is
///   broken in neither.
fn store() -> Store {
    let store = Store::open_in_memory(&Registry::new(Box::new(Generic))).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    for (run, finished) in [(1, true), (2, true), (3, false)] {
        conn.execute(
            "INSERT INTO runs VALUES (?, now(), CASE WHEN ? THEN now() END, 'crawl', 'x', 'h', \
             NULL, [])",
            params![run, finished],
        )
        .unwrap();
    }
    for run in [1, 2] {
        page(&store, run, 1, Some("r/one"), Some(ROOT));
        page(&store, run, 2, Some("r/two"), Some(ROOT));
        conn.execute(
            "INSERT INTO anchors VALUES (?, 2, 'sec', 'h2', 'content')",
            [run],
        )
        .unwrap();
        target(&store, run, 10, 404, "broken_url", None);
        link(&store, run, 1, 10, None, "anchor");
        link(&store, run, 1, 2, Some("sec"), "anchor");
    }
    target(&store, 1, 11, 404, "broken_url", None);
    link(&store, 1, 1, 11, None, "anchor");
    link(&store, 1, 2, 10, None, "anchor");
    target(&store, 2, 11, 200, "ok", None);
    link(&store, 2, 1, 11, None, "anchor");
    target(&store, 2, 12, 404, "broken_url", None);
    link(&store, 2, 1, 12, None, "anchor");
    page(&store, 2, 3, None, Some(ROOT));
    link(&store, 2, 3, 10, None, "anchor");
    link(&store, 2, 1, 2, Some("nope"), "anchor");
    store
}

fn diff(store: &Store, edit: impl FnOnce(&mut Request)) -> Output {
    let mut request = Request::new(Report::Diff);
    edit(&mut request);
    run(store, &request).unwrap()
}

fn changes(out: &Output) -> Vec<String> {
    let change = column(&out.table, "change");
    let href = column(&out.table, "raw_href");
    change
        .iter()
        .zip(&href)
        .map(|(c, h)| format!("{c} {h}"))
        .collect()
}

#[test]
fn the_latest_two_finished_runs_are_compared_and_new_comes_first() {
    let out = diff(&store(), |_| {});
    assert_eq!((out.from, out.run), (Some((RunId(1), false)), RunId(2)));
    assert!(!out.unfinished);
    assert_eq!(
        out.table.columns,
        [
            "change",
            "page",
            "raw_href",
            "target",
            "fragment",
            "kind",
            "outcome",
            "reason",
            "status",
            "source_repo",
            "source_path"
        ]
    );
    // `broken_url` before `broken_anchor`, then by page and href.
    assert_eq!(
        changes(&out),
        [
            "new 1>12#",
            "new 3>10#",
            "new 1>2#nope",
            "fixed 1>11#",
            "fixed 2>10#"
        ]
    );
    assert_eq!(out.notes, ["from run 1 to run 2"]);
}

#[test]
fn a_fixed_row_describes_the_link_as_it_was_broken() {
    let out = diff(&store(), |_| {});
    let at = |col| column(&out.table, col)[3].clone();
    assert_eq!(at("page"), url(1));
    assert_eq!(at("target"), url(11));
    assert_eq!(at("outcome"), "broken_url");
    assert_eq!(out.table.rows[3][8], crate::report::Cell::Int(404));
    assert_eq!(column(&out.table, "source_repo")[4], "r/two");
}

#[test]
fn from_and_to_name_the_runs_and_either_may_be_unfinished() {
    let store = store();
    let out = diff(&store, |r| {
        r.from = Some(RunId(2));
        r.to = Some(RunId(1));
    });
    assert_eq!(
        changes(&out),
        [
            "new 1>11#",
            "new 2>10#",
            "fixed 1>12#",
            "fixed 3>10#",
            "fixed 1>2#nope"
        ]
    );
    // Run 3 never finished and holds no link, so everything run 2 had broken
    // is fixed; the base defaults to the finished run before it.
    let out = diff(&store, |r| r.to = Some(RunId(3)));
    assert_eq!(
        (out.from, out.run, out.unfinished),
        (Some((RunId(2), false)), RunId(3), true)
    );
    assert_eq!(
        changes(&out),
        [
            "fixed 1>10#",
            "fixed 1>12#",
            "fixed 3>10#",
            "fixed 1>2#nope"
        ]
    );
    let out = diff(&store, |r| r.from = Some(RunId(3)));
    assert_eq!(out.from, Some((RunId(3), true)));
    assert_eq!(out.run, RunId(2));
}

#[test]
fn a_run_that_is_not_there_or_has_nothing_before_it_is_refused() {
    let store = store();
    let attempt = |edit: &dyn Fn(&mut Request)| {
        let mut request = Request::new(Report::Diff);
        edit(&mut request);
        run(&store, &request).unwrap_err()
    };
    let err = attempt(&|r| r.from = Some(RunId(9)));
    assert!(matches!(err, Error::NoSuchRun(RunId(9))));
    let err = attempt(&|r| r.to = Some(RunId(1)));
    assert!(matches!(err, Error::NoEarlierRun(RunId(1))));
    assert!(err.to_string().contains("--from"));
}

#[test]
fn the_filters_apply_to_the_page_in_the_run_a_row_comes_from() {
    let store = store();
    let out = diff(&store, |r| r.repo = Some("r/two".into()));
    assert_eq!(changes(&out), ["fixed 2>10#"]);
    let out = diff(&store, |r| r.repo = Some("r/one".into()));
    assert_eq!(changes(&out), ["new 1>12#", "new 1>2#nope", "fixed 1>11#"]);
    let out = diff(&store, |r| r.top = Some(2));
    assert_eq!(changes(&out), ["new 1>12#", "new 3>10#"]);
}

#[test]
fn kind_says_which_links_are_compared_on_both_sides() {
    let store = store();
    // The link to 12 was broken as an image in run 1 and as an anchor in 2.
    link(&store, 1, 1, 12, None, "image");
    target(&store, 1, 12, 404, "broken_url", None);
    let out = diff(&store, |_| {});
    assert!(changes(&out).contains(&"new 1>12#".to_owned()));
    let out = diff(&store, |r| r.kinds = vec![Kind::Image]);
    assert_eq!(changes(&out), ["fixed 1>12#"]);
    let out = diff(&store, |r| r.kinds = Kind::ALL.to_vec());
    assert!(!changes(&out).iter().any(|c| c.ends_with("1>12#")));
}

#[test]
fn links_sharing_a_key_are_one_row() {
    let store = store();
    store
        .connection()
        .execute(
            "INSERT INTO links VALUES (2, 1, 12, NULL, 'again', 'anchor', NULL)",
            [],
        )
        .unwrap();
    let out = diff(&store, |r| r.repo = Some("r/one".into()));
    assert_eq!(changes(&out), ["new 1>12#", "new 1>2#nope", "fixed 1>11#"]);
}
