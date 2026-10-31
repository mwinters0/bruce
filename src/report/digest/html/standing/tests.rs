use super::{Counts, Kind, pages, repos, tally};
use crate::config::{Entry, Severity, SeverityMap};
use crate::report::digest::tests::{of, ranked, with_doubled_slashes};

/// `digest` judged by the severity `rows` of (`key`, `level`).
fn mapped(digest: &mut crate::report::Digest, rows: &[(&str, &str)]) {
    digest.policy = rows
        .iter()
        .map(|(key, level)| Entry {
            host: None,
            key: format!("severity.{key}"),
            value: (*level).to_owned(),
        })
        .collect();
    digest.severity = SeverityMap::from_entries(&digest.policy);
}

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
fn a_summary_counts_red_then_amber_and_leaves_out_a_level_with_no_links() {
    let counts = |cells: &[(Kind, Severity, i64)]| {
        let mut counts = Counts::default();
        for (kind, level, n) in cells {
            counts.add(*kind, *level, *n);
        }
        counts
    };
    let red =
        "<span class=\"cn\"><span class=\"sev broken\" role=\"img\" aria-label=\"Broken\"></span>";
    let amber = "<span class=\"cn\"><span class=\"sev attention\" role=\"img\" aria-label=\"Attention\"></span>";
    assert_eq!(
        tally(&counts(&[
            (Kind::Broken, Severity::Broken, 181),
            (Kind::Fragment, Severity::Attention, 2)
        ])),
        format!("{red} 181 broken links</span>, {amber} 2 missing fragments</span>")
    );
    assert_eq!(
        tally(&counts(&[(Kind::Broken, Severity::Broken, 1)])),
        format!("{red} 1 broken link</span>")
    );
    assert_eq!(
        tally(&counts(&[(Kind::Fragment, Severity::Attention, 1)])),
        format!("{amber} 1 missing fragment</span>")
    );
    // The links broken through a link and those that could not be verified
    // are red, in that order after the broken ones, the first with the
    // guide's ⓘ.
    let info = super::super::guide::Entry::DoubledSlash.info();
    assert_eq!(
        tally(&counts(&[
            (Kind::Broken, Severity::Broken, 1),
            (Kind::Through, Severity::Broken, 579),
            (Kind::Unverifiable, Severity::Broken, 2)
        ])),
        format!(
            "{red} 1 broken link</span>, {red} 579 broken through it{info}</span>, {red} 2 \
             unverifiable</span>"
        )
    );
}

#[test]
fn a_link_at_none_is_in_no_count_and_a_kind_is_counted_at_its_level() {
    let mut counts = Counts::default();
    counts.add(Kind::Broken, Severity::None, 5);
    assert!(counts.is_empty());
    assert_eq!(tally(&counts), "");
    // The same kind at two levels is two counts, the red one first.
    counts.add(Kind::Broken, Severity::Attention, 2);
    counts.add(Kind::Broken, Severity::Broken, 1);
    let tallied = tally(&counts);
    assert!(
        tallied.find("1 broken link<").unwrap() < tallied.find("2 broken links<").unwrap(),
        "{tallied}"
    );
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
    assert!(!tally(&found[0].counts).contains("unverifiable"));
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
