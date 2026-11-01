use super::tally;
use crate::config::Severity;
use crate::report::digest::model::{Counts, Kind, pages};
use crate::report::digest::tests::{mapped, of, ranked};

#[test]
fn a_summary_counts_red_then_amber_and_leaves_out_a_level_with_no_links() {
    let counts = |cells: &[(Kind, Severity, i64)]| {
        let mut counts = Counts::default();
        for (kind, level, n) in cells {
            counts.add(*kind, *level, *n, None);
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
    counts.add(Kind::Broken, Severity::None, 5, None);
    assert!(counts.is_empty());
    assert_eq!(tally(&counts), "");
    // The same kind at two levels is two counts, the red one first.
    counts.add(Kind::Broken, Severity::Attention, 2, None);
    counts.add(Kind::Broken, Severity::Broken, 1, None);
    let tallied = tally(&counts);
    assert!(
        tallied.find("1 broken link<").unwrap() < tallied.find("2 broken links<").unwrap(),
        "{tallied}"
    );
}

#[test]
fn a_part_the_map_moved_names_the_reasons_that_moved_it_and_no_other() {
    let mut digest = of(&ranked());
    let tallied = |digest: &crate::report::Digest, title: &str| {
        let found = pages(digest);
        let page = found
            .iter()
            .find(|p| p.row.text("title") == Some(title))
            .unwrap();
        tally(&page.counts)
    };
    // At the outcome's own level no part names a reason.
    assert!(!tallied(&digest, "Page 3").contains("<code>"));
    // The errors of one reason softened: their part is amber and names it.
    mapped(&mut digest, &[("server_error", "attention")]);
    let moved = tallied(&digest, "Page 3");
    assert!(
        moved.contains("unverifiable (<code>server_error</code>)</span>"),
        "{moved}"
    );
    assert!(moved.contains("aria-label=\"Attention\""), "{moved}");
    // The outcome moved whole is its own level: no reason is named.
    mapped(&mut digest, &[("error", "attention")]);
    assert!(!tallied(&digest, "Page 3").contains("<code>"));
    // A reason set to the outcome's own level moves nothing.
    mapped(&mut digest, &[("server_error", "broken")]);
    assert!(!tallied(&digest, "Page 3").contains("<code>"));
}

#[test]
fn a_part_names_every_reason_that_moved_its_links_and_the_through_part_none() {
    let mut counts = Counts::default();
    counts.add(Kind::Unverifiable, Severity::Attention, 2, Some("tls"));
    counts.add(Kind::Unverifiable, Severity::Attention, 1, Some("timeout"));
    counts.add(Kind::Broken, Severity::Attention, 1, Some("doubled_slash"));
    counts.add(Kind::Through, Severity::Attention, 4, None);
    let tallied = tally(&counts);
    assert!(
        tallied.contains("3 unverifiable (<code>timeout</code>, <code>tls</code>)</span>"),
        "{tallied}"
    );
    assert!(
        tallied.contains("1 broken link (<code>doubled_slash</code>)</span>"),
        "{tallied}"
    );
    assert!(tallied.contains("4 broken through it"), "{tallied}");
    assert!(!tallied.contains("through it ("), "{tallied}");
}
