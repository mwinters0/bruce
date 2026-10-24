use std::collections::BTreeMap;

use super::super::*;
use crate::profile::Registry;
use crate::profile::generic::Generic;

fn open() -> Store {
    Store::open_in_memory(&Registry::new(Box::new(Generic))).unwrap()
}

/// A target and, when it was redirected, where it landed.
fn target(
    store: &Store,
    run: u64,
    id: u64,
    outcome: &str,
    reason: Option<&str>,
    final_url: Option<u64>,
) {
    store
        .connection()
        .execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, outcome, reason, final_url) \
             VALUES ('live', ?, ?, ?, ?, ?, ?)",
            params![
                run,
                id,
                format!("https://e.test/{id}"),
                outcome,
                reason,
                final_url.map(|f| format!("https://e.test/{f}"))
            ],
        )
        .unwrap();
}

fn page(store: &Store, run: u64, id: u64, anchors: &[&str]) {
    let conn = store.connection();
    conn.execute(
        "INSERT INTO pages VALUES (?, ?, NULL, NULL, NULL, NULL, 'whole', 'generic', NULL, NULL, NULL, \
         NULL, false, true, NULL)",
        params![run, id],
    )
    .unwrap();
    for anchor in anchors {
        conn.execute(
            "INSERT INTO anchors VALUES (?, ?, ?, 'h2', 'content')",
            params![run, id, anchor],
        )
        .unwrap();
    }
}

/// A link from page 1 in run 1, labelled by its `raw_href`.
fn link(store: &Store, label: &str, target: u64, fragment: Option<&str>, xref: Option<&str>) {
    store
        .connection()
        .execute(
            "INSERT INTO links VALUES (1, 1, ?, ?, ?, 'anchor', ?)",
            params![target, fragment, label, xref],
        )
        .unwrap();
}

type Verdict = (String, Option<String>, bool);

fn verdicts(store: &Store) -> BTreeMap<String, Verdict> {
    let mut stmt = store
        .connection()
        .prepare("SELECT raw_href, outcome, reason, redirected FROM link_verdicts")
        .unwrap();
    stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?)))
    })
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap()
}

fn expect(got: &BTreeMap<String, Verdict>, label: &str, outcome: &str, reason: Option<&str>) {
    let (o, r, _) = got.get(label).unwrap_or_else(|| panic!("no link {label}"));
    assert_eq!(
        (o.as_str(), r.as_deref()),
        (outcome, reason),
        "link {label}"
    );
}

#[test]
fn a_fragment_is_checked_against_the_page_the_target_landed_on() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    // 2 is a page, 3 redirects to it, 4 is an ok asset, 5 redirects onto a
    // page robots.txt forbids.
    target(&store, 1, 2, "ok", None, None);
    page(
        &store,
        1,
        2,
        &["intro", "caf\u{e9}", "a b", "x%20y", "Case"],
    );
    target(&store, 1, 3, "ok", None, Some(2));
    target(&store, 1, 4, "ok", None, None);
    target(&store, 1, 5, "unchecked", Some("robots"), Some(6));
    target(&store, 1, 6, "unchecked", Some("robots"), None);
    // Another run's anchors never answer for this run's.
    target(&store, 2, 2, "ok", None, None);
    page(&store, 2, 2, &["other"]);

    for (label, fragment) in [
        ("none", None),
        ("empty", Some("")),
        ("top", Some("top")),
        ("TOP", Some("TOP")),
        ("known", Some("intro")),
        ("unknown", Some("nope")),
        ("case", Some("case")),
        ("decoded", Some("caf%C3%A9")),
        ("space", Some("a%20b")),
        ("raw", Some("x%20y")),
        ("not-utf8", Some("%FF")),
        ("other-run", Some("other")),
        ("text", Some(":~:text=hi")),
        ("text-after", Some("intro:~:text=hi")),
    ] {
        link(&store, label, 2, fragment, None);
    }
    link(&store, "via-known", 3, Some("intro"), None);
    link(&store, "via-unknown", 3, Some("nope"), None);
    link(&store, "asset-fragment", 4, Some("intro"), None);
    link(&store, "asset", 4, None, None);
    link(&store, "robots", 5, Some("intro"), None);

    let got = verdicts(&store);
    for (label, outcome, reason) in [
        ("none", "ok", None),
        ("empty", "ok", None),
        ("top", "ok", None),
        ("TOP", "ok", None),
        ("known", "ok", None),
        ("unknown", "broken_anchor", None),
        ("case", "broken_anchor", None),
        ("decoded", "ok", None),
        ("space", "ok", None),
        ("raw", "ok", None),
        ("not-utf8", "broken_anchor", None),
        ("other-run", "broken_anchor", None),
        ("text", "unchecked", Some("text_fragment")),
        ("text-after", "unchecked", Some("text_fragment")),
        ("via-known", "ok", None),
        ("via-unknown", "broken_anchor", None),
        ("asset-fragment", "unchecked", Some("no_anchors")),
        ("asset", "ok", None),
        ("robots", "unchecked", Some("robots")),
    ] {
        expect(&got, label, outcome, reason);
    }
    assert_eq!(got.len(), 19, "one verdict a link");
    let redirected: Vec<&str> = got
        .iter()
        .filter(|(_, v)| v.2)
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(redirected, ["robots", "via-known", "via-unknown"]);
}

#[test]
fn a_target_that_is_not_ok_gives_its_link_its_own_outcome_and_reason() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    target(&store, 1, 2, "broken_url", None, None);
    target(&store, 1, 3, "error", Some("timeout"), None);
    target(&store, 1, 4, "unchecked", Some("out_of_scope"), None);
    target(&store, 1, 5, "broken_url", Some("loop"), None);
    for (label, id) in [("404", 2), ("timeout", 3), ("scope", 4), ("loop", 5)] {
        link(&store, label, id, Some("intro"), None);
    }
    // Nothing was written for target 9: the run was stopped.
    link(&store, "cut", 9, None, None);

    let got = verdicts(&store);
    expect(&got, "404", "broken_url", None);
    expect(&got, "timeout", "error", Some("timeout"));
    expect(&got, "scope", "unchecked", Some("out_of_scope"));
    expect(&got, "loop", "broken_url", Some("loop"));
    expect(&got, "cut", "unchecked", Some("cut"));
}

#[test]
fn an_unresolved_xref_is_broken_whatever_the_fragment_names() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    target(&store, 1, 1, "ok", None, None);
    page(&store, 1, 1, &["comp::page.adoc"]);
    // Antora leaves the page itself as the target, the refspec its fragment.
    link(
        &store,
        "xref",
        1,
        Some("comp::page.adoc"),
        Some("comp::page.adoc"),
    );
    link(&store, "plain", 1, Some("comp::page.adoc"), None);

    let got = verdicts(&store);
    expect(&got, "xref", "broken_url", Some("unresolved_xref"));
    expect(&got, "plain", "ok", None);
}

#[test]
fn an_unresolved_xref_is_told_by_its_fragment_where_a_base_href_moved_its_target() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    target(&store, 1, 1, "ok", None, None);
    page(&store, 1, 1, &[]);
    // A `<base href>` made another page the target; the refspec is still the
    // whole fragment.
    target(&store, 1, 2, "ok", None, None);
    page(&store, 1, 2, &["comp::page.adoc#frag"]);
    link(
        &store,
        "xref",
        2,
        Some("comp::page.adoc#frag"),
        Some("comp::page.adoc#frag"),
    );

    let got = verdicts(&store);
    expect(&got, "xref", "broken_url", Some("unresolved_xref"));
}

#[test]
fn a_resolved_xref_is_judged_as_any_link_to_its_target() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    target(&store, 1, 1, "ok", None, None);
    page(&store, 1, 1, &[]);
    target(&store, 1, 2, "ok", None, None);
    page(&store, 1, 2, &["there"]);
    link(&store, "found", 2, Some("there"), Some("c::p.adoc#there"));
    link(&store, "no anchor", 2, Some("nope"), Some("c::p.adoc#nope"));

    let got = verdicts(&store);
    expect(&got, "found", "ok", None);
    expect(&got, "no anchor", "broken_anchor", None);
}

#[test]
fn a_resolved_xref_whose_url_is_broken_is_unresolved_and_one_not_known_to_be_is_not() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    target(&store, 1, 1, "ok", None, None);
    page(&store, 1, 1, &[]);
    // The URL a refspec names (D57) that production answers 404, and others
    // that are no verdict on the refspec.
    target(&store, 1, 2, "broken_url", Some("http_404"), None);
    target(&store, 1, 3, "error", Some("timeout"), None);
    target(&store, 1, 4, "unchecked", Some("robots"), None);
    link(&store, "gone", 2, None, Some("c::q.adoc"));
    link(&store, "error", 3, None, Some("c::r.adoc"));
    link(&store, "robots", 4, None, Some("c::s.adoc"));
    link(&store, "plain", 2, None, None);

    let got = verdicts(&store);
    expect(&got, "gone", "broken_url", Some("unresolved_xref"));
    expect(&got, "error", "error", Some("timeout"));
    expect(&got, "robots", "unchecked", Some("robots"));
    expect(&got, "plain", "broken_url", Some("http_404"));
}

#[test]
fn every_open_installs_the_view() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.duckdb");
    let registry = Registry::new(Box::new(Generic));
    drop(Store::open(&path, &registry).unwrap());
    let conn = duckdb::Connection::open(&path).unwrap();
    conn.execute("DROP VIEW link_verdicts", []).unwrap();
    drop(conn);
    let store = Store::open(&path, &registry).unwrap();
    assert!(verdicts(&store).is_empty());
}

/// A target whose URL is `url` and that landed on `final_url`.
fn slash_target(store: &Store, id: u64, url: &str, final_url: &str) {
    store
        .connection()
        .execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, outcome, final_url) \
             VALUES ('live', 1, ?, ?, 'ok', ?)",
            params![id, url, final_url],
        )
        .unwrap();
}

#[test]
fn a_redirect_that_only_adds_or_drops_a_final_slash_is_ok_and_pedantic() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    let base = "https://e.test/a";
    slash_target(&store, 2, base, "https://e.test/a/");
    slash_target(&store, 3, "https://e.test/b/", "https://e.test/b");
    slash_target(&store, 4, "https://e.test/c?x=1", "https://e.test/c/?x=1");
    slash_target(&store, 5, "https://e.test/d", "https://e.test/d//");
    slash_target(&store, 6, "https://e.test/e", "https://e.test/e/f");
    slash_target(&store, 7, "https://other.test/g", "https://e.test/g/");
    // 8 lands on a page that lacks the fragment, 9 on one that has it.
    slash_target(&store, 8, "https://e.test/h", "https://e.test/h/");
    slash_target(&store, 9, "https://e.test/i", "https://e.test/i/");
    store
        .connection()
        .execute_batch(
            "INSERT INTO targets (answered_by, run_id, url_id, url, outcome) \
                 VALUES ('live', 1, 20, 'https://e.test/h/', 'ok'), \
                        ('live', 1, 21, 'https://e.test/i/', 'ok');
             INSERT INTO pages VALUES (1, 20, NULL, NULL, NULL, NULL, 'whole', 'generic', NULL, NULL, \
                 NULL, NULL, false, true, NULL), (1, 21, NULL, NULL, NULL, NULL, 'whole', 'generic', \
                 NULL, NULL, NULL, NULL, false, true, NULL);
             INSERT INTO anchors VALUES (1, 21, 'here', 'h2', 'content');",
        )
        .unwrap();
    slash_target(&store, 10, "https://e.test/j", "https://e.test/j/");
    store
        .connection()
        .execute(
            "UPDATE targets SET outcome = 'broken_url' WHERE url_id = 10",
            [],
        )
        .unwrap();
    for (label, id, fragment) in [
        ("added", 2, None),
        ("dropped", 3, None),
        ("query", 4, None),
        ("doubled", 5, None),
        ("deeper", 6, None),
        ("host", 7, None),
        ("no-anchor", 8, Some("here")),
        ("anchor", 9, Some("here")),
        ("broken", 10, None),
    ] {
        link(&store, label, id, fragment, None);
    }
    let got = verdicts(&store);
    let pedantic = Some("pedantic_trailing_slash");
    for (label, outcome, reason) in [
        ("added", "ok", pedantic),
        ("dropped", "ok", pedantic),
        ("query", "ok", pedantic),
        ("doubled", "broken_url", Some("doubled_slash")),
        ("deeper", "ok", None),
        ("host", "ok", None),
        ("no-anchor", "broken_anchor", None),
        ("anchor", "ok", pedantic),
        ("broken", "broken_url", None),
    ] {
        expect(&got, label, outcome, reason);
    }
}

/// A target of any outcome whose URL is `url`, that landed on `final_url` if
/// it did.
fn raw_target(store: &Store, id: u64, url: &str, outcome: &str, final_url: Option<&str>) {
    store
        .connection()
        .execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, outcome, final_url) \
             VALUES ('live', 1, ?, ?, ?, ?)",
            params![id, url, outcome, final_url],
        )
        .unwrap();
}

#[test]
fn a_link_to_a_url_with_a_doubled_slash_is_broken_whatever_it_answered() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    raw_target(&store, 2, "https://e.test/a//b", "ok", None);
    raw_target(
        &store,
        3,
        "https://e.test/c",
        "ok",
        Some("https://e.test/c//"),
    );
    raw_target(&store, 4, "https://e.test/d//e", "error", None);
    raw_target(&store, 5, "https://e.test/f///", "broken_url", None);
    raw_target(&store, 6, "https://e.test//", "ok", None);
    // Never asked, off the allowlist or behind a `skip`: the URL is the fault.
    raw_target(&store, 7, "https://e.test/g//h", "unchecked", None);
    raw_target(&store, 12, "https://o.test/i//j", "unchecked", None);
    store
        .connection()
        .execute(
            "UPDATE targets SET reason = 'skipped' WHERE url_id = 7; \
             UPDATE targets SET reason = 'host_not_allowlisted' WHERE url_id = 12",
            [],
        )
        .unwrap();
    // Not a web page's URL: a `data:` body and a `mailto:` name no path.
    raw_target(
        &store,
        13,
        "data:text/plain;base64,YQ//Yg==",
        "unchecked",
        None,
    );
    raw_target(&store, 14, "mailto:a//b@e.test", "unchecked", None);
    // A `scheme://` in the path, a doubled slash in a query or a fragment,
    // and a single slash double nothing.
    raw_target(
        &store,
        8,
        "https://web.archive.org/web/2020/https://e.test/z",
        "ok",
        None,
    );
    raw_target(&store, 9, "https://e.test/q?next=a//b", "ok", None);
    raw_target(&store, 10, "https://e.test/p/", "ok", None);
    raw_target(
        &store,
        11,
        "https://e.test/r/",
        "ok",
        Some("https://e.test/r/s?u=http://x"),
    );
    for (label, id) in [
        ("named", 2),
        ("landed", 3),
        ("error", 4),
        ("broken", 5),
        ("root", 6),
        ("unchecked", 7),
        ("archive", 8),
        ("query", 9),
        ("single", 10),
        ("landed-query", 11),
        ("off", 12),
        ("data", 13),
        ("mail", 14),
    ] {
        link(&store, label, id, None, None);
    }
    let got = verdicts(&store);
    let doubled = Some("doubled_slash");
    for (label, outcome, reason) in [
        ("named", "broken_url", doubled),
        ("landed", "broken_url", doubled),
        ("error", "broken_url", doubled),
        ("broken", "broken_url", doubled),
        ("root", "broken_url", doubled),
        ("unchecked", "broken_url", doubled),
        ("off", "broken_url", doubled),
        ("data", "unchecked", None),
        ("mail", "unchecked", None),
        ("archive", "ok", None),
        ("query", "ok", None),
        ("single", "ok", None),
        ("landed-query", "ok", None),
    ] {
        expect(&got, label, outcome, reason);
    }
}

/// A link of page `src` in run 1.
fn link_from(store: &Store, src: u64, href: &str, target: u64) {
    store
        .connection()
        .execute(
            "INSERT INTO links VALUES (1, ?, ?, NULL, ?, 'anchor', NULL)",
            params![src, target, href],
        )
        .unwrap();
}

fn counted(store: &Store, src: u64) -> BTreeMap<String, (bool, bool)> {
    let mut stmt = store
        .connection()
        .prepare("SELECT raw_href, counted, relative FROM link_verdicts WHERE src_url_id = ?")
        .unwrap();
    stmt.query_map([src], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn a_doubled_slash_page_counts_nothing_with_a_twin_and_its_absolute_links_without_one() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    // 1 is a clean page. 30 doubles a slash and 31, its twin, is a page; 32
    // doubles one and its twin is not.
    for (id, url) in [
        (1, "https://e.test/one"),
        (30, "https://e.test/d//x"),
        (31, "https://e.test/d/x"),
        (32, "https://e.test/f//g"),
    ] {
        raw_target(&store, id, url, "ok", None);
        page(&store, 1, id, &[]);
    }
    raw_target(&store, 40, "https://e.test/gone", "broken_url", None);
    raw_target(&store, 41, "https://e.test/fine", "ok", None);
    for src in [1, 30, 32] {
        link_from(&store, src, "../gone/", 40);
        link_from(&store, src, "../fine/", 41);
        link_from(&store, src, "https://e.test/gone", 40);
        link_from(&store, src, "/gone", 40);
        link_from(&store, src, "#gone", 40);
        link_from(&store, src, "?next=1", 41);
    }
    let all = |src| counted(&store, src);
    // A relative href is neither absolute, rooted nor a fragment: a query
    // keeps the page's path.
    let relative: Vec<_> = all(1)
        .into_iter()
        .filter(|(_, (_, relative))| *relative)
        .map(|(href, _)| href)
        .collect();
    assert_eq!(relative, ["../fine/", "../gone/", "?next=1"]);
    assert!(all(1).values().all(|(counted, _)| *counted));
    assert!(
        all(30).values().all(|(counted, _)| !*counted),
        "the twin has them"
    );
    let no_twin = all(32);
    let uncounted: Vec<_> = no_twin
        .iter()
        .filter(|(_, (counted, _))| !*counted)
        .map(|(href, _)| href.as_str())
        .collect();
    assert_eq!(uncounted, ["../gone/"], "only its broken relative link");
}

#[test]
fn a_doubled_slash_page_linking_to_itself_is_judged_as_any_link() {
    let store = open();
    store.begin_run(Mode::Crawl, "t", None, &[]).unwrap();
    raw_target(&store, 32, "https://e.test/f//g", "ok", None);
    page(&store, 1, 32, &["here"]);
    link_from(&store, 32, "#here", 32);
    link_from(&store, 32, "#nope", 32);
    store
        .connection()
        .execute("UPDATE links SET fragment = substr(raw_href, 2)", [])
        .unwrap();
    let mut stmt = store
        .connection()
        .prepare("SELECT raw_href, outcome, reason, counted FROM link_verdicts ORDER BY 1")
        .unwrap();
    let got: Vec<(String, String, Option<String>, bool)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        got,
        [
            ("#here".to_owned(), "ok".to_owned(), None, true),
            ("#nope".to_owned(), "broken_anchor".to_owned(), None, true),
        ]
    );
}
