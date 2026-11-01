use super::{pages, repos};
use crate::config::Severity;
use crate::report::digest::tests::{mapped, of, ranked, with_doubled_slashes};

/// The pages ranked, by title where they have one and by URL otherwise.
fn order(digest: &crate::report::Digest) -> Vec<String> {
    pages(digest)
        .iter()
        .map(|p| {
            p.row
                .text("title")
                .or(p.row.text("page"))
                .unwrap()
                .to_owned()
        })
        .collect()
}

#[test]
fn pages_rank_by_their_mapped_red_links_then_their_amber_ones() {
    let mut digest = of(&ranked());
    // Outcome order: page 1 (three red, one amber), 3 (two errors), 4, 2.
    assert_eq!(order(&digest), ["Page 1", "Page 3", "Page 4", "Page 2"]);
    // An error softened to attention ranks below every page with a red link,
    // which the outcome order the JSON keeps cannot show.
    mapped(&mut digest, &[("error", "attention")]);
    assert_eq!(order(&digest), ["Page 1", "Page 4", "Page 2", "Page 3"]);
    // A fragment raised to red ties page 4's two red links with page 3's two.
    mapped(&mut digest, &[("broken_anchor", "broken")]);
    assert_eq!(order(&digest), ["Page 1", "Page 3", "Page 4", "Page 2"]);
}

#[test]
fn a_kind_at_none_leaves_a_page_its_counts_and_the_ranking() {
    let mut digest = of(&ranked());
    mapped(&mut digest, &[("error", "none")]);
    // Page 3 holds errors alone, so it has no row; page 1 counts the rest.
    assert_eq!(order(&digest), ["Page 1", "Page 4", "Page 2"]);
    let found = pages(&digest);
    assert!(found[0].findings.iter().all(|f| !f.error));
    // The JSON stays the data: the table still ranks and holds page 3.
    assert_eq!(digest.pages.rows.len(), 4);
}

#[test]
fn a_reasons_level_moves_its_page_down_a_rank_and_not_only_its_dots() {
    let mut digest = of(&with_doubled_slashes());
    let before = order(&digest);
    let page_7 = before.iter().position(|t| t.ends_with("/seven")).unwrap();
    mapped(&mut digest, &[("doubled_slash", "attention")]);
    let after = order(&digest);
    assert_eq!(after.len(), before.len());
    let moved = after.iter().position(|t| t.ends_with("/seven")).unwrap();
    assert!(moved > page_7, "{before:?} {after:?}");
    // Its count is amber and the page is not red.
    let found = pages(&digest);
    let seven = found
        .iter()
        .find(|p| p.row.text("page") == Some("https://e.test/seven"))
        .unwrap();
    assert_eq!(seven.counts.of_level(Severity::Broken), 0);
    assert!(seven.counts.of_level(Severity::Attention) > 0);
}

#[test]
fn a_repo_counts_and_ranks_its_pages_findings_and_a_repo_of_none_is_clean() {
    let mut digest = of(&ranked());
    let names = |digest: &crate::report::Digest| -> Vec<(Option<String>, usize)> {
        let found = pages(digest);
        repos(digest, &found)
            .iter()
            .map(|r| (r.row.text("repo").map(str::to_owned), r.pages))
            .collect()
    };
    let default = names(&digest);
    assert_eq!(default[0], (Some("r/one".to_owned()), 1));
    // The repo with no source holds pages 3 and 4, then the clean repos.
    assert_eq!(default[1], (None, 2));
    mapped(
        &mut digest,
        &[
            ("error", "none"),
            ("broken_url", "none"),
            ("broken_anchor", "none"),
        ],
    );
    assert!(names(&digest).iter().all(|(_, pages)| *pages == 0));
}

#[test]
fn a_pages_findings_are_its_red_links_then_its_amber_ones_and_leave_out_none() {
    let mut digest = of(&ranked());
    let levels = |digest: &crate::report::Digest, title: &str| -> Vec<(Severity, bool)> {
        let found = pages(digest);
        let page = found
            .iter()
            .find(|p| p.row.text("title") == Some(title))
            .unwrap();
        page.findings.iter().map(|f| (f.level, f.error)).collect()
    };
    let default = levels(&digest, "Page 1");
    assert_eq!(default.len(), 4);
    assert!(default.windows(2).all(|w| w[0].0 >= w[1].0), "{default:?}");
    // Page 3's two errors, red, come after no broken link.
    assert_eq!(
        levels(&digest, "Page 3"),
        [(Severity::Broken, true), (Severity::Broken, true)]
    );
    mapped(&mut digest, &[("error", "none")]);
    assert!(
        pages(&digest)
            .iter()
            .all(|p| p.findings.iter().all(|f| !f.error))
    );
}
