use super::*;
use crate::profile::Registry;
use crate::store::Mode;

use std::time::SystemTime;

use duckdb::params;

fn store() -> Store {
    Store::open_in_memory(&Registry::new(Box::new(crate::profile::generic::Generic))).unwrap()
}

/// Starts a run holding one row in each table, begun `days` days ago and
/// finished if `finished`.
fn run(store: &Store, days: u32, finished: bool) -> RunId {
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let conn = store.connection();
    let id = run.0;
    for sql in [
        "INSERT INTO targets (answered_by, run_id, url_id, url, outcome) VALUES ('live', ?, 1, 'https://a.test/', 'ok')",
        "INSERT INTO pages VALUES (?, 1, NULL, NULL, NULL, NULL, 'whole', 'generic', NULL, NULL, NULL, NULL, true, false, NULL)",
        "INSERT INTO page_facts VALUES (?, 1, 'k', 'v')",
        "INSERT INTO anchors VALUES (?, 1, 'x', 'h2', 'content')",
        "INSERT INTO links VALUES (?, 1, 1, NULL, '/', 'anchor', NULL)",
        "INSERT INTO bodies VALUES (?, 1, '<p>')",
    ] {
        conn.execute(sql, [id]).unwrap();
    }
    conn.execute(
        "UPDATE runs SET started_at = ? WHERE run_id = ?",
        params![
            crate::store::rows::timestamp(
                SystemTime::now() - Duration::from_secs(u64::from(days) * DAY)
            ),
            id
        ],
    )
    .unwrap();
    if finished {
        store.finish_run(run).unwrap();
    }
    run
}

fn left(store: &Store) -> Vec<u64> {
    let mut stmt = store
        .connection()
        .prepare("SELECT run_id FROM runs ORDER BY run_id")
        .unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn rows_of(store: &Store, run: RunId) -> i64 {
    TABLES
        .iter()
        .map(|t| {
            store
                .connection()
                .query_row(
                    &format!("SELECT count(*) FROM {t} WHERE run_id = ?"),
                    [run.0],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap()
        })
        .sum()
}

const DAY: u64 = 86_400;

#[test]
fn keep_deletes_every_run_older_than_the_latest_finished_n() {
    let s = store();
    for _ in 0..4 {
        run(&s, 0, true);
    }
    assert_eq!(s.prune(Rule::Keep(2)).unwrap(), [RunId(1), RunId(2)]);
    assert_eq!(left(&s), [3, 4]);
    assert_eq!(rows_of(&s, RunId(1)), 0);
    assert_eq!(rows_of(&s, RunId(3)), 7);
}

#[test]
fn keep_counts_finished_runs_and_deletes_an_older_unfinished_one() {
    let s = store();
    run(&s, 0, false); // 1: dead, older than the finished runs
    run(&s, 0, true); // 2
    run(&s, 0, false); // 3: between finished runs
    run(&s, 0, true); // 4
    assert_eq!(
        s.prune(Rule::Keep(1)).unwrap(),
        [RunId(1), RunId(2), RunId(3)]
    );
    assert_eq!(left(&s), [4]);
}

#[test]
fn an_unfinished_run_after_the_latest_finished_one_is_kept() {
    let s = store();
    run(&s, 0, true);
    run(&s, 0, true);
    run(&s, 0, false); // the last attempt
    assert_eq!(s.prune(Rule::Keep(1)).unwrap(), [RunId(1)]);
    assert_eq!(left(&s), [2, 3]);
    assert_eq!(rows_of(&s, RunId(3)), 7);
}

#[test]
fn keep_with_fewer_finished_runs_than_n_deletes_nothing_it_can_anchor() {
    let s = store();
    run(&s, 0, false);
    assert!(s.prune(Rule::Keep(3)).unwrap().is_empty());
    run(&s, 0, true);
    assert_eq!(s.prune(Rule::Keep(3)).unwrap(), [RunId(1)]);
    assert_eq!(left(&s), [2]);
}

#[test]
fn older_than_deletes_the_runs_started_before_the_cutoff() {
    let s = store();
    run(&s, 40, true);
    run(&s, 20, true);
    run(&s, 1, true);
    let age = Duration::from_secs(30 * DAY);
    assert_eq!(s.prune(Rule::OlderThan(age)).unwrap(), [RunId(1)]);
    assert_eq!(left(&s), [2, 3]);
}

#[test]
fn older_than_keeps_the_last_attempt_but_not_a_dead_one() {
    let s = store();
    run(&s, 90, false); // 1: dead
    run(&s, 80, true); // 2
    run(&s, 70, false); // 3: the last attempt, though old
    let age = Duration::from_secs(30 * DAY);
    assert_eq!(s.prune(Rule::OlderThan(age)).unwrap(), [RunId(1), RunId(2)]);
    assert_eq!(left(&s), [3]);
}

#[test]
fn older_than_with_no_finished_run_keeps_every_unfinished_one() {
    let s = store();
    run(&s, 90, false);
    assert!(
        s.prune(Rule::OlderThan(Duration::from_secs(DAY)))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_prune_with_nothing_to_delete_changes_nothing() {
    let s = store();
    run(&s, 0, true);
    assert!(s.prune(Rule::Keep(1)).unwrap().is_empty());
    assert_eq!(left(&s), [1]);
    assert_eq!(rows_of(&s, RunId(1)), 7);
}

#[test]
fn a_pruned_database_still_numbers_runs_in_order() {
    let s = store();
    run(&s, 0, true);
    run(&s, 0, true);
    s.prune(Rule::Keep(1)).unwrap();
    assert_eq!(run(&s, 0, true), RunId(3));
}

#[test]
fn a_pruned_file_reopens_holding_what_was_kept() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.duckdb");
    let registry = Registry::new(Box::new(crate::profile::generic::Generic));
    {
        let s = Store::open(&path, &registry).unwrap();
        run(&s, 0, true);
        run(&s, 0, true);
        assert_eq!(s.prune(Rule::Keep(1)).unwrap(), [RunId(1)]);
    }
    let s = Store::open(&path, &registry).unwrap();
    assert_eq!(left(&s), [2]);
}

/// Makes `run` name `baseline` as its `baseline_run`.
fn names(store: &Store, run: RunId, baseline: RunId) {
    store
        .connection()
        .execute(
            "UPDATE runs SET baseline_run = ? WHERE run_id = ?",
            [baseline.0, run.0],
        )
        .unwrap();
}

#[test]
fn keep_spares_the_baseline_a_kept_run_names_and_does_not_count_it() {
    let s = store();
    run(&s, 0, true); // 1
    let baseline = run(&s, 0, true); // 2
    run(&s, 0, true); // 3
    let ci = run(&s, 0, true); // 4
    names(&s, ci, baseline);
    assert_eq!(s.prune(Rule::Keep(1)).unwrap(), [RunId(1), RunId(3)]);
    assert_eq!(left(&s), [2, 4]);
    assert_eq!(rows_of(&s, baseline), 7);
}

#[test]
fn older_than_spares_the_baseline_a_kept_run_names() {
    let s = store();
    let baseline = run(&s, 40, true); // 1
    run(&s, 40, true); // 2
    let ci = run(&s, 1, true); // 3
    names(&s, ci, baseline);
    let age = Duration::from_secs(30 * DAY);
    assert_eq!(s.prune(Rule::OlderThan(age)).unwrap(), [RunId(2)]);
    assert_eq!(left(&s), [1, 3]);
}

#[test]
fn a_spared_baseline_spares_its_own_baseline() {
    let s = store();
    let first = run(&s, 0, true); // 1
    let second = run(&s, 0, true); // 2
    let ci = run(&s, 0, true); // 3
    names(&s, second, first);
    names(&s, ci, second);
    assert!(s.prune(Rule::Keep(1)).unwrap().is_empty());
    assert_eq!(left(&s), [1, 2, 3]);
}

#[test]
fn the_baseline_of_a_deleted_run_goes_with_it() {
    let s = store();
    let baseline = run(&s, 0, true); // 1
    let old = run(&s, 0, true); // 2
    run(&s, 0, true); // 3
    names(&s, old, baseline);
    assert_eq!(s.prune(Rule::Keep(1)).unwrap(), [RunId(1), RunId(2)]);
    assert_eq!(left(&s), [3]);
}

#[test]
fn the_baseline_an_unfinished_last_attempt_names_is_spared() {
    let s = store();
    let baseline = run(&s, 0, true); // 1
    run(&s, 0, true); // 2
    let attempt = run(&s, 0, false); // 3
    names(&s, attempt, baseline);
    assert_eq!(s.prune(Rule::Keep(1)).unwrap(), Vec::<RunId>::new());
    assert_eq!(left(&s), [1, 2, 3]);
}
