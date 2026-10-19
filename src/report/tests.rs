use duckdb::params;

use super::*;
use crate::extract::Kind;
use crate::profile::Registry;
use crate::profile::generic::Generic;
use crate::store::RunId;

const ROOT: &str = "https://e.test/";

fn url(id: u64) -> String {
    format!("https://e.test/{id}")
}

fn target(store: &Store, run: u64, id: u64, status: u16, outcome: &str, reason: Option<&str>) {
    store
        .connection()
        .execute(
            "INSERT INTO targets (run_id, url_id, url, status, outcome, reason) \
             VALUES (?, ?, ?, ?, ?, ?)",
            params![run, id, url(id), status, outcome, reason],
        )
        .unwrap();
}

fn page(store: &Store, run: u64, id: u64, repo: Option<&str>, root: Option<&str>) {
    let conn = store.connection();
    target(store, run, id, 200, "ok", None);
    conn.execute(
        "INSERT INTO pages VALUES (?, ?, NULL, NULL, NULL, NULL, 'generic', NULL, ?, NULL, ?, \
         false, true, ?)",
        params![run, id, repo, repo.map(|_| format!("{id}.adoc")), root],
    )
    .unwrap();
}

fn link(store: &Store, run: u64, src: u64, to: u64, fragment: Option<&str>, kind: &str) {
    store
        .connection()
        .execute(
            "INSERT INTO links VALUES (?, ?, ?, ?, ?, ?, NULL)",
            params![
                run,
                src,
                to,
                fragment,
                format!("{src}>{to}#{}", fragment.unwrap_or("")),
                kind
            ],
        )
        .unwrap();
}

fn fact(store: &Store, run: u64, id: u64, key: &str, value: &str) {
    store
        .connection()
        .execute(
            "INSERT INTO page_facts VALUES (?, ?, ?, ?)",
            params![run, id, key, value],
        )
        .unwrap();
}

/// Run 1, finished, and run 2, which is not. Page 1 (repo `r/one`, component
/// `docs`) links to a 404, a missing fragment, a present one, a 503, a
/// skipped URL and, by image, another 404; page 2 (`r/two`, `other`) to the
/// 404 and to page 2 itself. Page 3 is crawled and links to nothing, page 4
/// is allowlisted.
fn store() -> Store {
    let store = Store::open_in_memory(&Registry::new(Box::new(Generic))).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    for (id, root) in [(1, true), (2, true), (3, true), (4, false)] {
        let repo = match id {
            1 => Some("r/one"),
            2 => Some("r/two"),
            _ => None,
        };
        page(&store, 1, id, repo, root.then_some(ROOT));
    }
    conn.execute(
        "INSERT INTO anchors VALUES (1, 2, 'sec', 'h2', 'content')",
        [],
    )
    .unwrap();
    target(&store, 1, 10, 404, "broken_url", None);
    target(&store, 1, 12, 503, "error", Some("server_error"));
    target(&store, 1, 13, 404, "broken_url", None);
    target(&store, 1, 14, 0, "unchecked", Some("skipped"));
    link(&store, 1, 1, 10, None, "anchor");
    link(&store, 1, 1, 2, Some("nope"), "anchor");
    link(&store, 1, 1, 2, Some("sec"), "anchor");
    link(&store, 1, 1, 12, None, "anchor");
    link(&store, 1, 1, 13, None, "image");
    link(&store, 1, 1, 14, None, "anchor");
    link(&store, 1, 2, 10, None, "anchor");
    link(&store, 1, 2, 2, None, "anchor");
    fact(&store, 1, 1, "antora.component", "handbook");
    fact(&store, 1, 2, "antora.component", "other");
    for (run, finished) in [(1, true), (2, false)] {
        conn.execute(
            "INSERT INTO runs VALUES (?, now(), CASE WHEN ? THEN now() END, 'crawl', 'x', 'h', \
             NULL, [])",
            params![run, finished],
        )
        .unwrap();
    }
    page(&store, 2, 1, Some("r/one"), Some(ROOT));
    target(&store, 2, 10, 404, "broken_url", None);
    link(&store, 2, 1, 10, None, "anchor");
    store
}

fn column(table: &Table, name: &str) -> Vec<String> {
    let at = table.columns.iter().position(|c| c == name).unwrap();
    table.rows.iter().map(|r| r[at].shown_for_test()).collect()
}

fn report(store: &Store, edit: impl FnOnce(&mut Request)) -> Output {
    let mut request = Request::new(Report::Broken);
    edit(&mut request);
    run(store, &request).unwrap()
}

impl Cell {
    fn shown_for_test(&self) -> String {
        match self {
            Cell::Text(s) => s.clone(),
            other => format!("{other:?}"),
        }
    }
}

#[test]
fn broken_lists_broken_urls_before_broken_anchors_with_their_source() {
    let out = report(&store(), |_| {});
    assert_eq!(out.run, RunId(1));
    assert!(!out.unfinished);
    assert_eq!(
        column(&out.table, "raw_href"),
        ["1>10#", "2>10#", "1>2#nope"]
    );
    assert_eq!(
        column(&out.table, "outcome"),
        ["broken_url", "broken_url", "broken_anchor"]
    );
    assert_eq!(
        column(&out.table, "source_repo"),
        ["r/one", "r/two", "r/one"]
    );
    assert_eq!(
        column(&out.table, "source_path"),
        ["1.adoc", "2.adoc", "1.adoc"]
    );
    assert_eq!(column(&out.table, "page")[0], url(1));
    assert_eq!(column(&out.table, "target")[0], url(10));
    assert_eq!(column(&out.table, "fragment")[2], "nope");
}

#[test]
fn a_kind_widens_broken_to_the_links_of_that_kind() {
    let out = report(&store(), |r| r.kinds = vec![Kind::Image]);
    assert_eq!(column(&out.table, "raw_href"), ["1>13#"]);
    let out = report(&store(), |r| r.kinds = Kind::ALL.to_vec());
    assert_eq!(out.table.rows.len(), 4);
}

#[test]
fn errors_lists_the_links_whose_outcome_is_error_alone() {
    let out = report(&store(), |r| r.report = Report::Errors);
    assert_eq!(column(&out.table, "raw_href"), ["1>12#"]);
    assert_eq!(column(&out.table, "reason"), ["server_error"]);
    assert_eq!(out.table.rows[0][7], Cell::Int(503));
}

#[test]
fn repo_and_fact_keep_the_links_of_the_pages_that_match() {
    let out = report(&store(), |r| r.repo = Some("r/two".into()));
    assert_eq!(column(&out.table, "raw_href"), ["2>10#"]);
    let out = report(&store(), |r| {
        r.facts = vec![("antora.component".into(), "handbook".into())];
    });
    assert_eq!(column(&out.table, "raw_href"), ["1>10#", "1>2#nope"]);
    let out = report(&store(), |r| {
        r.repo = Some("r/two".into());
        r.facts = vec![("antora.component".into(), "handbook".into())];
    });
    assert!(out.table.rows.is_empty());
}

#[test]
fn a_value_is_bound_and_never_read_as_sql() {
    let out = report(&store(), |r| r.repo = Some("x' OR '1'='1".into()));
    assert!(out.table.rows.is_empty());
}

#[test]
fn top_keeps_the_first_rows() {
    let out = report(&store(), |r| r.top = Some(2));
    assert_eq!(column(&out.table, "raw_href"), ["1>10#", "2>10#"]);
}

#[test]
fn stats_counts_each_crawled_page_and_totals_the_run() {
    let out = report(&store(), |r| r.report = Report::Stats);
    assert_eq!(
        out.table.columns,
        [
            "page",
            "links",
            "broken_url",
            "broken_anchor",
            "error",
            "unchecked"
        ]
    );
    // Page 4 is allowlisted, so it is no row; page 3 links to nothing.
    assert_eq!(column(&out.table, "page"), [url(1), url(2), url(3)]);
    let n = |i: usize| out.table.rows[i][1..].to_vec();
    let ints = |v: [i64; 5]| v.map(Cell::Int).to_vec();
    assert_eq!(n(0), ints([5, 1, 1, 1, 1]));
    assert_eq!(n(1), ints([2, 1, 0, 0, 0]));
    assert_eq!(n(2), ints([0, 0, 0, 0, 0]));
    assert_eq!(out.notes.len(), 1);
    assert_eq!(
        out.notes[0],
        "total: 3 pages, 7 links, 2 broken_url, 1 broken_anchor, 1 error, 1 unchecked\n\
         unchecked by reason:\n  skipped: 1 links, 1 targets"
    );
}

#[test]
fn stats_counts_a_target_once_however_many_links_name_it() {
    let store = store();
    link(&store, 1, 2, 14, None, "anchor");
    link(&store, 1, 3, 14, Some("x"), "anchor");
    let out = report(&store, |r| r.report = Report::Stats);
    assert!(
        out.notes[0].ends_with("unchecked by reason:\n  skipped: 3 links, 1 targets"),
        "{}",
        out.notes[0]
    );
    // The filters reach the target count: only page 2's link is left.
    let out = report(&store, |r| {
        r.report = Report::Stats;
        r.repo = Some("r/two".into());
    });
    assert!(out.notes[0].ends_with("skipped: 1 links, 1 targets"));
}

#[test]
fn stats_counts_the_kinds_it_is_given_and_top_leaves_the_total_whole() {
    let out = report(&store(), |r| {
        r.report = Report::Stats;
        r.kinds = vec![Kind::Image];
        r.top = Some(1);
    });
    assert_eq!(column(&out.table, "page"), [url(1)]);
    assert_eq!(out.table.rows[0][1..3], [Cell::Int(1), Cell::Int(1)]);
    assert!(out.notes[0].starts_with("total: 3 pages, 1 links, 1 broken_url"));
    assert!(!out.notes[0].contains("unchecked by reason"));
}

#[test]
fn stats_takes_the_same_filters() {
    let out = report(&store(), |r| {
        r.report = Report::Stats;
        r.repo = Some("r/two".into());
    });
    assert_eq!(column(&out.table, "page"), [url(2)]);
    assert!(out.notes[0].starts_with("total: 1 pages, 2 links"));
}

#[test]
fn the_latest_finished_run_is_read_unless_a_run_is_named() {
    let store = store();
    let out = report(&store, |_| {});
    assert_eq!((out.run, out.unfinished), (RunId(1), false));
    let out = report(&store, |r| r.run = Some(RunId(2)));
    assert_eq!((out.run, out.unfinished), (RunId(2), true));
    assert_eq!(column(&out.table, "raw_href"), ["1>10#"]);
    let missing = run(&store, &{
        let mut r = Request::new(Report::Broken);
        r.run = Some(RunId(9));
        r
    });
    assert!(matches!(missing, Err(Error::NoSuchRun(RunId(9)))));
}

#[test]
fn with_no_finished_run_a_report_says_so() {
    let store = Store::open_in_memory(&Registry::new(Box::new(Generic))).unwrap();
    store.install(VIEWS).unwrap();
    store
        .begin_run(crate::store::Mode::Crawl, "h", None, &[])
        .unwrap();
    let err = run(&store, &Request::new(Report::Broken)).unwrap_err();
    assert!(matches!(err, Error::NoFinishedRun));
    assert!(err.to_string().contains("--run"));
}
