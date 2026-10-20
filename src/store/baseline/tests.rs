use super::*;
use crate::profile::Registry;
use crate::profile::generic::Generic;
use crate::store::{Mode, SCHEMA_VERSION};

use duckdb::params;

fn registry() -> Registry {
    Registry::new(Box::new(Generic))
}

fn count(store: &Store, sql: &str) -> i64 {
    store.connection().query_row(sql, [], |r| r.get(0)).unwrap()
}

/// A run holding `n` targets (the first redirected, so its chain is a
/// `STRUCT[]`), one page with a fact and an anchor, one link and one body.
fn run(store: &Store, n: u64, baseline: Option<RunId>, finished: bool) -> RunId {
    let run = store
        .begin_run(Mode::Crawl, "abc", baseline, &["https://a.test/".into()])
        .unwrap();
    let conn = store.connection();
    for id in 1..=n {
        let chain = if id == 1 {
            "[{'url': 'https://a.test/old', 'status': 301, 'refresh': false}]"
        } else {
            "[]"
        };
        conn.execute(
            &format!(
                "INSERT INTO targets (answered_by, run_id, url_id, url, outcome, redirect_chain) \
                 VALUES ('live', ?, ?, ?, 'ok', {chain})"
            ),
            params![run.0, id, format!("https://a.test/{id}")],
        )
        .unwrap();
    }
    for sql in [
        "INSERT INTO pages VALUES (?, 1, 'T', 'text/html', 5, NULL, 'generic', NULL, NULL, NULL, NULL, true, false, NULL)",
        "INSERT INTO page_facts VALUES (?, 1, 'k', 'v')",
        "INSERT INTO anchors VALUES (?, 1, 'x', 'h2', 'content')",
        "INSERT INTO links VALUES (?, 1, 1, 'x', '#x', 'anchor', NULL)",
        "INSERT INTO bodies VALUES (?, 1, '<p>')",
    ] {
        conn.execute(sql, [run.0]).unwrap();
    }
    if finished {
        store.finish_run(run).unwrap();
    }
    run
}

fn source() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("src.duckdb"), &registry()).unwrap();
    (dir, store)
}

#[test]
fn the_latest_finished_run_is_exported_alone_in_a_file_of_the_current_schema() {
    let (dir, store) = source();
    let first = run(&store, 2, None, true);
    let second = run(&store, 3, Some(first), true);
    run(&store, 4, None, false);
    let out = dir.path().join("baseline.duckdb");
    let exported = store.export_baseline(None, &out, &registry()).unwrap();
    assert_eq!(
        exported,
        Exported {
            run: second,
            targets: 3,
            pages: 1,
            facts: 1,
            anchors: 1,
            links: 1
        }
    );
    let file = Store::open(&out, &registry()).unwrap();
    let version: String = file
        .connection()
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION.to_string());
    assert_eq!(count(&file, "SELECT count(*) FROM runs"), 1);
    assert_eq!(count(&file, "SELECT run_id FROM runs"), 2);
    assert_eq!(
        count(
            &file,
            "SELECT count(*) FROM runs WHERE baseline_run IS NULL"
        ),
        1
    );
    assert_eq!(
        count(&file, "SELECT count(*) FROM targets WHERE run_id = 2"),
        3
    );
    assert_eq!(count(&file, "SELECT count(*) FROM targets"), 3);
    assert_eq!(
        count(
            &file,
            "SELECT len(redirect_chain) FROM targets WHERE url_id = 1"
        ),
        1
    );
    assert_eq!(count(&file, "SELECT count(*) FROM bodies"), 0);
    assert_eq!(count(&file, "SELECT count(*) FROM link_verdicts"), 1);
}

#[test]
fn a_named_finished_run_is_exported_and_the_source_is_left_whole() {
    let (dir, store) = source();
    let first = run(&store, 2, None, true);
    run(&store, 3, None, true);
    let out = dir.path().join("baseline.duckdb");
    let exported = store
        .export_baseline(Some(first), &out, &registry())
        .unwrap();
    assert_eq!((exported.run, exported.targets), (first, 2));
    assert_eq!(count(&store, "SELECT count(*) FROM targets"), 5);
    assert_eq!(count(&store, "SELECT count(*) FROM bodies"), 2);
}

#[test]
fn an_unfinished_run_a_missing_one_and_no_finished_run_are_refused_and_leave_no_file() {
    let (dir, store) = source();
    let out = dir.path().join("baseline.duckdb");
    let refused = |run| store.export_baseline(run, &out, &registry()).unwrap_err();
    assert!(matches!(refused(None), Error::NoFinishedRun));
    let unfinished = run(&store, 2, None, false);
    assert!(matches!(refused(None), Error::NoFinishedRun));
    assert!(matches!(refused(Some(unfinished)), Error::Unfinished(r) if r == unfinished));
    assert!(matches!(
        refused(Some(RunId(9))),
        Error::NoSuchRun(RunId(9))
    ));
    assert!(!out.exists());
}
