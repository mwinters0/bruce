//! The extractor over the saved pages (D24). Hand-written assertions for each
//! property a decision names, each citing it; beside them one summary
//! snapshot a fixture, which `cargo insta review` shows a change to.

mod common;

use std::collections::BTreeMap;

use bruce::cli::registry;
use bruce::config::ProfileChoice;
use bruce::extract::{Extracted, Placement, extract};
use common::{Fixture, fixtures};

fn run(f: &Fixture) -> Extracted {
    extract(&f.html, &f.url, &ProfileChoice::Auto, &registry())
        .unwrap_or_else(|e| panic!("{}: {e}", f.path))
}

fn count(x: &Extracted, placement: Placement) -> usize {
    x.anchors
        .iter()
        .filter(|a| a.placement == placement)
        .count()
}

/// What a snapshot holds: counts, never every anchor.
fn summary(x: &Extracted) -> String {
    let mut tags = BTreeMap::new();
    for a in &x.anchors {
        *tags.entry(a.tag.as_str()).or_insert(0) += 1;
    }
    let tags: Vec<_> = tags.iter().map(|(t, n)| format!("{t} {n}")).collect();
    let facts: Vec<_> = x
        .facts
        .pairs()
        .iter()
        .map(|(k, v)| format!("  {k} = {v}"))
        .collect();
    format!(
        "profile: {}\ntitle: {}\nanchors: {} (content {}, chrome {}, unknown {})\nby tag: {}\nfacts:{}\n",
        x.profile,
        x.title.as_deref().unwrap_or("-"),
        x.anchors.len(),
        count(x, Placement::Content),
        count(x, Placement::Chrome),
        count(x, Placement::Unknown),
        tags.join(", "),
        if facts.is_empty() {
            " none".to_string()
        } else {
            format!("\n{}", facts.join("\n"))
        },
    )
}

#[test]
fn every_fixture_is_summarised() {
    let pages = fixtures();
    assert_eq!(pages.len(), 40);
    for f in &pages {
        insta::with_settings!({ snapshot_path => "snapshots", prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(f.path.replace('/', "__"), summary(&run(f)));
        });
    }
}

/// Every docs.fp.o page but the 404 declares Antora (I12) and has an
/// `article.doc` (I8), so none has an `unknown` anchor (D34).
#[test]
fn docs_fp_o_pages_are_antora_with_a_region() {
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let x = run(f);
        if f.path.ends_with("404.html") {
            assert_eq!(x.profile, "generic", "{}: no generator meta", f.path);
            continue;
        }
        assert_eq!(x.profile, "antora", "{}", f.path);
        assert_eq!(count(&x, Placement::Unknown), 0, "{}", f.path);
        assert!(x.title.is_some(), "{}", f.path);
    }
}

/// I9: the template's chrome ids sit outside the article on every page, and
/// an empty `id=""` is no anchor (D12).
#[test]
fn chrome_ids_are_chrome_on_every_antora_page() {
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let x = run(f);
        if x.profile != "antora" {
            continue;
        }
        let chrome: Vec<&str> = x
            .anchors
            .iter()
            .filter(|a| a.placement == Placement::Chrome)
            .map(|a| a.value.as_str())
            .collect();
        assert!(chrome.contains(&"search-input"), "{}: {chrome:?}", f.path);
        assert!(chrome.len() >= 3, "{}: {chrome:?}", f.path);
        assert!(x.anchors.iter().all(|a| !a.value.is_empty()), "{}", f.path);
    }
}

/// I9: `@asciidoctor/tabs` ids are static, so tabs are content anchors.
#[test]
fn tab_ids_are_content_anchors() {
    let f = fixtures()
        .into_iter()
        .find(|f| f.path == "generators/antora/tabs.html")
        .unwrap();
    let x = run(&f);
    let tabs: Vec<_> = x
        .anchors
        .iter()
        .filter(|a| a.value.starts_with("_tabs_"))
        .collect();
    assert!(!tabs.is_empty());
    assert!(tabs.iter().all(|a| a.placement == Placement::Content));
}

/// D34: `generic` draws its region at `<main>` or `[role=main]`, and a page
/// with neither says `unknown` rather than guess.
#[test]
fn generic_regions_follow_the_landmarks() {
    let by_path = |p: &str| {
        let f = fixtures().into_iter().find(|f| f.path == p).unwrap();
        run(&f)
    };
    for landmark in ["generic/main-landmark", "generic/role-main"] {
        let x = by_path(&format!("generators/{landmark}.html"));
        assert_eq!(x.profile, "generic");
        assert_eq!(count(&x, Placement::Unknown), 0, "{landmark}");
        assert!(count(&x, Placement::Content) > 0, "{landmark}");
    }
    let none = by_path("generators/generic/no-landmarks.html");
    assert!(!none.anchors.is_empty());
    assert_eq!(count(&none, Placement::Unknown), none.anchors.len());
}
