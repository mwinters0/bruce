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
         NULL, NULL)",
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
    verdicts_of(store, "link_verdicts")
}

/// The verdicts of a relation: the view, or a call of the macro.
fn verdicts_of(store: &Store, relation: &str) -> BTreeMap<String, Verdict> {
    let mut stmt = store
        .connection()
        .prepare(&format!(
            "SELECT raw_href, outcome, reason, redirected FROM {relation}"
        ))
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
                 NULL, NULL, NULL), (1, 21, NULL, NULL, NULL, NULL, 'whole', 'generic', \
                 NULL, NULL, NULL, NULL, NULL);
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
    // The view is the macro with each link judged by its own run's policy (D77).
    assert_eq!(got, verdicts_of(&store, "link_verdicts_with()"));
    // A policy with the reason switched off keeps the verdicts and loses the
    // reason, whichever run's policy is named.
    let off = crate::config::Policy {
        pedantic_trailing_slash: false,
        ..crate::config::Policy::default()
    };
    store.begin_run(Mode::Crawl, "t", None, &[], &off).unwrap();
    assert_eq!(
        got,
        verdicts_of(&store, "link_verdicts_with(policy_run := 1)")
    );
    let off = verdicts_of(&store, "link_verdicts_with(policy_run := 2)");
    for (label, outcome, reason) in [
        ("added", "ok", None),
        ("query", "ok", None),
        ("doubled", "broken_url", Some("doubled_slash")),
        ("no-anchor", "broken_anchor", None),
    ] {
        expect(&off, label, outcome, reason);
    }
    // A run with no row for the setting is at its default.
    store
        .connection()
        .execute("DELETE FROM policy WHERE run_id = 1", [])
        .unwrap();
    assert_eq!(got, verdicts(&store));
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
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

/// Starts run 1 with `policy`.
fn judged_by(policy: &crate::config::Policy) -> Store {
    let store = open();
    store
        .begin_run(Mode::Crawl, "t", None, &[], policy)
        .unwrap();
    store
}

/// A target that answered `status`, and was classed `outcome` and `reason` by
/// the crawl; it landed on `final_url` if it did.
fn answered(
    store: &Store,
    id: u64,
    url: &str,
    status: u16,
    outcome: &str,
    reason: Option<&str>,
    final_url: Option<&str>,
) {
    store
        .connection()
        .execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, status, outcome, reason, final_url) \
             VALUES ('live', 1, ?, ?, ?, ?, ?, ?)",
            params![id, url, status, outcome, reason, final_url],
        )
        .unwrap();
}

#[test]
fn a_status_the_policy_judges_is_read_over_the_outcome_the_crawl_stored() {
    use crate::config::Answer;
    let mut policy = crate::config::Policy::default();
    policy.status.insert(403, Answer::Ok);
    policy.status.insert(404, Answer::Error);
    policy.hosts.insert(
        "forge.test".to_string(),
        crate::config::HostJudging {
            doubled_slash: None,
            status: [(403, Answer::BrokenUrl), (404, Answer::BrokenUrl)].into(),
            ..Default::default()
        },
    );
    let store = judged_by(&policy);
    let hundreds = |id, url: &str, status, outcome, reason, landing: Option<&str>| {
        answered(&store, id, url, status, outcome, reason, landing)
    };
    hundreds(2, "https://a.test/x", 403, "error", None, None);
    hundreds(3, "https://forge.test/x", 403, "error", None, None);
    hundreds(4, "https://a.test/y", 404, "broken_url", None, None);
    hundreds(5, "https://a.test/z", 500, "error", None, None);
    hundreds(
        6,
        "https://short.test/r",
        403,
        "error",
        None,
        Some("https://forge.test/x"),
    );
    hundreds(
        7,
        "https://a.test/w",
        404,
        "broken_url",
        Some("wrong_trailing_slash"),
        None,
    );
    hundreds(
        8,
        "https://a.test/v",
        302,
        "broken_url",
        Some("bad_redirect"),
        None,
    );
    hundreds(9, "https://forge.test/y", 401, "error", None, None);
    hundreds(
        10,
        "https://forge.test/k",
        404,
        "broken_url",
        Some("wrong_trailing_slash"),
        None,
    );
    for (label, id, fragment) in [
        ("default-403", 2, None),
        ("forge-403", 3, None),
        ("404", 4, None),
        ("500", 5, None),
        ("forge-landing", 6, None),
        ("probed", 7, None),
        ("hop", 8, None),
        ("forge-401", 9, None),
        ("kept", 10, None),
        ("fragment", 2, Some("x")),
    ] {
        link(&store, label, id, fragment, None);
    }
    let got = verdicts(&store);
    for (label, outcome, reason) in [
        // The file's default for every host, and a host's own over it.
        ("default-403", "ok", None),
        ("forge-403", "broken_url", None),
        ("404", "error", None),
        // A code no map names is judged as the crawl classed it.
        ("500", "error", None),
        ("forge-401", "error", None),
        // The host that answered is the one judged, not the one linked.
        ("forge-landing", "broken_url", None),
        // A reason the crawl set goes with the class it saw, and stays where
        // the map leaves the outcome as stored.
        ("probed", "error", None),
        ("kept", "broken_url", Some("wrong_trailing_slash")),
        // A redirect is a hop, whatever a map says.
        ("hop", "broken_url", Some("bad_redirect")),
        // An `ok` target with no body read has no anchors to check.
        ("fragment", "unchecked", Some("no_anchors")),
    ] {
        expect(&got, label, outcome, reason);
    }
    // A comparison judges by another run's policy: run 2 has the defaults.
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
    let under = verdicts_of(&store, "link_verdicts_with(policy_run := 2)");
    for (label, outcome) in [
        ("default-403", "error"),
        ("forge-403", "error"),
        ("404", "broken_url"),
    ] {
        expect(&under, label, outcome, None);
    }
}

#[test]
fn a_scheme_is_unchecked_where_the_policy_lists_it_and_broken_where_it_does_not() {
    let mut policy = crate::config::Policy::default();
    policy.schemes.push("matrix".to_string());
    let store = judged_by(&policy);
    // As the crawl classed them under that policy.
    for (id, url, outcome, reason) in [
        (2, "matrix:r/fedora:matrix.org", "unchecked", "scheme"),
        (3, "mailto:me@e.test", "unchecked", "scheme"),
        (4, "ssh://git@e.test/r.git", "broken_url", "bad_scheme"),
    ] {
        raw_target(&store, id, url, outcome, None);
        store
            .connection()
            .execute(
                "UPDATE targets SET reason = ? WHERE url_id = ?",
                params![reason, id],
            )
            .unwrap();
    }
    for (label, id) in [("matrix", 2), ("mail", 3), ("ssh", 4)] {
        link(&store, label, id, None, None);
    }
    let got = verdicts(&store);
    expect(&got, "matrix", "unchecked", Some("scheme"));
    expect(&got, "mail", "unchecked", Some("scheme"));
    expect(&got, "ssh", "broken_url", Some("bad_scheme"));
    // Under the defaults `matrix` reads the other way.
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
    let under = verdicts_of(&store, "link_verdicts_with(policy_run := 2)");
    expect(&under, "matrix", "broken_url", Some("bad_scheme"));
    expect(&under, "mail", "unchecked", Some("scheme"));
    expect(&under, "ssh", "broken_url", Some("bad_scheme"));
}

#[test]
fn a_doubled_slash_is_a_fault_only_on_a_host_the_policy_judges_it_on() {
    let mut policy = crate::config::Policy::default();
    policy.hosts.insert(
        "off.test".to_string(),
        crate::config::HostJudging {
            doubled_slash: Some(false),
            status: Default::default(),
            ..Default::default()
        },
    );
    let store = judged_by(&policy);
    raw_target(&store, 2, "https://on.test/a//b", "ok", None);
    raw_target(&store, 3, "https://off.test/a//b", "ok", None);
    raw_target(
        &store,
        4,
        "https://off.test/c",
        "ok",
        Some("https://off.test/c//"),
    );
    raw_target(&store, 5, "https://off.test/d//e", "broken_url", None);
    for (label, id) in [("on", 2), ("off", 3), ("off-landed", 4), ("off-404", 5)] {
        link(&store, label, id, None, None);
    }
    let got = verdicts(&store);
    expect(&got, "on", "broken_url", Some("doubled_slash"));
    // The link then reads as what the target answered.
    expect(&got, "off", "ok", None);
    expect(&got, "off-landed", "ok", None);
    expect(&got, "off-404", "broken_url", None);

    // A page at a doubled URL answers for its relative links only where the
    // slash is a fault: on `off.test` they count on the page that holds them.
    raw_target(&store, 30, "https://off.test/p//q", "ok", None);
    raw_target(&store, 32, "https://on.test/p//q", "ok", None);
    raw_target(&store, 40, "https://e.test/gone", "broken_url", None);
    for src in [30, 32] {
        page(&store, 1, src, &[]);
        link_from(&store, src, "../gone/", 40);
    }
    assert!(counted(&store, 30)["../gone/"].0);
    assert!(!counted(&store, 32)["../gone/"].0);

    // The default for every host is the file's `[default]` key, over which a
    // host's own stands.
    let mut policy = crate::config::Policy {
        doubled_slash: false,
        ..crate::config::Policy::default()
    };
    policy.hosts.insert(
        "on.test".to_string(),
        crate::config::HostJudging {
            doubled_slash: Some(true),
            status: Default::default(),
            ..Default::default()
        },
    );
    store
        .begin_run(Mode::Crawl, "t", None, &[], &policy)
        .unwrap();
    let got = verdicts_of(&store, "link_verdicts_with(policy_run := 2)");
    expect(&got, "on", "broken_url", Some("doubled_slash"));
    expect(&got, "off", "ok", None);
}

/// A target that took `hops` hops to `final_url`, answering `outcome`.
fn chained(store: &Store, id: u64, url: &str, outcome: &str, hops: usize, final_url: &str) {
    let chain = vec!["{'url': 'https://e.test/hop', 'status': 301, 'refresh': false}"; hops];
    store
        .connection()
        .execute(
            &format!(
                "INSERT INTO targets (answered_by, run_id, url_id, url, outcome, final_url, \
                 redirect_chain) VALUES ('live', 1, ?, ?, ?, ?, [{}])",
                chain.join(", ")
            ),
            params![id, url, outcome, final_url],
        )
        .unwrap();
}

#[test]
fn a_chain_longer_than_the_hosts_limit_is_broken_whatever_its_fragment_and_ok_within_it() {
    let mut policy = crate::config::Policy {
        max_redirects: 2,
        ..crate::config::Policy::default()
    };
    for (host, limit) in [("forge.test", 4), ("strict.test", 0)] {
        policy.hosts.insert(
            host.to_string(),
            crate::config::HostJudging {
                max_redirects: Some(limit),
                ..Default::default()
            },
        );
    }
    let store = judged_by(&policy);
    chained(&store, 2, "https://e.test/a", "ok", 2, "https://e.test/z");
    chained(&store, 3, "https://e.test/b", "ok", 3, "https://e.test/z");
    chained(
        &store,
        4,
        "https://forge.test/c",
        "ok",
        4,
        "https://e.test/z",
    );
    chained(
        &store,
        5,
        "https://strict.test/d",
        "ok",
        1,
        "https://e.test/z",
    );
    chained(
        &store,
        6,
        "https://e.test/e",
        "broken_url",
        3,
        "https://e.test/z",
    );
    chained(&store, 7, "https://e.test/f/", "ok", 3, "https://e.test/f");
    chained(&store, 8, "https://e.test/g", "ok", 0, "https://e.test/z");
    for (label, id, fragment) in [
        ("within", 2, None),
        ("over", 3, None),
        ("over-fragment", 3, Some("nope")),
        ("forge", 4, None),
        ("strict", 5, None),
        ("target-broken", 6, None),
        ("slash-only", 7, None),
        ("none", 8, None),
    ] {
        link(&store, label, id, fragment, None);
    }
    let over = Some("too_many_redirects");
    let got = verdicts(&store);
    expect(&got, "within", "ok", None);
    expect(&got, "over", "broken_url", over);
    expect(&got, "over-fragment", "broken_url", over);
    // The limit is the linked URL's host's, over the default.
    expect(&got, "forge", "ok", None);
    expect(&got, "strict", "broken_url", over);
    // A target broken for its own reason keeps it, the worse fault.
    expect(&got, "target-broken", "broken_url", None);
    // The slash-only chain counts like any other, and is no longer pedantic.
    expect(&got, "slash-only", "broken_url", over);
    expect(&got, "none", "ok", None);
    // A link over the limit is still a redirected one, naming its landing.
    assert!(got["over"].2);

    // A policy with the default limit, another run's, judges the same rows ok.
    store
        .begin_run(
            Mode::Crawl,
            "t",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
    let got = verdicts_of(&store, "link_verdicts_with(policy_run := 2)");
    expect(&got, "over", "ok", None);
    expect(&got, "strict", "ok", None);
    expect(&got, "slash-only", "ok", Some("pedantic_trailing_slash"));
}

/// The query UF2 describes, run by the watch test below in a process of its own.
// upstream: UF2
#[test]
fn macro_parameter_in_a_join_condition_child() {
    if std::env::var_os("BRUCE_UF2_CHILD").is_none() {
        return;
    }
    let conn = duckdb::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE a (x INT); CREATE TABLE b (x INT, v VARCHAR);
         CREATE MACRO m(r := NULL) AS TABLE
             SELECT a.x, b.v FROM a LEFT JOIN b ON b.x = coalesce(r, a.x);
         CREATE VIEW vv AS SELECT * FROM m();
         SELECT * FROM vv;",
    )
    .unwrap();
}

/// A DuckDB built with assertions aborts on a macro parameter in a join
/// condition; this fails at the release that does not (UF2).
// upstream: UF2
#[test]
fn a_macro_parameter_in_a_join_condition_still_aborts_a_debug_duckdb() {
    if !cfg!(debug_assertions) {
        return;
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "store::views::tests::macro_parameter_in_a_join_condition_child",
        ])
        .env("BRUCE_UF2_CHILD", "1")
        .output()
        .unwrap();
    assert!(
        !status.status.success(),
        "the query now plans: remove the `pr` join (UF2)"
    );
}

#[test]
fn a_trial_policy_judges_every_run_and_writes_nothing_to_the_database() {
    use crate::config::Answer;
    let dir = std::env::temp_dir().join(format!("bruce-trial-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("t.duckdb");
    let registry = Registry::new(Box::new(Generic));
    let store = Store::open(&file, &registry).unwrap();
    for _ in 0..2 {
        store
            .begin_run(
                Mode::Crawl,
                "t",
                None,
                &[],
                &crate::config::Policy::default(),
            )
            .unwrap();
    }
    answered(&store, 2, "https://a.test/x", 404, "broken_url", None, None);
    link(&store, "404", 2, None, None);
    let stored = |store: &Store| -> i64 {
        store
            .connection()
            .query_row("SELECT count(*) FROM policy", [], |r| r.get(0))
            .unwrap()
    };
    let before = stored(&store);
    expect(&verdicts(&store), "404", "broken_url", None);

    let mut trial = crate::config::Policy::default();
    trial.status.insert(404, Answer::Ok);
    store.try_policy(&trial).unwrap();
    // The view, the macro and a query of the table itself read the trial's.
    expect(&verdicts(&store), "404", "ok", None);
    let under = verdicts_of(&store, "link_verdicts_with(policy_run := 2)");
    expect(&under, "404", "ok", None);
    let rows: i64 = store
        .connection()
        .query_row(
            "SELECT count(*) FROM policy_in_force WHERE key = 'status.404' AND value = 'ok'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 2, "a row for each run");
    assert_eq!(stored(&store), before);

    drop(store);
    // The file was never written, so a new connection finds the crawl's.
    let store = Store::open(&file, &registry).unwrap();
    expect(&verdicts(&store), "404", "broken_url", None);
    drop(store);
    std::fs::remove_dir_all(&dir).unwrap();
}
