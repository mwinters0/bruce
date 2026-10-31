use super::super::tests::{compared, of, store};
use super::super::{Against, digest};
use crate::report::Digest;
use crate::report::table::{Cell, Table};
use crate::report::tests::{column, link, target};
use crate::store::{RunId, Store};

fn compared_digest() -> Digest {
    digest_of(&compared())
}

fn digest_of(store: &Store) -> Digest {
    digest(
        store,
        Some(RunId(1)),
        Against::Run(RunId(4)),
        &crate::report::tests::registry(),
    )
    .unwrap()
}

fn sections(d: &Digest) -> [(&'static str, &Table); 9] {
    [
        ("pages", &d.pages),
        ("by_repo", &d.by_repo),
        ("broken", &d.broken),
        ("errors", &d.errors),
        ("broken_targets", &d.broken_targets),
        ("redirects", &d.redirects),
        ("orphans", &d.orphans),
        ("unlisted", &d.unlisted),
        ("sitemap_entries", &d.sitemap_entries),
    ]
}

#[test]
fn a_broken_link_is_new_where_section_3_lists_it() {
    let d = compared_digest();
    // Run 4's page 1 linked to the 404 (10) too; the other three are new.
    assert_eq!(
        column(&d.broken, "new"),
        ["Bool(false)", "Bool(true)", "Bool(true)", "Bool(true)"]
    );
    let new = d
        .broken
        .rows
        .iter()
        .filter(|r| r.last() == Some(&Cell::Bool(true)))
        .count();
    assert_eq!(new, d.new_breakage.rows.len());
    assert_eq!(column(&d.errors, "new"), ["Bool(true)"]);
}

#[test]
fn a_row_is_new_where_the_compared_runs_section_lacked_it() {
    let d = compared_digest();
    // Run 4's page 1 held a broken link, so it is no new page; page 2 held none.
    assert_eq!(column(&d.pages, "new"), ["Bool(false)", "Bool(true)"]);
    assert_eq!(
        column(&d.broken_targets, "new"),
        ["Bool(false)", "Bool(true)", "Bool(true)"]
    );
    for t in [&d.redirects, &d.orphans, &d.unlisted, &d.sitemap_entries] {
        assert!(column(t, "new").iter().all(|c| c == "Bool(true)"));
    }
}

#[test]
fn a_page_and_a_repo_count_their_new_links_and_a_clean_repo_is_never_new() {
    let d = compared_digest();
    // Page 1's two new broken links and its new error; page 2's one.
    assert_eq!(column(&d.pages, "new_links"), ["Int(3)", "Int(1)"]);
    assert_eq!(column(&d.by_repo, "repo"), ["r/one", "r/two", "Null"]);
    assert_eq!(
        column(&d.by_repo, "new"),
        ["Bool(false)", "Bool(true)", "Bool(false)"]
    );
    assert_eq!(
        column(&d.by_repo, "new_links"),
        ["Int(3)", "Int(1)", "Int(0)"]
    );
}

/// [`compared`] judged by run 1's policy `rows`, `(key, value)`.
fn compared_under(rows: &[(&str, &str)]) -> Digest {
    let store = compared();
    for (key, value) in rows {
        store
            .connection()
            .execute(
                "INSERT INTO policy VALUES (1, NULL, ?, ?)",
                duckdb::params![key, value],
            )
            .unwrap();
    }
    digest_of(&store)
}

#[test]
fn under_the_default_map_new_listed_is_new() {
    let d = compared_digest();
    for (name, t) in sections(&d) {
        if t.columns.iter().any(|c| c == "new_listed") {
            assert_eq!(column(t, "new_listed"), column(t, "new"), "{name}");
        } else {
            assert!(["broken", "orphans", "unlisted", "sitemap_entries"].contains(&name));
        }
    }
}

#[test]
fn a_page_and_a_repo_whose_findings_then_were_at_none_are_new_listed_and_not_new() {
    // Run 4's page 1 held only a broken URL, which run 1's map makes no
    // finding: the page is no new row of the data, and is new in the HTML.
    let d = compared_under(&[("severity.broken_url", "none")]);
    assert_eq!(column(&d.pages, "page").len(), 2);
    assert_eq!(column(&d.pages, "new"), ["Bool(false)", "Bool(true)"]);
    // Page 2's only link is to that broken URL: it is no finding, so no new one.
    assert_eq!(
        column(&d.pages, "new_listed"),
        ["Bool(true)", "Bool(false)"]
    );
    assert_eq!(column(&d.by_repo, "repo"), ["r/one", "r/two", "Null"]);
    assert_eq!(
        column(&d.by_repo, "new"),
        ["Bool(false)", "Bool(true)", "Bool(false)"]
    );
    assert_eq!(
        column(&d.by_repo, "new_listed"),
        ["Bool(true)", "Bool(false)", "Bool(false)"]
    );
}

#[test]
fn an_error_is_new_listed_where_the_compared_runs_was_at_none() {
    let with_timeout = || {
        let store = compared();
        // Run 4's page 1 linked to the allowlisted host too, which timed out.
        target(&store, 4, 12, 0, "error", Some("timeout"));
        store
            .connection()
            .execute(
                "UPDATE targets SET url = 'https://slow.test/12' WHERE run_id = 4 AND url_id = 12",
                [],
            )
            .unwrap();
        link(&store, 4, 1, 12, None, "anchor");
        store
    };
    // Run 1 got a 503 from it: an error in both, so no new one.
    let d = digest_of(&with_timeout());
    assert_eq!(column(&d.errors, "new"), ["Bool(false)"]);
    assert_eq!(column(&d.errors, "new_listed"), ["Bool(false)"]);
    // With timeouts no finding, the error then was none and is new in the HTML.
    let store = with_timeout();
    store
        .connection()
        .execute(
            "INSERT INTO policy VALUES (1, NULL, 'severity.timeout', 'none')",
            [],
        )
        .unwrap();
    let d = digest_of(&store);
    assert_eq!(column(&d.errors, "new"), ["Bool(false)"]);
    assert_eq!(column(&d.errors, "new_listed"), ["Bool(true)"]);
}

#[test]
fn with_nothing_to_compare_with_nothing_is_known_to_be_new() {
    let d = of(&store());
    for (name, t) in sections(&d) {
        assert!(!t.rows.is_empty(), "{name} has rows to mark");
        assert!(column(t, "new").iter().all(|c| c == "Null"), "{name}");
    }
    for t in [&d.pages, &d.by_repo] {
        assert!(column(t, "new_links").iter().all(|c| c == "Null"));
    }
    for t in [
        &d.pages,
        &d.by_repo,
        &d.errors,
        &d.broken_targets,
        &d.redirects,
    ] {
        assert!(column(t, "new_listed").iter().all(|c| c == "Null"));
    }
}
