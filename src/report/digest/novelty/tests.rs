use super::super::tests::{compared, of, store};
use super::super::{Against, digest};
use crate::report::Digest;
use crate::report::table::{Cell, Table};
use crate::report::tests::column;
use crate::store::RunId;

fn compared_digest() -> Digest {
    digest(&compared(), Some(RunId(1)), Against::Run(RunId(4))).unwrap()
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
}
