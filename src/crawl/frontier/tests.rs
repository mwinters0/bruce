use super::*;
use crate::fetch::local::Site;

fn frontier() -> Frontier {
    let config = Config::parse(
        r#"
        allowlist = ["forge.example.org"]

        [[root]]
        url = "https://docs.example.org/en-US/a/"
        strip_query = true

        [[root]]
        url = "https://docs.example.org/en-US/b/"
        "#,
    )
    .unwrap();
    Frontier::new(config)
}

fn plan(f: &Frontier, url: &str, kind: Kind) -> (Plan, String) {
    let (plan, form) = f.plan(&Url::parse(url).unwrap(), kind);
    (plan, form.as_str().to_owned())
}

#[test]
fn a_url_in_a_roots_scope_is_that_roots_and_has_its_form() {
    let f = frontier();
    assert_eq!(
        plan(&f, "https://docs.example.org/en-US/a/x/?q=1", Kind::Image),
        (
            Plan::Page { root: 0 },
            "https://docs.example.org/en-US/a/x/".to_owned()
        ),
        "root a strips the query"
    );
    assert_eq!(
        plan(&f, "https://docs.example.org/en-US/b/x/?q=1", Kind::Anchor),
        (
            Plan::Page { root: 1 },
            "https://docs.example.org/en-US/b/x/?q=1".to_owned()
        )
    );
}

#[test]
fn an_allowlisted_host_is_one_hop_deep_by_the_kind_of_link() {
    let f = frontier();
    let url = "https://forge.example.org/p/?x=1";
    assert_eq!(
        plan(&f, url, Kind::Anchor),
        (Plan::Anchors, url.to_owned()),
        "no root governs it, so the query stays"
    );
    assert_eq!(plan(&f, url, Kind::Image).0, Plan::Status);
    assert_eq!(plan(&f, url, Kind::Canonical).0, Plan::Status);
}

#[test]
fn everything_else_is_unchecked_and_says_which_case() {
    let f = frontier();
    let reason = |url: &str| match plan(&f, url, Kind::Anchor).0 {
        Plan::Unchecked(reason) => reason,
        other => panic!("{url} is {other:?}"),
    };
    assert_eq!(reason("https://docs.example.org/fr/a/"), "out_of_scope");
    assert_eq!(reason("https://docs.example.org/en-US/c/"), "out_of_scope");
    assert_eq!(reason("https://other.example.net/"), "host_not_allowlisted");
    assert_eq!(reason("mailto:me@example.org"), "scheme");
    assert_eq!(reason("tel:+15551234"), "scheme");
    assert_eq!(reason("javascript:void(0)"), "scheme");
    assert_eq!(reason("data:text/plain,hi"), "scheme");
}

#[test]
fn a_skipped_url_is_skipped_before_scope_and_the_allowlist() {
    let config = Config::parse(
        r#"
        allowlist = ["forge.example.org"]
        skip = ["https://*/*/*/issues/new*", "https://docs.example.org/en-US/a/old/**"]

        [[root]]
        url = "https://docs.example.org/en-US/a/"
        "#,
    )
    .unwrap();
    let f = Frontier::new(config);
    let skipped = |url: &str, kind| plan(&f, url, kind).0 == Plan::Unchecked("skipped");
    assert!(skipped(
        "https://forge.example.org/g/r/issues/new?title=x",
        Kind::Anchor
    ));
    assert!(
        skipped("https://other.example.net/g/r/issues/new", Kind::Image),
        "any host, listed or not"
    );
    assert!(
        skipped("https://docs.example.org/en-US/a/old/x/", Kind::Anchor),
        "inside a root's scope on its own host"
    );
    assert!(!skipped(
        "https://docs.example.org/en-US/a/new/",
        Kind::Anchor
    ));
    assert!(!skipped("https://forge.example.org/g/r/src/", Kind::Anchor));
    assert_eq!(
        f.hop_plan(
            &Url::parse("https://forge.example.org/g/r/issues/new").unwrap(),
            Kind::Anchor
        )
        .0,
        Plan::Unchecked("skipped"),
        "a hop onto a skipped URL ends the chain (D50)"
    );
}

#[test]
fn a_hop_is_followed_wherever_it_goes() {
    let f = frontier();
    let hop = |url: &str| f.hop_plan(&Url::parse(url).unwrap(), Kind::Anchor).0;
    assert_eq!(hop("https://other.example.net/"), Plan::Status);
    assert_eq!(hop("https://docs.example.org/fr/a/"), Plan::Status);
    assert_eq!(hop("https://forge.example.org/"), Plan::Anchors);
    assert_eq!(
        hop("https://docs.example.org/en-US/a/"),
        Plan::Page { root: 0 }
    );
}

#[test]
fn a_url_id_is_claimed_once() {
    let f = frontier();
    assert!(f.claim(7));
    assert!(!f.claim(7));
    assert!(f.claim(8));
}

/// A frontier over a build of `https://docs.example.org/en-US/a/` holding
/// its index and `x/`, checked against a baseline that holds the targets
/// with the ids of the URLs given.
fn against(externals: Externals, held: &[&str]) -> Frontier {
    let prefix = Url::parse("https://docs.example.org/en-US/a/").unwrap();
    let built = ["x/", ""].map(|p| (prefix.join(p).unwrap(), std::path::PathBuf::new()));
    let mut f = frontier();
    let index = held
        .iter()
        .map(|u| f.plan(&Url::parse(u).unwrap(), Kind::Anchor).1.id());
    f.against(
        Arc::new(Site::new(&prefix, built)),
        Baseline {
            run: crate::store::RunId(1),
            index: crate::store::BaselineIndex::new(index),
            externals,
        },
    );
    f
}

#[test]
fn against_a_baseline_a_url_the_build_lacks_is_copied_deferred_or_asked() {
    let f = against(
        Externals::Baseline,
        &[
            "https://docs.example.org/en-US/b/known/",
            "https://forge.example.org/p/",
        ],
    );
    let class = |url: &str, kind| plan(&f, url, kind).0;
    assert_eq!(
        class("https://docs.example.org/en-US/a/x/", Kind::Anchor),
        Plan::Page { root: 0 },
        "a built page is read"
    );
    assert_eq!(
        class("https://docs.example.org/en-US/a/x", Kind::Anchor),
        Plan::Page { root: 0 },
        "so is the slashless form the build moves to it"
    );
    assert_eq!(
        class("https://docs.example.org/en-US/a/y/", Kind::Anchor),
        Plan::Unbuilt,
        "under the prefix and not built waits for the held set"
    );
    assert_eq!(
        class("https://docs.example.org/en-US/b/known/", Kind::Anchor),
        Plan::Baseline,
        "in scope, outside the prefix and in the baseline"
    );
    assert_eq!(
        class("https://docs.example.org/en-US/b/new/", Kind::Anchor),
        Plan::Anchors,
        "in scope, outside the prefix and not in the baseline: asked, never read for its links"
    );
    assert_eq!(
        class("https://docs.example.org/en-US/b/new/", Kind::Image),
        Plan::Status
    );
    assert_eq!(
        class("https://forge.example.org/p/", Kind::Anchor),
        Plan::Baseline
    );
    assert_eq!(
        class("https://forge.example.org/q/", Kind::Anchor),
        Plan::Anchors
    );
    assert_eq!(
        class("https://other.example.net/", Kind::Anchor),
        Plan::Unchecked("host_not_allowlisted")
    );
    let beyond = |url: &str| {
        let (_, form) = f.plan(&Url::parse(url).unwrap(), Kind::Anchor);
        f.beyond(&form, Kind::Anchor)
    };
    assert_eq!(
        beyond("https://docs.example.org/en-US/b/known/"),
        Plan::Baseline
    );
    assert_eq!(beyond("https://docs.example.org/en-US/a/y/"), Plan::Anchors);
    assert_eq!(
        f.hop_plan(
            &Url::parse("https://docs.example.org/en-US/a/y/").unwrap(),
            Kind::Anchor
        )
        .0,
        Plan::Status,
        "a hop is asked, not deferred"
    );
}

#[test]
fn externals_decides_an_allowlisted_url_and_no_other() {
    let held = ["https://forge.example.org/p/"];
    let external = |externals, url: &str| plan(&against(externals, &held), url, Kind::Anchor).0;
    let known = "https://forge.example.org/p/";
    assert_eq!(
        external(Externals::Skip, known),
        Plan::Unchecked(EXTERNALS_SKIPPED)
    );
    assert_eq!(external(Externals::Baseline, known), Plan::Baseline);
    assert_eq!(external(Externals::Live, known), Plan::Anchors);
    let new = "https://forge.example.org/new/";
    assert_eq!(external(Externals::Baseline, new), Plan::Anchors);
    assert_eq!(
        external(Externals::Skip, "https://docs.example.org/en-US/b/new/"),
        Plan::Anchors,
        "a URL of a root is not an external"
    );
}

#[test]
fn a_run_with_no_build_plans_as_before() {
    let f = frontier();
    assert_eq!(
        plan(&f, "https://docs.example.org/en-US/a/y/", Kind::Anchor).0,
        Plan::Page { root: 0 }
    );
    assert!(f.baseline().is_none());
    assert!(f.prefix().is_none());
}
