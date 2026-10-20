use std::cell::RefCell;
use std::time::{Duration, UNIX_EPOCH};

use super::*;
use crate::profile::{Extract, Profile, Sink, View};

/// A profile that ships one view over `page_facts`.
struct Viewing;

impl Profile for Viewing {
    fn name(&self) -> &'static str {
        "viewing"
    }

    fn detect(&self, _: &str) -> bool {
        false
    }

    fn extract<'h>(&self, _: &'h RefCell<Sink>) -> Extract<'h> {
        Extract::region(&[])
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
        root: Some("https://example.test/docs/".into()),
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
        store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap()
    };
    let store = Store::open(&path, &registry()).unwrap();
    assert_eq!(count(&store, "SELECT count(*) FROM runs"), 1);
    assert_eq!(
        store.begin_run(Mode::Local, "abc", Some(run), &[]).unwrap(),
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
    let first = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let second = store
        .begin_run(Mode::Local, "abc", Some(first), &[])
        .unwrap();
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
        "INSERT INTO targets (answered_by, run_id, url_id, url, outcome) VALUES ('live', 1, 1, 'https://a.test/', 'fine')",
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
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
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
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
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
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let conn = store.connection();
    for (id, url) in [(1, "https://a.test/one"), (2, "https://a.test/two")] {
        conn.execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, outcome) VALUES ('live', 1, ?, ?, 'ok')",
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

fn target(run: RunId, url_id: u64, chain: Vec<Hop>) -> TargetRow {
    TargetRow {
        run_id: run.0,
        url_id,
        url: format!("https://a.test/{url_id}"),
        status: Some(200),
        final_url: None,
        redirect_chain: chain,
        outcome: Outcome::Ok,
        reason: None,
        checked_at: None,
        duration_ms: Some(30),
        answered_by: AnsweredBy::Live,
    }
}

fn hop(url: &str, status: u16, refresh: bool) -> Hop {
    Hop {
        url: url.into(),
        status,
        refresh,
    }
}

#[test]
fn a_target_records_where_its_answer_came_from() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let writer = Writer::start(store, 4);
    let h = writer.handle();
    let by = |id, chain, answered_by| {
        let mut t = target(run, id, chain);
        t.answered_by = answered_by;
        Row::Target(t)
    };
    // Both write paths: the appender, and the statement a redirect chain needs.
    h.send(by(1, Vec::new(), AnsweredBy::Local)).unwrap();
    h.send(by(2, vec![hop("a", 301, false)], AnsweredBy::Baseline))
        .unwrap();
    h.send(by(3, Vec::new(), AnsweredBy::Live)).unwrap();
    let store = writer.finish().unwrap();
    let got: Vec<String> = store
        .connection()
        .prepare("SELECT answered_by FROM targets ORDER BY url_id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(got, ["local", "baseline", "live"]);
    let bad = store.connection().execute(
        "INSERT INTO targets (answered_by, run_id, url_id, url, outcome) VALUES ('cache', 1, 9, 'u', 'ok')",
        [],
    );
    assert!(bad.is_err());
}

#[test]
fn a_targets_chain_keeps_each_hops_url_status_and_kind() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let writer = Writer::start(store, 4);
    let h = writer.handle();
    h.send(Row::Target(target(run, 1, Vec::new()))).unwrap();
    // A quote and a comma in a URL must reach the column as written.
    let odd = "https://a.test/it's,here?a=1&b=[2]";
    let mut moved = target(run, 2, vec![hop(odd, 301, false), hop("b", 200, true)]);
    moved.final_url = Some("https://a.test/final".into());
    h.send(Row::Target(moved)).unwrap();
    h.send(Row::Target(target(run, 3, vec![hop("c", 302, false)])))
        .unwrap();
    let store = writer.finish().unwrap();
    assert_eq!(count(&store, "SELECT count(*) FROM targets"), 3);
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM targets WHERE redirect_chain IS NULL AND url_id = 1"
        ),
        1
    );
    let chain: (String, u16, bool, u16, bool, usize) = store
        .connection()
        .query_row(
            "SELECT redirect_chain[1].url, redirect_chain[1].status, redirect_chain[1].refresh, \
             redirect_chain[2].status, redirect_chain[2].refresh, len(redirect_chain) \
             FROM targets WHERE url_id = 2",
            [],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(chain, (odd.to_string(), 301, false, 200, true, 2));
    let one: (String, String) = store
        .connection()
        .query_row(
            "SELECT redirect_chain[1].url, outcome FROM targets WHERE url_id = 3",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(one, ("c".to_string(), "ok".to_string()));
}

#[test]
fn a_targets_time_and_outcome_round_trip_on_both_write_paths() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let at = UNIX_EPOCH + Duration::from_micros(1_700_000_000_123_456);
    let writer = Writer::start(store, 4);
    let h = writer.handle();
    for (id, chain) in [(1, Vec::new()), (2, vec![hop("x", 302, false)])] {
        let mut t = target(run, id, chain);
        t.status = None;
        t.outcome = Outcome::BrokenUrl;
        t.reason = Some("dns".into());
        t.checked_at = Some(at);
        h.send(Row::Target(t)).unwrap();
    }
    h.send(Row::Target(target(run, 3, Vec::new()))).unwrap();
    let store = writer.finish().unwrap();
    let sql = "SELECT count(*) FROM targets WHERE status IS NULL AND outcome = 'broken_url' \
               AND reason = 'dns' AND duration_ms = 30 \
               AND epoch_us(checked_at) = 1700000000123456";
    assert_eq!(count(&store, sql), 2);
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM targets WHERE checked_at IS NULL"
        ),
        1
    );
}

#[test]
fn anchors_and_links_are_appended_with_their_kind() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    let writer = Writer::start(store, 4);
    let h = writer.handle();
    h.send(Row::Anchor(AnchorRow {
        run_id: run.0,
        url_id: 1,
        anchor: "intro".into(),
        tag: "h2".into(),
        placement: "content",
    }))
    .unwrap();
    for kind in ["anchor", "sitemap"] {
        h.send(Row::Link(LinkRow {
            run_id: run.0,
            src_url_id: 1,
            target_url_id: 2,
            fragment: (kind == "anchor").then(|| "intro".to_string()),
            raw_href: "../b/".into(),
            kind,
            xref_refspec: None,
        }))
        .unwrap();
    }
    let store = writer.finish().unwrap();
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM anchors WHERE placement = 'content'"
        ),
        1
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM links WHERE kind = 'sitemap' AND xref_refspec IS NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM links WHERE fragment = 'intro'"
        ),
        1
    );
}

#[test]
fn a_link_stores_no_verdict() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let columns: Vec<String> = store
        .connection()
        .prepare("SELECT column_name FROM information_schema.columns WHERE table_name = 'links'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    for gone in ["outcome", "reason", "redirected"] {
        assert!(
            !columns.iter().any(|c| c == gone),
            "links.{gone} is a view's"
        );
    }
}

#[test]
fn a_run_records_its_roots_and_a_page_its_root() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let roots = [
        "https://a.test/docs/".to_string(),
        "https://b.test/".to_string(),
    ];
    let run = store.begin_run(Mode::Crawl, "abc", None, &roots).unwrap();
    let none = store.begin_run(Mode::Local, "abc", None, &[]).unwrap();
    let listed: String = store
        .connection()
        .query_row(
            "SELECT array_to_string(roots, ' ') FROM runs WHERE run_id = ?",
            [run.0],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(listed, "https://a.test/docs/ https://b.test/");
    assert_eq!(
        count(
            &store,
            &format!("SELECT len(roots) FROM runs WHERE run_id = {}", none.0)
        ),
        0
    );
    let writer = Writer::start(store, 4);
    let h = writer.handle();
    h.send(Row::Page(page(run, 1))).unwrap();
    h.send(Row::Page(PageRow {
        root: None,
        ..page(run, 2)
    }))
    .unwrap();
    let store = writer.finish().unwrap();
    assert_eq!(
        count(
            &store,
            "SELECT count(*) FROM pages WHERE root = 'https://example.test/docs/'"
        ),
        1
    );
    assert_eq!(
        count(&store, "SELECT count(*) FROM pages WHERE root IS NULL"),
        1
    );
}

#[test]
fn a_database_of_schema_version_1_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.duckdb");
    drop(Store::open(&path, &registry()).unwrap());
    Connection::open(&path)
        .unwrap()
        .execute(
            "UPDATE meta SET value = '1' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    let err = Store::open(&path, &registry()).err().unwrap();
    assert!(
        matches!(err, Error::SchemaVersion { ref found, expected: SCHEMA_VERSION } if found == "1")
    );
}

/// The watch for UF1: this fails on purpose at the duckdb release whose
/// appender takes a list, when `Tables::insert_redirected` can go.
// upstream: UF1
#[test]
fn the_appender_still_takes_no_list() {
    use duckdb::types::Value;
    let store = Store::open_in_memory(&registry()).unwrap();
    let mut appender = store.connection().appender("targets").unwrap();
    let chain = Value::List(vec![Value::Struct(
        vec![
            ("url".to_string(), Value::Text("x".into())),
            ("status".to_string(), Value::USmallInt(301)),
            ("refresh".to_string(), Value::Boolean(false)),
        ]
        .into(),
    )]);
    let err = appender
        .append_row(params![
            1u64,
            1u64,
            "u",
            Value::Null,
            Value::Null,
            chain,
            "ok",
            Value::Null,
            Value::Null,
            Value::Null
        ])
        .unwrap_err();
    assert!(err.to_string().contains("not yet supported"), "{err}");
}
