use super::*;

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
