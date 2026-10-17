use std::cell::RefCell;

use super::*;
use crate::profile::{ElementHandlers, Facts, Profile, View};

/// A profile that ships one view over `page_facts`.
struct Viewing;

impl Profile for Viewing {
    fn name(&self) -> &'static str {
        "viewing"
    }

    fn detect(&self, _: &str) -> bool {
        false
    }

    fn extract<'h>(&self, _: &'h RefCell<Facts>) -> ElementHandlers<'h> {
        Vec::new()
    }

    fn resolve(&self, _: &str, _: &Url, _: &dyn Lookup) -> Result<Option<Url>, BoxError> {
        Ok(None)
    }

    fn views(&self) -> &'static [View] {
        &[View {
            name: "viewing_colours",
            select: "SELECT url_id, value AS colour FROM page_facts \
                     WHERE key = 'viewing.colour'",
        }]
    }
}

fn registry() -> Registry {
    Registry::new(Box::new(Viewing))
}

fn count(store: &Store, sql: &str) -> i64 {
    store.connection().query_row(sql, [], |r| r.get(0)).unwrap()
}

fn page(run: RunId, url_id: u64) -> PageRow {
    PageRow {
        run_id: run.0,
        url_id,
        title: Some("A page".into()),
        content_type: Some("text/html".into()),
        size: Some(10),
        content_hash: Some(vec![1, 2, 3]),
        profile: "viewing".into(),
        source_url: None,
        source_repo: None,
        source_ref: None,
        source_path: None,
        in_sitemap: true,
        via_links: false,
    }
}

fn fact(run: RunId, url_id: u64, key: &str, value: &str) -> Row {
    Row::Fact(FactRow {
        run_id: run.0,
        url_id,
        key: key.into(),
        value: value.into(),
    })
}

#[test]
fn a_new_database_records_its_schema_version() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let version: String = store
        .connection()
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION.to_string());
}

#[test]
fn a_file_reopens_with_its_rows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.duckdb");
    let run = {
        let store = Store::open(&path, &registry()).unwrap();
        store.begin_run(Mode::Crawl, "abc", None).unwrap()
    };
    let store = Store::open(&path, &registry()).unwrap();
    assert_eq!(count(&store, "SELECT count(*) FROM runs"), 1);
    assert_eq!(
        store.begin_run(Mode::Local, "abc", Some(run)).unwrap(),
        RunId(2)
    );
}

#[test]
fn another_schema_version_is_refused_naming_both() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.duckdb");
    let store = Store::open(&path, &registry()).unwrap();
    store
        .connection()
        .execute(
            "UPDATE meta SET value = '999' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    drop(store);
    let err = Store::open(&path, &registry()).err().unwrap();
    assert_eq!(
        err.to_string(),
        format!("database has schema version 999, but this bruce reads version {SCHEMA_VERSION}")
    );
}

#[test]
fn a_database_that_is_not_ours_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("other.duckdb");
    Connection::open(&path)
        .unwrap()
        .execute_batch("CREATE TABLE notes (x INTEGER)")
        .unwrap();
    let err = Store::open(&path, &registry()).err().unwrap();
    assert!(matches!(err, Error::NotBruce), "{err}");
    let conn = Connection::open(&path).unwrap();
    assert!(!table_exists(&conn, "meta").unwrap());
}

#[test]
fn a_profiles_views_are_created_on_open() {
    let store = Store::open_in_memory(&registry()).unwrap();
    store
        .connection()
        .execute(
            "INSERT INTO page_facts VALUES (1, 7, 'viewing.colour', 'red')",
            [],
        )
        .unwrap();
    assert_eq!(count(&store, "SELECT count(*) FROM viewing_colours"), 1);
}

#[test]
fn a_run_is_numbered_in_order_and_finished() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let first = store.begin_run(Mode::Crawl, "abc", None).unwrap();
    let second = store.begin_run(Mode::Local, "abc", Some(first)).unwrap();
    assert_eq!((first, second), (RunId(1), RunId(2)));
    store.finish_run(first).unwrap();
    let finished = "SELECT count(*) FROM runs WHERE finished_at IS NOT NULL";
    assert_eq!(count(&store, finished), 1);
    assert_eq!(
        count(&store, "SELECT baseline_run FROM runs WHERE run_id = 2"),
        1
    );
}

#[test]
fn the_schema_refuses_an_outcome_outside_d16() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let bad = store.connection().execute(
        "INSERT INTO targets (run_id, url_id, url, outcome) VALUES (1, 1, 'https://a.test/', 'fine')",
        [],
    );
    assert!(bad.is_err());
    let bad = store.connection().execute(
        "INSERT INTO anchors VALUES (1, 1, 'x', 'h2', 'sidebar')",
        [],
    );
    assert!(bad.is_err());
}

#[test]
fn the_writer_appends_what_is_sent_and_returns_the_store() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None).unwrap();
    let writer = Writer::start(store, 1);
    let handle = writer.handle();
    for url_id in 0..50 {
        handle.send(Row::Page(page(run, url_id))).unwrap();
        handle
            .send(fact(run, url_id, "viewing.colour", "red"))
            .unwrap();
    }
    let store = writer.finish().unwrap();
    assert_eq!(count(&store, "SELECT count(*) FROM pages"), 50);
    assert_eq!(count(&store, "SELECT count(*) FROM viewing_colours"), 50);
    assert_eq!(handle.send(Row::Page(page(run, 99))), Err(WriterGone));
}

#[test]
fn senders_on_several_threads_share_one_writer() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None).unwrap();
    let writer = Writer::start(store, 2);
    let threads: Vec<_> = (0..4u64)
        .map(|t| {
            let handle = writer.handle();
            std::thread::spawn(move || {
                for i in 0..25 {
                    handle.send(Row::Page(page(run, t * 100 + i))).unwrap();
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let store = writer.finish().unwrap();
    assert_eq!(
        count(&store, "SELECT count(DISTINCT url_id) FROM pages"),
        100
    );
}

#[test]
fn a_lookup_finds_the_pages_carrying_every_fact() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None).unwrap();
    let conn = store.connection();
    for (id, url) in [(1, "https://a.test/one"), (2, "https://a.test/two")] {
        conn.execute(
            "INSERT INTO targets (run_id, url_id, url, outcome) VALUES (1, ?, ?, 'ok')",
            params![id, url],
        )
        .unwrap();
    }
    let writer = Writer::start(store, 8);
    let h = writer.handle();
    for id in [1, 2] {
        h.send(Row::Page(page(run, id))).unwrap();
        h.send(fact(run, id, "viewing.colour", "red")).unwrap();
    }
    h.send(fact(run, 2, "viewing.size", "big")).unwrap();
    let store = writer.finish().unwrap();
    let lookup = store.lookup(run);
    let urls = |facts: &[(&str, &str)]| -> Vec<String> {
        let mut u: Vec<_> = lookup
            .pages_with_facts(facts)
            .unwrap()
            .iter()
            .map(Url::to_string)
            .collect();
        u.sort();
        u
    };
    assert_eq!(urls(&[("viewing.colour", "red")]).len(), 2);
    assert_eq!(
        urls(&[("viewing.colour", "red"), ("viewing.size", "big")]),
        ["https://a.test/two"]
    );
    assert!(urls(&[("viewing.colour", "blue")]).is_empty());
    assert!(
        store
            .lookup(RunId(2))
            .pages_with_facts(&[])
            .unwrap()
            .is_empty()
    );
}
