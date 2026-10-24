use duckdb::params;

use super::super::tests::{ROOT, column, fact, link, registry};
use super::{counts, prefix};
use crate::report::{Cell, Gate, InverseMode, Report, Request, Store, VIEWS, gate, run};
use crate::store::RunId;

const SITE: &str = "https://e.test/en";

fn at(path: &str) -> String {
    format!("{SITE}/{path}")
}

/// A target of `run` at `path`, answered by `by`; a page also gets its row.
fn put(store: &Store, run: u64, id: u64, path: &str, outcome: &str, by: &str, page: bool) {
    let conn = store.connection();
    conn.execute(
        "INSERT INTO targets (answered_by, run_id, url_id, url, status, outcome) \
         VALUES (?, ?, ?, ?, 200, ?)",
        params![by, run, id, at(path), outcome],
    )
    .unwrap();
    if page {
        conn.execute(
            "INSERT INTO pages VALUES (?, ?, NULL, NULL, NULL, NULL, 'whole', 'generic', NULL, ?, NULL, \
             ?, false, true, ?)",
            params![run, id, format!("r/{id}"), format!("{id}.adoc"), ROOT],
        )
        .unwrap();
    }
}

fn anchor(store: &Store, run: u64, id: u64, name: &str) {
    store
        .connection()
        .execute(
            "INSERT INTO anchors VALUES (?, ?, ?, 'h2', 'content')",
            params![run, id, name],
        )
        .unwrap();
}

/// Run 1 is the baseline; run 2 a CI run over a build of the component
/// `handbook` (no version) and of `named` at `1.0`. Page 1, in `other`, is
/// outside the build and links to every target below; page 20, in `named` at
/// 2.0, is outside it too and links to 3 and 7:
///
/// - 2, a `handbook` page the build still has, at `#lost` (an anchor run 2
///   lacks) and at `#keep` (one it has);
/// - 3, a `handbook` page the build lacks; 4, a `handbook` image a built page still
///   names; 5, a `handbook` image no built page names and the build lacks; 10, one
///   the build keeps;
/// - 6, a page of `other`; 7, a `named/1.0` page the build lacks; 8, a
///   `named/2.0` page it lacks.
///
/// Page 9, in `handbook` and not built, links to 3: gone with its page.
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
    for (id, path) in [
        (1, "other/a/"),
        (2, "handbook/kept/"),
        (3, "handbook/gone/"),
        (6, "other/b/"),
        (7, "named/1.0/gone/"),
        (8, "named/2.0/gone/"),
        (9, "handbook/dead/"),
        (20, "named/2.0/q/"),
        (21, "named/1.0/kept/"),
    ] {
        put(&store, 1, id, path, "ok", "live", true);
    }
    put(
        &store,
        1,
        4,
        "handbook/_images/gone.png",
        "ok",
        "live",
        false,
    );
    put(
        &store,
        1,
        5,
        "handbook/_images/unlinked.png",
        "ok",
        "live",
        false,
    );
    put(
        &store,
        1,
        10,
        "handbook/_images/kept.png",
        "ok",
        "live",
        false,
    );
    fact(&store, 1, 1, "antora.component", "other");
    fact(&store, 1, 20, "antora.component", "named");
    fact(&store, 1, 20, "antora.version", "2.0");
    anchor(&store, 1, 2, "keep");
    anchor(&store, 1, 2, "lost");
    for (src, to, fragment, kind) in [
        (1, 2, Some("lost"), "anchor"),
        (1, 2, Some("keep"), "anchor"),
        (1, 3, None, "anchor"),
        (1, 4, None, "image"),
        (1, 5, None, "image"),
        (1, 10, None, "image"),
        (1, 6, None, "anchor"),
        (1, 7, None, "anchor"),
        (1, 8, None, "anchor"),
        (20, 3, None, "anchor"),
        (20, 7, None, "anchor"),
        (9, 3, None, "anchor"),
    ] {
        link(&store, 1, src, to, fragment, kind);
    }
    // The build: `handbook` has the page 2 and a `named/1.0` page, and the
    // image 4, which a built page names, answers `broken_url`.
    put(&store, 2, 2, "handbook/kept/", "ok", "local", true);
    put(&store, 2, 21, "named/1.0/kept/", "ok", "local", true);
    put(
        &store,
        2,
        4,
        "handbook/_images/gone.png",
        "broken_url",
        "local",
        false,
    );
    put(
        &store,
        2,
        10,
        "handbook/_images/kept.png",
        "ok",
        "local",
        false,
    );
    put(&store, 2, 6, "other/b/", "ok", "baseline", true);
    anchor(&store, 2, 2, "keep");
    fact(&store, 2, 2, "antora.component", "handbook");
    fact(&store, 2, 21, "antora.component", "named");
    fact(&store, 2, 21, "antora.version", "1.0");
    store
}

fn gate_of(store: &Store, edit: impl FnOnce(&mut Request)) -> Gate {
    let mut request = Request::new(Report::Introduced);
    request.run = Some(RunId(2));
    edit(&mut request);
    gate(store, &request).unwrap()
}

fn rows(table: &crate::report::Table) -> Vec<String> {
    let removed = column(table, "removed");
    let raw = column(table, "raw_href");
    removed
        .iter()
        .zip(raw)
        .map(|(r, h)| format!("{r} {h}"))
        .collect()
}

#[test]
fn a_page_outside_the_build_linking_to_what_the_change_removed_is_inverse_breakage() {
    let g = gate_of(&store(), |_| {});
    // Ordered by the linking page's group, then page: `named` before `other`.
    assert_eq!(
        rows(&g.inverse),
        [
            "page 20>3#",
            "page 20>7#",
            "anchor 1>2#lost",
            "page 1>3#",
            "file 1>4#",
            "file 1>5#",
            "page 1>7#",
        ]
    );
    assert_eq!(
        g.inverse.columns,
        [
            "page",
            "raw_href",
            "target",
            "fragment",
            "kind",
            "removed",
            "component",
            "source_repo",
            "source_path"
        ]
    );
    assert_eq!(column(&g.inverse, "component")[0], "named");
    assert_eq!(g.inverse.rows[2][6], Cell::Text("other".into()));
}

#[test]
fn what_the_build_keeps_or_does_not_hold_is_no_inverse_breakage() {
    let g = gate_of(&store(), |_| {});
    let raw = column(&g.inverse, "raw_href");
    // 2 at `#keep`, 6 (another component's), 10 (an image the build keeps), 8 (a
    // version the build lacks) and 9's link (a page in the build's own
    // component).
    for kept in ["1>2#keep", "1>6#", "1>10#", "1>8#", "9>3#"] {
        assert!(!raw.contains(&kept.to_owned()), "{kept} in {raw:?}");
    }
}

#[test]
fn the_inverse_check_fails_the_gate_unless_it_is_asked_to_warn() {
    let store = store();
    let g = gate_of(&store, |_| {});
    assert!(g.introduced.rows.is_empty());
    assert!(g.fails());
    let g = gate_of(&store, |r| r.inverse = InverseMode::Warn);
    assert!(!g.fails());
    assert!(
        g.text(false)
            .contains("inverse breakage (warning only): 7 links")
    );
}

#[test]
fn the_text_counts_the_inbound_pages_across_the_components_between_the_sections() {
    let g = gate_of(&store(), |_| {});
    assert_eq!(counts(&g.inverse), (2, 2));
    let text = g.text(false);
    let at = |s: &str| text.find(s).unwrap_or_else(|| panic!("{s} in {text}"));
    assert!(
        at("introduced breakage: none")
            < at("inverse breakage: 7 links, 2 inbound pages across 2 components")
    );
    assert!(at("inverse breakage: 7 links") < at("introduced redirected links: none"));
    assert!(g.json(false).contains("\"introduced\":[],\"inverse\":[{"));
}

#[test]
fn the_filters_apply_to_the_linking_page() {
    let store = store();
    let g = gate_of(&store, |r| r.repo = Some("r/20".into()));
    assert_eq!(rows(&g.inverse), ["page 20>3#", "page 20>7#"]);
    let g = gate_of(&store, |r| {
        r.facts = vec![("antora.component".into(), "other".into())];
    });
    assert_eq!(
        column(&g.inverse, "component"),
        ["other", "other", "other", "other", "other"]
    );
    let g = gate_of(&store, |r| r.kinds = vec![crate::extract::Kind::Image]);
    assert_eq!(rows(&g.inverse), ["file 1>4#", "file 1>5#"]);
}

#[test]
fn a_build_that_holds_nothing_removes_only_anchors() {
    let store = store();
    store
        .connection()
        .execute("DELETE FROM page_facts WHERE run_id = 2", [])
        .unwrap();
    let g = gate_of(&store, |_| {});
    assert_eq!(rows(&g.inverse), ["anchor 1>2#lost"]);
}

#[test]
fn a_page_built_in_the_run_is_the_gates_and_not_the_inverse_checks() {
    let store = store();
    put(&store, 2, 1, "other/a/", "ok", "local", true);
    let g = gate_of(&store, |_| {});
    assert_eq!(rows(&g.inverse), ["page 20>3#", "page 20>7#"]);
}

#[test]
fn an_alias_landing_outside_the_held_set_is_its_owners_and_one_landing_inside_is_not() {
    let store = store();
    // 30 and 31 are aliases in `handbook` the build lacks: 30 lands on page 6
    // of `other`, which owns it, and 31 on page 3 of `handbook`, which the
    // change deleted. Page 1 links to both.
    for (id, path, landing) in [(30, "handbook/moved/", 6), (31, "handbook/old/", 3)] {
        put(&store, 1, id, path, "ok", "live", false);
        store
            .connection()
            .execute(
                "UPDATE targets SET final_url = (SELECT url FROM targets WHERE run_id = 1 \
                 AND url_id = ?) WHERE run_id = 1 AND url_id = ?",
                params![landing, id],
            )
            .unwrap();
        link(&store, 1, 1, id, None, "anchor");
    }
    let g = gate_of(&store, |_| {});
    let raw = column(&g.inverse, "raw_href");
    assert!(!raw.contains(&"1>30#".to_owned()), "{raw:?}");
    assert!(raw.contains(&"1>31#".to_owned()), "{raw:?}");
    assert_eq!(raw.len(), 8);
}

#[test]
fn a_build_serving_its_latest_under_another_segment_holds_that_segment() {
    let store = store();
    // `named` at 3.0 is served at `latest`: the build has 41, and lacks the
    // page 40, the image 43 and the page 42, which links to 3.
    for (id, path, page) in [
        (40, "named/latest/gone/", true),
        (41, "named/latest/kept/", true),
        (42, "named/latest/dead/", true),
        (43, "named/latest/_images/gone.png", false),
    ] {
        put(&store, 1, id, path, "ok", "live", page);
    }
    put(&store, 2, 41, "named/latest/kept/", "ok", "local", true);
    fact(&store, 2, 41, "antora.component", "named");
    fact(&store, 2, 41, "antora.version", "3.0");
    link(&store, 1, 1, 40, None, "anchor");
    link(&store, 1, 1, 43, None, "image");
    link(&store, 1, 42, 3, None, "anchor");
    let g = gate_of(&store, |_| {});
    let raw = column(&g.inverse, "raw_href");
    assert!(raw.contains(&"1>40#".to_owned()), "{raw:?}");
    assert!(raw.contains(&"1>43#".to_owned()), "{raw:?}");
    assert!(
        !raw.contains(&"42>3#".to_owned()),
        "a deleted latest page links from inside the build: {raw:?}"
    );
    assert_eq!(raw.len(), 9);
}

#[test]
fn the_inverse_report_prints_the_rows_and_the_baseline_and_counts_in_notes() {
    let mut request = Request::new(Report::Inverse);
    request.run = Some(RunId(2));
    let out = run(&store(), &request).unwrap();
    assert_eq!(out.table.rows.len(), 7);
    assert_eq!(
        out.notes,
        [
            "against baseline run 1, crawled on 2026-10-19",
            "7 links, 2 inbound pages across 2 components"
        ]
    );
}

#[test]
fn a_prefix_runs_through_the_components_segment_and_a_named_versions() {
    let url = at("handbook/kept/");
    assert_eq!(prefix(&url, "handbook", None), Some(at("handbook/")));
    assert_eq!(
        prefix(&url, "handbook", Some("master")),
        Some(at("handbook/"))
    );
    assert_eq!(prefix(&url, "handbook", Some("")), Some(at("handbook/")));
    let named = at("handbook/1.0/m/p/");
    assert_eq!(
        prefix(&named, "handbook", Some("1.0")),
        Some(at("handbook/1.0/"))
    );
    assert_eq!(
        prefix(&named, "handbook", Some("2.0")),
        Some(at("handbook/1.0/")),
        "the segment the page sits at"
    );
    assert_eq!(
        prefix(&at("handbook/latest/p/"), "handbook", Some("3.0")),
        Some(at("handbook/latest/")),
        "a version served under another name"
    );
    assert_eq!(
        prefix(&at("handbook/x/handbook/1.0/p/"), "handbook", Some("1.0")),
        Some(at("handbook/x/handbook/1.0/")),
        "the version's own name first"
    );
    assert_eq!(prefix(&at("handbook/"), "handbook", Some("1.0")), None);
    assert_eq!(prefix(&url, "other", None), None);
    assert_eq!(prefix(&at("a%20b/p/"), "a b", None), Some(at("a%20b/")));
}
