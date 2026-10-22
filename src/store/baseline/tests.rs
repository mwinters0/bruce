use super::*;
use crate::profile::Registry;
use crate::profile::generic::Generic;
use crate::store::{Mode, SCHEMA_VERSION};

use duckdb::params;
use std::path::{Path, PathBuf};

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

/// A source database exporting its latest run to `baseline.duckdb`, and an
/// empty CI database beside it.
fn exported() -> (tempfile::TempDir, Store, Store, PathBuf) {
    let (dir, store) = source();
    run(&store, 2, None, true);
    let out = dir.path().join("baseline.duckdb");
    store.export_baseline(None, &out, &registry()).unwrap();
    let ci = Store::open(&dir.path().join("ci.duckdb"), &registry()).unwrap();
    (dir, store, ci, out)
}

#[test]
fn an_imported_baseline_is_a_run_of_its_own_under_the_next_number() {
    let (_dir, _store, ci, out) = exported();
    let earlier = run(&ci, 1, None, true);
    let imported = ci.import_baseline(&out).unwrap();
    assert_eq!(
        imported,
        Imported {
            run: RunId(earlier.0 + 1),
            reused: false
        }
    );
    let n = imported.run.0;
    for (table, rows) in [
        ("targets", 2),
        ("pages", 1),
        ("page_facts", 1),
        ("anchors", 1),
    ] {
        assert_eq!(
            count(
                &ci,
                &format!("SELECT count(*) FROM {table} WHERE run_id = {n}")
            ),
            rows,
            "{table}"
        );
    }
    assert_eq!(
        count(
            &ci,
            &format!(
                "SELECT count(*) FROM runs WHERE run_id = {n} \
                 AND finished_at IS NOT NULL AND baseline_run IS NULL"
            )
        ),
        1
    );
    assert_eq!(
        count(
            &ci,
            &format!("SELECT count(*) FROM link_verdicts WHERE run_id = {n} AND outcome = 'ok'")
        ),
        1
    );
    assert_eq!(
        count(&ci, "SELECT count(*) FROM bodies WHERE run_id = 2"),
        0
    );
    assert_eq!(
        count(
            &ci,
            &format!("SELECT count(*) FROM targets WHERE run_id = {n} AND answered_by = 'live'")
        ),
        2
    );
}

#[test]
fn a_baseline_already_imported_is_reused_and_copies_nothing() {
    let (_dir, _store, ci, out) = exported();
    let first = ci.import_baseline(&out).unwrap();
    let rows = count(&ci, "SELECT count(*) FROM targets");
    let again = ci.import_baseline(&out).unwrap();
    assert_eq!(
        again,
        Imported {
            run: first.run,
            reused: true
        }
    );
    assert_eq!(count(&ci, "SELECT count(*) FROM runs"), 1);
    assert_eq!(count(&ci, "SELECT count(*) FROM targets"), rows);
}

#[test]
fn the_database_a_baseline_came_from_already_holds_it() {
    let (_dir, store, _ci, out) = exported();
    let imported = store.import_baseline(&out).unwrap();
    assert_eq!(
        imported,
        Imported {
            run: RunId(1),
            reused: true
        }
    );
}

#[test]
fn a_file_that_is_not_a_baseline_is_refused_and_left_alone() {
    let (dir, store, ci, out) = exported();
    let refused = |path: &Path| ci.import_baseline(path).unwrap_err();

    let missing = dir.path().join("missing.duckdb");
    assert!(matches!(refused(&missing), Error::Db(_)));
    assert!(!missing.exists());

    let empty = dir.path().join("empty.duckdb");
    duckdb::Connection::open(&empty).unwrap();
    assert!(matches!(refused(&empty), Error::NotBruce));

    // Two runs: the history database itself.
    run(&store, 1, None, true);
    let history = dir.path().join("src.duckdb");
    drop(store);
    assert!(matches!(refused(&history), Error::NotBaseline(_)));

    // Another schema version.
    let old = dir.path().join("old.duckdb");
    std::fs::copy(&out, &old).unwrap();
    duckdb::Connection::open(&old)
        .unwrap()
        .execute(
            "UPDATE meta SET value = '3' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    assert!(matches!(
        refused(&old),
        Error::SchemaVersion {
            expected: SCHEMA_VERSION,
            ..
        }
    ));

    // A run that never finished.
    let unfinished = dir.path().join("unfinished.duckdb");
    std::fs::copy(&out, &unfinished).unwrap();
    duckdb::Connection::open(&unfinished)
        .unwrap()
        .execute("UPDATE runs SET finished_at = NULL", [])
        .unwrap();
    assert!(matches!(refused(&unfinished), Error::NotBaseline(_)));

    assert_eq!(count(&ci, "SELECT count(*) FROM runs"), 0);
}

/// Two runs of one database: run 1 holds a redirect from `/old` to `/new`,
/// the page `/new` with an anchor and a fact, and `/gone`, a 404; run 2 is
/// the CI run, holding one link and nothing else.
fn two_runs() -> (Store, RunId, RunId) {
    let store = Store::open_in_memory(&registry()).unwrap();
    let from = store
        .begin_run(Mode::Crawl, "abc", None, &["https://a.test/".into()])
        .unwrap();
    let to = store
        .begin_run(Mode::Local, "abc", Some(from), &["https://a.test/".into()])
        .unwrap();
    let conn = store.connection();
    for sql in [
        "INSERT INTO targets VALUES (1, 10, 'https://a.test/old', 301, 'https://a.test/new', \
         [{'url': 'https://a.test/old', 'status': 301, 'refresh': false}], 'ok', NULL, NULL, 5, 'live')",
        "INSERT INTO targets VALUES (1, 11, 'https://a.test/new', 200, NULL, [], 'ok', NULL, NULL, 5, 'live')",
        "INSERT INTO targets VALUES (1, 12, 'https://a.test/gone', 404, NULL, [], 'broken_url', 'status', NULL, 5, 'live')",
        "INSERT INTO pages VALUES (1, 11, 'New', 'text/html', 5, NULL, 'generic', NULL, NULL, NULL, NULL, true, false, NULL)",
        "INSERT INTO page_facts VALUES (1, 11, 'k', 'v')",
        "INSERT INTO anchors VALUES (1, 11, 'x', 'h2', 'content')",
        "INSERT INTO links VALUES (2, 20, 10, 'x', '/old#x', 'anchor', NULL)",
    ] {
        conn.execute(sql, []).unwrap();
    }
    (store, from, to)
}

#[test]
fn a_copied_redirect_brings_its_landing_page_and_anchors_and_is_answered_by_the_baseline() {
    let (store, from, to) = two_runs();
    assert!(store.copy_target(from, to, 10).unwrap());
    assert_eq!(
        count(&store, "SELECT count(*) FROM targets WHERE run_id = 2"),
        2
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM targets WHERE run_id = 2 AND answered_by = 'baseline'"
        ),
        2
    );
    assert_eq!(
        count(
            &store,
            "SELECT len(redirect_chain) FROM targets WHERE run_id = 2 AND url_id = 10"
        ),
        1
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM pages WHERE run_id = 2 AND url_id = 11"
        ),
        1
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM anchors WHERE run_id = 2 AND url_id = 11"
        ),
        1
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM page_facts WHERE run_id = 2"),
        0
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM links WHERE run_id = 2"),
        1
    );
    // The copy is what `link_verdicts` needs: the link's fragment is found on the landing page.
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM link_verdicts WHERE run_id = 2 AND outcome = 'ok'"
        ),
        1
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM targets WHERE run_id = 1"),
        3
    );
}

#[test]
fn a_target_the_run_already_holds_is_left_alone_and_a_missing_one_copies_nothing() {
    let (store, from, to) = two_runs();
    // The landing page was built: its row says so and is not replaced.
    store
        .connection()
        .execute(
            "INSERT INTO targets VALUES (2, 11, 'https://a.test/new', 200, NULL, [], 'ok', NULL, NULL, 5, 'local')",
            [],
        )
        .unwrap();
    assert!(store.copy_target(from, to, 10).unwrap());
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM targets WHERE run_id = 2 AND url_id = 11"
        ),
        1
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM targets WHERE run_id = 2 AND url_id = 11 AND answered_by = 'local'"
        ),
        1
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM pages WHERE run_id = 2"),
        0
    );

    assert!(store.copy_target(from, to, 10).unwrap());
    assert_eq!(
        count(&store, "SELECT count(*) FROM targets WHERE run_id = 2"),
        2
    );

    assert!(!store.copy_target(from, to, 99).unwrap());
    assert!(store.copy_target(from, to, 12).unwrap());
    assert_eq!(
        count(&store, "SELECT count(*) FROM targets WHERE run_id = 2"),
        3
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM pages WHERE run_id = 2"),
        0
    );
}

#[test]
fn the_index_lists_the_targets_of_one_run() {
    let (store, from, to) = two_runs();
    let index = store.baseline_index(from).unwrap();
    assert!([10, 11, 12].iter().all(|id| index.holds(*id)));
    assert!(!index.holds(13));
    let none = store.baseline_index(to).unwrap();
    assert!(!none.holds(10));
}

/// A copy the writer is asked for lands after the rows sent before it, so a
/// landing page the run wrote itself, still in an appender, is not doubled.
#[test]
fn a_copy_through_the_writer_follows_the_rows_sent_before_it() {
    use crate::store::{AnsweredBy, Outcome, Row, TargetRow, Writer};

    let (store, from, to) = two_runs();
    let writer = Writer::start(store, 16);
    let handle = writer.handle();
    handle
        .send(Row::Target(TargetRow {
            run_id: to.0,
            url_id: 11,
            url: "https://a.test/new".into(),
            status: Some(200),
            final_url: None,
            redirect_chain: Vec::new(),
            outcome: Outcome::Ok,
            reason: None,
            checked_at: None,
            duration_ms: None,
            answered_by: AnsweredBy::Local,
        }))
        .unwrap();
    handle.copy_target(from, to, 10).unwrap();
    let store = writer.finish().unwrap();
    assert_eq!(
        count(&store, "SELECT count(*) FROM targets WHERE run_id = 2"),
        2
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM targets WHERE run_id = 2 AND url_id = 11 AND answered_by = 'local'"
        ),
        1
    );
}
