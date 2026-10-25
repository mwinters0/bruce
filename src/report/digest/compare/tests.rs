use serde_json::Value;

use super::super::tests::{compared, of, store};
use super::super::{Against, digest};
use crate::report::Error;
use crate::report::tests::{registry, url};
use crate::store::RunId;

fn runs(store: &crate::store::Store) -> i64 {
    store
        .connection()
        .query_row("SELECT count(*) FROM runs", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn a_digest_is_compared_with_the_finished_crawl_run_before_it() {
    let store = compared();
    let conn = store.connection();
    // Run 2 is a `check-dir` run and run 3 never finished: neither is one.
    assert_eq!(super::previous(conn, RunId(4)).unwrap(), Some(RunId(1)));
    assert_eq!(super::previous(conn, RunId(3)).unwrap(), Some(RunId(1)));
    assert_eq!(super::previous(conn, RunId(1)).unwrap(), None);
    let d = digest(&store, Some(RunId(3)), Against::Previous).unwrap();
    assert_eq!(d.compared.unwrap().run.id, RunId(1));
}

#[test]
fn a_run_shares_its_day_with_another_crawl_run_started_that_day() {
    let store = compared();
    let conn = store.connection();
    assert!(!super::shares_day(conn, RunId(4)).unwrap());
    let add = |id: u64, mode: &str| {
        conn.execute(
            "INSERT INTO runs SELECT ?, started_at + INTERVAL 1 HOUR, NULL, ?, '0.0.8', 'h', \
             NULL, roots, [] FROM runs WHERE run_id = 4",
            duckdb::params![id, mode],
        )
        .unwrap();
    };
    // A `check-dir` run that day is no crawl, and no digest names it.
    add(5, "local");
    assert!(!super::shares_day(conn, RunId(4)).unwrap());
    add(6, "crawl");
    assert!(super::shares_day(conn, RunId(4)).unwrap());
    let d = digest(&store, Some(RunId(1)), Against::Run(RunId(4))).unwrap();
    assert!(d.compared.unwrap().shares_day);
}

#[test]
fn a_digest_with_nothing_to_compare_with_says_so() {
    let d = of(&store());
    assert!(d.compared.is_none() && d.anchors_removed.rows.is_empty());
}

#[test]
fn a_named_run_is_compared_with_its_figures_and_the_anchors_it_lost() {
    let store = compared();
    let d = digest(&store, Some(RunId(1)), Against::Run(RunId(4))).unwrap();
    let c = d.compared.as_ref().unwrap();
    assert_eq!((c.run.id, c.file), (RunId(4), false));
    assert_eq!(
        (c.totals.pages, c.totals.links, c.totals.broken_url),
        (2, 1, 1)
    );
    assert_eq!(c.run.tool_version, "0.0.8");
    let t = &d.anchors_removed;
    assert_eq!(
        crate::report::tests::column(t, "page"),
        vec![url(2)],
        "{t:?}"
    );
    assert_eq!(crate::report::tests::column(t, "anchor"), vec!["nope"]);
    assert_eq!(crate::report::tests::column(t, "links"), vec!["Int(1)"]);
    assert_eq!(
        crate::report::tests::column(t, "linking_pages"),
        vec![format!("List([\"{}\"])", url(1))]
    );
}

#[test]
fn a_run_that_is_not_a_crawl_or_not_there_is_refused() {
    let store = compared();
    assert!(matches!(
        digest(&store, Some(RunId(1)), Against::Run(RunId(2))),
        Err(Error::NotCrawl(RunId(2)))
    ));
    assert!(matches!(
        digest(&store, Some(RunId(1)), Against::Run(RunId(9))),
        Err(Error::NoSuchRun(RunId(9)))
    ));
}

/// A file holding run 4 of [`compared`], and the directory it is in.
fn exported() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("baseline.duckdb");
    compared()
        .export_baseline(Some(RunId(4)), &out, &registry())
        .unwrap();
    (dir, out)
}

#[test]
fn a_baseline_file_is_imported_for_the_digest_and_removed_after() {
    let (_dir, out) = exported();
    let store = store();
    let before = runs(&store);
    let d = digest(&store, Some(RunId(1)), Against::File(&out)).unwrap();
    let c = d.compared.as_ref().unwrap();
    assert!(c.file);
    assert_eq!(c.totals.pages, 2);
    assert_eq!(d.anchors_removed.rows.len(), 1);
    assert_eq!(runs(&store), before, "the import is not left behind");
    let json: Value = serde_json::from_str(&d.json()).unwrap();
    assert_eq!(json["compared_with"]["id"], Value::Null);
    assert_eq!(json["compared_with"]["from_file"], true);
    assert_eq!(json["compared_with"]["totals"]["pages"], 2);
}

#[test]
fn a_baseline_the_database_already_holds_is_left_there() {
    let store = compared();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("b.duckdb");
    store
        .export_baseline(Some(RunId(4)), &out, &registry())
        .unwrap();
    let before = runs(&store);
    let d = digest(&store, Some(RunId(1)), Against::File(&out)).unwrap();
    assert!(d.compared.unwrap().file);
    assert_eq!(runs(&store), before);
    assert_eq!(
        store
            .connection()
            .query_row("SELECT count(*) FROM runs WHERE run_id = 4", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn a_file_that_is_no_baseline_is_refused_and_the_database_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("junk.duckdb");
    std::fs::write(&junk, b"not a database").unwrap();
    let store = store();
    let before = runs(&store);
    assert!(matches!(
        digest(&store, Some(RunId(1)), Against::File(&junk)),
        Err(Error::Baseline(_))
    ));
    assert_eq!(runs(&store), before);
}

#[test]
fn the_json_holds_the_run_compared_with() {
    let d = digest(&compared(), Some(RunId(1)), Against::Run(RunId(4))).unwrap();
    let json: Value = serde_json::from_str(&d.json()).unwrap();
    let with = &json["compared_with"];
    assert_eq!(with["id"], 4);
    assert_eq!(with["from_file"], false);
    assert_eq!(with["tool_version"], "0.0.8");
    assert_eq!(with["started_at"], "2026-10-22T20:04:11Z");
    assert_eq!(with["finished_at"], "2026-10-22T20:05:11Z");
    assert_eq!(with["totals"]["broken_url"], 1);
    assert_eq!(json["anchors_removed"][0]["anchor"], "nope");
    assert_eq!(json["anchors_removed"][0]["linking_pages"][0], url(1));
}
