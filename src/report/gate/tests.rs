use duckdb::params;

use super::super::tests::{ROOT, column, link, page, registry, target, url};
use crate::extract::Kind;
use crate::report::{Cell, Request};
use crate::report::{Error, Gate, Report, Store, VIEWS, gate, run};
use crate::store::RunId;

fn moved(store: &Store, run: u64, id: u64) {
    target(store, run, id, 200, "ok", None);
    store
        .connection()
        .execute(
            "UPDATE targets SET final_url = ?, redirect_chain = [] WHERE run_id = ? AND url_id = ?",
            params![url(2), run, id],
        )
        .unwrap();
}

/// Run 1 is the baseline and run 2 a CI run checked against it. Pages 1 and
/// 2 are built; page 5 is only in the baseline, and its link to 10 is broken
/// there. Page 2 has the anchor `sec` in run 2. From page 1:
///
/// - 10 is broken in both runs: already in the baseline.
/// - 11 was `ok` in the baseline: introduced, state `ok`.
/// - 12 is not in the baseline: introduced, state `new`.
/// - 2 at `#nope` is a link the baseline never had: introduced, `new`.
/// - 14 answers a 503: an error, listed.
/// - 20 redirects in run 2 only, 21 in both.
/// - 15 is a broken image: introduced, state `new`; the gate judges every kind.
///
/// Page 2 links to 13, `unchecked` in the baseline and broken now: introduced,
/// state `unchecked`.
fn store() -> Store {
    let store = Store::open_in_memory(&registry()).unwrap();
    store.install(VIEWS).unwrap();
    let conn = store.connection();
    for (run, mode, baseline) in [(1, "crawl", None), (2, "local", Some(1))] {
        conn.execute(
            "INSERT INTO runs VALUES (?, '2026-10-19 12:00:00+00', now(), ?, 'x', 'h', ?, [], [])",
            params![run, mode, baseline],
        )
        .unwrap();
    }
    for run in [1, 2] {
        page(&store, run, 1, Some("r/one"), Some(ROOT));
        page(&store, run, 2, Some("r/two"), Some(ROOT));
        target(&store, run, 10, 404, "broken_url", None);
        link(&store, run, 1, 10, None, "anchor");
    }
    page(&store, 1, 5, Some("r/five"), Some(ROOT));
    link(&store, 1, 5, 10, None, "anchor");
    conn.execute(
        "INSERT INTO anchors VALUES (2, 2, 'sec', 'h2', 'content')",
        [],
    )
    .unwrap();
    target(&store, 1, 11, 200, "ok", None);
    link(&store, 1, 1, 11, None, "anchor");
    target(&store, 2, 11, 404, "broken_url", None);
    link(&store, 2, 1, 11, None, "anchor");
    target(&store, 2, 12, 404, "broken_url", None);
    link(&store, 2, 1, 12, None, "anchor");
    link(&store, 2, 1, 2, Some("nope"), "anchor");
    link(&store, 2, 1, 2, Some("sec"), "anchor");
    target(&store, 1, 13, 0, "unchecked", Some("robots"));
    link(&store, 1, 2, 13, None, "anchor");
    target(&store, 2, 13, 404, "broken_url", None);
    link(&store, 2, 2, 13, None, "anchor");
    target(&store, 2, 14, 503, "error", Some("server_error"));
    link(&store, 2, 1, 14, None, "anchor");
    target(&store, 2, 15, 404, "broken_url", None);
    link(&store, 2, 1, 15, None, "image");
    moved(&store, 2, 20);
    link(&store, 2, 1, 20, None, "anchor");
    moved(&store, 1, 21);
    moved(&store, 2, 21);
    link(&store, 1, 1, 21, None, "anchor");
    link(&store, 2, 1, 21, None, "anchor");
    store
}

fn gate_of(store: &Store, edit: impl FnOnce(&mut Request)) -> Gate {
    let mut request = Request::new(Report::Introduced);
    request.run = Some(RunId(2));
    edit(&mut request);
    gate(store, &request).unwrap()
}

#[test]
fn introduced_is_broken_here_and_not_broken_in_the_baseline_and_names_its_state() {
    let g = gate_of(&store(), |_| {});
    assert!(g.fails());
    assert_eq!((g.run, g.baseline), (RunId(2), RunId(1)));
    assert_eq!(g.baseline_date, "2026-10-19");
    assert_eq!(
        g.introduced.columns,
        [
            "page",
            "raw_href",
            "target",
            "fragment",
            "kind",
            "outcome",
            "reason",
            "status",
            "baseline",
            "source_repo",
            "source_path"
        ]
    );
    // `broken_url` before `broken_anchor`, then by page.
    assert_eq!(
        column(&g.introduced, "raw_href"),
        ["1>11#", "1>12#", "1>15#", "2>13#", "1>2#nope"]
    );
    assert_eq!(
        column(&g.introduced, "baseline"),
        ["ok", "new", "new", "unchecked", "new"]
    );
    assert_eq!(g.introduced.rows[0][7], Cell::Int(404));
}

#[test]
fn breakage_the_baseline_had_is_counted_apart_and_a_link_on_a_page_not_built_is_no_one() {
    let g = gate_of(&store(), |_| {});
    assert_eq!(column(&g.already, "raw_href"), ["1>10#"]);
    assert!(!g.already.columns.contains(&"baseline".to_owned()));
}

#[test]
fn an_error_is_listed_and_never_introduced() {
    let g = gate_of(&store(), |_| {});
    assert_eq!(column(&g.errors, "raw_href"), ["1>14#"]);
    assert!(!column(&g.introduced, "raw_href").contains(&"1>14#".to_owned()));
}

#[test]
fn a_redirect_is_introduced_where_the_baseline_did_not_have_it() {
    let g = gate_of(&store(), |_| {});
    assert_eq!(column(&g.redirected, "raw_href"), ["1>20#"]);
    assert_eq!(column(&g.redirected, "landing"), [url(2)]);
}

#[test]
fn a_filter_that_keeps_nothing_leaves_a_gate_that_does_not_fail() {
    let g = gate_of(&store(), |r| r.repo = Some("r/none".into()));
    assert!(!g.fails());
    assert_eq!(
        g.text(false),
        "against baseline run 1, crawled on 2026-10-19\n\n\
         introduced breakage: none\n\n\
         inverse breakage: none\n\n\
         introduced redirected links: none\n\n\
         errors: none\n\n\
         breakage already in the baseline: none"
    );
}

#[test]
fn the_gate_reads_every_kind_unless_a_report_narrows_it() {
    let g = gate_of(&store(), |_| {});
    assert!(column(&g.introduced, "raw_href").contains(&"1>15#".to_owned()));
    let g = gate_of(&store(), |r| r.kinds = vec![Kind::Anchor]);
    assert!(!column(&g.introduced, "raw_href").contains(&"1>15#".to_owned()));
}

#[test]
fn the_text_names_the_sections_and_prints_the_old_breakage_on_request() {
    let g = gate_of(&store(), |_| {});
    let text = g.text(false);
    let at = |s: &str| text.find(s).unwrap_or_else(|| panic!("{s} in {text}"));
    assert!(at("introduced breakage: 5") < at("introduced redirected links: 1"));
    assert!(at("introduced redirected links: 1") < at("errors: 1"));
    assert!(at("errors: 1") < at("breakage already in the baseline: 1 (--all lists them)"));
    assert!(!text.contains("1>10#"));
    assert!(
        g.text(true)
            .contains("breakage already in the baseline: 1\n")
    );
    assert!(g.text(true).contains("1>10#"));
}

#[test]
fn json_is_one_object_whose_old_breakage_is_a_count_unless_asked_for() {
    let g = gate_of(&store(), |_| {});
    let json = g.json(false);
    assert!(json.starts_with(
        "{\"run\":2,\"baseline_run\":1,\"baseline_date\":\"2026-10-19\",\"introduced\":[{"
    ));
    assert!(json.ends_with("\"already_broken\":{\"count\":1,\"links\":null}}"));
    assert!(g.json(true).ends_with("\"links\":[{\"page\":\"https://e.test/1\",\"raw_href\":\"1>10#\",\
        \"target\":\"https://e.test/10\",\"fragment\":null,\"kind\":\"anchor\",\"outcome\":\"broken_url\",\
        \"reason\":null,\"status\":404,\"source_repo\":\"r/one\",\"source_path\":\"1.adoc\"}]}}"));
}

#[test]
fn the_introduced_report_prints_the_rows_and_the_rest_in_notes() {
    let mut request = Request::new(Report::Introduced);
    request.run = Some(RunId(2));
    let out = run(&store(), &request).unwrap();
    assert_eq!(out.table.rows.len(), 5);
    assert_eq!(
        out.notes[0],
        "against baseline run 1, crawled on 2026-10-19"
    );
    assert!(out.notes[1].starts_with("inverse breakage: none\n\nintroduced redirected links: 1\n"));
}

#[test]
fn a_run_that_names_no_baseline_is_refused() {
    let mut request = Request::new(Report::Introduced);
    request.run = Some(RunId(1));
    assert!(matches!(
        run(&store(), &request),
        Err(Error::NoBaseline(RunId(1)))
    ));
}
