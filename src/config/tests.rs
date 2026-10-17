use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

fn invalid_msg(text: &str) -> String {
    match Config::parse(text) {
        Err(Error::Invalid(msg)) => msg,
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn a_run_given_only_a_url_has_every_default() {
    let c = Config::for_url("https://docs.example.org/en-US/").unwrap();
    assert_eq!(c.roots.len(), 1);
    let r = &c.roots[0];
    assert_eq!(r.url.as_str(), "https://docs.example.org/en-US/");
    assert!(r.include.is_empty() && r.exclude.is_empty());
    assert_eq!(r.sitemap, None);
    assert_eq!(r.profile, ProfileChoice::Auto);
    assert!(r.robots);
    assert_eq!(r.crawl_delay, None);
    assert!(c.allowlist.is_empty());
    assert_eq!(c.rate.per_second, DEFAULT_PER_SECOND);
    assert!(c.rate.hosts.is_empty());
}

#[test]
fn an_empty_file_is_every_default_with_no_root() {
    let c = Config::parse("").unwrap();
    assert!(c.roots.is_empty());
    assert!(c.allowlist.is_empty());
    assert_eq!(c.rate.per_second, DEFAULT_PER_SECOND);
}

#[test]
fn a_root_in_a_file_takes_the_defaults_of_one_given_on_its_own() {
    let c = Config::parse("[[root]]\nurl = \"https://docs.example.org/en-US/\"\n").unwrap();
    let r = &c.roots[0];
    assert_eq!(r.profile, ProfileChoice::Auto);
    assert!(r.robots && r.sitemap.is_none() && r.crawl_delay.is_none());
}

#[test]
fn every_key_is_read() {
    let c = Config::parse(
        r#"
        allowlist = ["GitHub.com", "pagure.io"]

        [rate]
        per_second = 5
        host."GitLab.com".per_second = 0.5

        [[root]]
        url = "https://docs.example.org/en-US/"
        include = ["https://docs.example.org/en-US/**"]
        exclude = ["https://docs.example.org/en-US/fedora/f2?/**"]
        sitemap = "https://docs.example.org/sitemap.xml"
        profile = "antora"
        robots = false
        crawl_delay = 0.25

        [[root]]
        url = "https://other.example.net/"
        profile = "auto"
        "#,
    )
    .unwrap();
    assert_eq!(c.allowlist, ["github.com", "pagure.io"]);
    assert_eq!(c.rate.per_second, 5.0);
    assert_eq!(c.rate.per_second_for("gitlab.com"), 0.5);
    assert_eq!(c.rate.per_second_for("pagure.io"), 5.0);
    let r = &c.roots[0];
    assert_eq!(r.include.globs(), ["https://docs.example.org/en-US/**"]);
    assert_eq!(
        r.sitemap.as_ref().map(Url::as_str),
        Some("https://docs.example.org/sitemap.xml")
    );
    assert_eq!(r.profile, ProfileChoice::Named("antora".into()));
    assert!(!r.robots);
    assert_eq!(r.crawl_delay, Some(0.25));
    assert_eq!(c.roots[1].profile, ProfileChoice::Auto);
}

#[test]
fn an_unknown_key_is_refused() {
    for text in [
        "allowlists = []",
        "[rate]\nper_secnd = 1",
        "[[root]]\nurl = \"https://a.example/\"\nprofil = \"auto\"",
        "[rate.host.\"a.example\"]\nper_second = 1\nburst = 2",
    ] {
        assert!(
            matches!(Config::parse(text), Err(Error::Parse(_))),
            "{text}"
        );
    }
}

#[test]
fn a_root_needs_an_http_url_without_a_fragment() {
    assert!(matches!(
        Config::parse("[[root]]\nprofile = \"auto\""),
        Err(Error::Parse(_))
    ));
    assert!(invalid_msg("[[root]]\nurl = \"not a url\"").starts_with("root[0]: url:"));
    assert!(invalid_msg("[[root]]\nurl = \"ftp://a.example/\"").contains("not http or https"));
    assert!(invalid_msg("[[root]]\nurl = \"https://a.example/#top\"").contains("fragment"));
    assert!(Config::for_url("file:///srv/site/").is_err());
}

#[test]
fn a_root_named_twice_is_refused() {
    let msg = invalid_msg(
        "[[root]]\nurl = \"https://a.example/x/\"\n[[root]]\nurl = \"https://A.example/x/\"",
    );
    assert!(msg.starts_with("root[1]:") && msg.contains("already a root"));
}

#[test]
fn bad_values_name_their_key() {
    let cases = [
        (
            "[[root]]\nurl = \"https://a.example/\"\ninclude = [\"a[\"]",
            "root[0]: include:",
        ),
        (
            "[[root]]\nurl = \"https://a.example/\"\nsitemap = \"x\"",
            "root[0]: sitemap:",
        ),
        (
            "[[root]]\nurl = \"https://a.example/\"\nprofile = \"\"",
            "root[0]: profile:",
        ),
        (
            "[[root]]\nurl = \"https://a.example/\"\ncrawl_delay = -1",
            "root[0]: crawl_delay:",
        ),
        ("allowlist = [\"https://a.example/\"]", "allowlist:"),
        ("[rate]\nper_second = 0", "rate.per_second:"),
        (
            "[rate.host.\"a.example\"]\nper_second = -2",
            "rate.host.\"a.example\".per_second:",
        ),
        (
            "[rate.host.\"A.example\"]\nper_second = 1\n[rate.host.\"a.example\"]\nper_second = 2",
            "named twice",
        ),
    ];
    for (text, want) in cases {
        let msg = invalid_msg(text);
        assert!(msg.contains(want), "{text}: {msg}");
    }
}

#[test]
fn the_default_scope_is_the_roots_path_prefix() {
    let r = Root::new("https://docs.example.org/en-US/fedora/index.html?x=1").unwrap();
    assert_eq!(r.prefix(), "https://docs.example.org/en-US/fedora/");
    assert!(r.in_scope(&url("https://docs.example.org/en-US/fedora/f40/a.html")));
    assert!(!r.in_scope(&url("https://docs.example.org/en-US/quick-docs/")));
    assert!(!r.in_scope(&url("https://docs.example.org/de/fedora/")));

    let bare = Root::new("https://docs.example.org").unwrap();
    assert_eq!(bare.prefix(), "https://docs.example.org/");
    assert!(!bare.in_scope(&url("https://docs.example.org.evil.example/")));
}

#[test]
fn include_replaces_the_prefix_and_exclude_wins() {
    let c = Config::parse(
        r#"
        [[root]]
        url = "https://docs.example.org/en-US/"
        include = ["https://docs.example.org/*/fedora/**"]
        exclude = ["https://docs.example.org/*/fedora/f2?/**"]
        "#,
    )
    .unwrap();
    let r = &c.roots[0];
    assert!(r.in_scope(&url("https://docs.example.org/de/fedora/f40/a.html")));
    assert!(!r.in_scope(&url("https://docs.example.org/de/fedora/f26/a.html")));
    assert!(!r.in_scope(&url("https://docs.example.org/en-US/quick-docs/")));
    // `*` stops at `/`.
    assert!(!r.in_scope(&url("https://docs.example.org/a/b/fedora/x")));
}

#[test]
fn a_file_is_read_from_disk_and_a_missing_one_says_where() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.toml");
    std::fs::write(&path, "[[root]]\nurl = \"https://a.example/\"\n").unwrap();
    assert_eq!(Config::load(&path).unwrap().roots.len(), 1);

    let missing = dir.path().join("absent.toml");
    let err = Config::load(&missing).unwrap_err();
    assert!(matches!(err, Error::Read { .. }));
    assert!(err.to_string().starts_with(&missing.display().to_string()));
}
