use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

fn in_scope(r: &Root, s: &str) -> bool {
    r.in_scope(&r.canonical(&url(s)))
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
    assert_eq!(r.hash, HashScope::default());
    assert!(c.allowlist.is_empty());
    assert!(c.skip.is_empty());
    assert_eq!(c.rate.per_second, DEFAULT_PER_SECOND);
    assert_eq!(c.rate.concurrency, 4);
    assert_eq!(c.user_agent, concat!("bruce/", env!("CARGO_PKG_VERSION")));
    assert_eq!(c.robots_token(), "bruce");
    assert!(c.hosts.is_empty());
    assert!(c.is_root_host("docs.example.org"));
    assert!(!c.is_root_host("example.org"));
}

#[test]
fn an_empty_file_is_every_default_with_no_root() {
    let c = Config::parse("").unwrap();
    assert!(c.roots.is_empty());
    assert!(c.allowlist.is_empty());
    assert_eq!(c.rate.per_second, DEFAULT_PER_SECOND);
    assert_eq!(c.user_agent, DEFAULT_USER_AGENT);
}

#[test]
fn a_root_in_a_file_takes_the_defaults_of_one_given_on_its_own() {
    let c = Config::parse("[[root]]\nurl = \"https://docs.example.org/en-US/\"\n").unwrap();
    let r = &c.roots[0];
    assert_eq!(r.profile, ProfileChoice::Auto);
    assert!(r.sitemap.is_none());
}

#[test]
fn every_key_is_read() {
    let c = Config::parse(
        r#"
        allowlist = ["GitHub.com", "pagure.io"]
        skip = ["https://pagure.io/**/issues/new*"]
        user_agent = "Acme-Docs/2 (+https://acme.example/bot)"

        [rate]
        per_second = 5
        concurrency = 8

        [host."GitLab.com"]
        per_second = 0.5
        concurrency = 1

        [host."docs.example.org"]
        per_second = 0.25
        robots = false

        [[root]]
        url = "https://docs.example.org/en-US/"
        include = ["https://docs.example.org/en-US/**"]
        exclude = ["https://docs.example.org/en-US/fedora/f2?/**"]
        sitemap = "https://docs.example.org/sitemap.xml"
        profile = "antora"

        [[root]]
        url = "https://other.example.net/"
        profile = "auto"
        "#,
    )
    .unwrap();
    assert_eq!(c.allowlist, ["github.com", "pagure.io"]);
    assert_eq!(c.skip.globs(), ["https://pagure.io/**/issues/new*"]);
    assert_eq!(c.rate.per_second, 5.0);
    assert_eq!(c.rate.concurrency, 8);
    assert_eq!(c.user_agent, "Acme-Docs/2 (+https://acme.example/bot)");
    assert_eq!(c.robots_token(), "acme-docs");
    assert_eq!(
        c.hosts["gitlab.com"],
        HostSettings {
            per_second: Some(0.5),
            concurrency: Some(1),
            robots: true
        }
    );
    assert_eq!(
        c.hosts["docs.example.org"],
        HostSettings {
            per_second: Some(0.25),
            concurrency: None,
            robots: false
        }
    );
    assert!(!c.hosts.contains_key("pagure.io"));
    let r = &c.roots[0];
    assert_eq!(r.include.globs(), ["https://docs.example.org/en-US/**"]);
    assert_eq!(
        r.sitemap.as_ref().map(Url::as_str),
        Some("https://docs.example.org/sitemap.xml")
    );
    assert_eq!(r.profile, ProfileChoice::Named("antora".into()));
    assert_eq!(c.roots[1].profile, ProfileChoice::Auto);
}

#[test]
fn an_unknown_key_is_refused() {
    for text in [
        "allowlists = []",
        "[rate]\nper_secnd = 1",
        "[[root]]\nurl = \"https://a.example/\"\nprofil = \"auto\"",
        "[host.\"a.example\"]\nper_second = 1\nburst = 2",
        "[rate]\nhost.\"a.example\".per_second = 1",
        "[[root]]\nurl = \"https://a.example/\"\nrobots = false",
        "[[root]]\nurl = \"https://a.example/\"\ncrawl_delay = 1",
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
fn two_roots_differing_only_in_escaping_are_one_root() {
    let msg = invalid_msg(
        "[[root]]\nurl = \"https://a.example/~x/\"\n[[root]]\nurl = \"https://a.example/%7Ex/\"",
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
        ("allowlist = [\"https://a.example/\"]", "allowlist:"),
        ("skip = [\"a[\"]", "skip:"),
        ("[rate]\nper_second = 0", "rate.per_second:"),
        ("[rate]\nconcurrency = 0", "rate.concurrency:"),
        (
            "[host.\"a.example\"]\nconcurrency = 0",
            "host.\"a.example\".concurrency:",
        ),
        ("user_agent = \"\"", "user_agent:"),
        ("user_agent = \"/1.0\"", "user_agent:"),
        ("user_agent = \"bot\\u00e9/1\"", "user_agent:"),
        (
            "[host.\"a.example\"]\nper_second = -2",
            "host.\"a.example\".per_second:",
        ),
        ("[host.\"a.example/x\"]", "host:"),
        (
            "[host.\"A.example\"]\nper_second = 1\n[host.\"a.example\"]\nper_second = 2",
            "named twice",
        ),
    ];
    for (text, want) in cases {
        let msg = invalid_msg(text);
        assert!(msg.contains(want), "{text}: {msg}");
    }
}

#[test]
fn the_robots_token_is_the_first_product_name_lowercased() {
    for (agent, token) in [
        ("bruce/0.1.0", "bruce"),
        (
            "Fedora-Docs-Bot/3 (+https://example.org)",
            "fedora-docs-bot",
        ),
        ("LinkCheck", "linkcheck"),
        ("my_bot.v2 extra", "my_bot.v2"),
    ] {
        assert_eq!(robots_token(agent).as_deref(), Ok(token), "{agent}");
    }
}

#[test]
fn robots_can_be_turned_off_only_for_a_host_a_root_starts_on() {
    let off = "[host.\"a.example\"]\nrobots = false\n";
    let root = "[[root]]\nurl = \"https://A.example/x/\"\n";
    let c = Config::parse(&format!("{off}{root}")).unwrap();
    assert!(!c.hosts["a.example"].robots);

    let external = "[host.\"c.example\"]\nrobots = false\n";
    let msg = invalid_msg(&format!("{external}{root}"));
    assert!(msg.starts_with("host.\"c.example\".robots:"), "{msg}");

    // A file with no root leaves one to the command line, so the check is
    // the caller's to run once it has the root.
    let c = Config::parse(off).unwrap();
    assert!(c.check_robots_hosts().is_err());
    let c = Config {
        roots: vec![Root::new("https://a.example/").unwrap()],
        ..c
    };
    assert!(c.check_robots_hosts().is_ok());
}

#[test]
fn a_roots_url_may_not_be_skipped() {
    let root = "[[root]]\nurl = \"https://a.example/en-US/\"\n";
    let msg = invalid_msg(&format!("skip = [\"https://a.example/**\"]\n{root}"));
    assert!(
        msg.starts_with("root[0]: url:") && msg.contains("skip"),
        "{msg}"
    );
    // A file naming no root leaves the check to whoever adds one.
    let c = Config::parse("skip = [\"https://a.example/**\"]").unwrap();
    assert!(c.check_skipped_roots().is_ok());
    let c = Config {
        roots: vec![Root::new("https://a.example/").unwrap()],
        ..c
    };
    assert!(c.check_skipped_roots().is_err());
}

#[test]
fn a_roots_url_must_be_in_its_own_scope() {
    let msg = invalid_msg(
        "[[root]]\nurl = \"https://a.example/en-US/\"\ninclude = [\"https://a.example/de/**\"]",
    );
    assert!(
        msg.starts_with("root[0]: url:") && msg.contains("own scope"),
        "{msg}"
    );
    let msg = invalid_msg(
        "[[root]]\nurl = \"https://a.example/en-US/\"\nexclude = [\"https://a.example/en-US/\"]",
    );
    assert!(msg.contains("own scope"), "{msg}");
    // The page the crawl starts on may be named by an include of its own.
    Config::parse(
        "[[root]]\nurl = \"https://a.example/en-US/\"\ninclude = [\"https://a.example/*/\"]",
    )
    .unwrap();
}

#[test]
fn the_default_scope_is_the_roots_path_prefix() {
    let r = Root::new("https://docs.example.org/en-US/fedora/index.html?x=1").unwrap();
    assert_eq!(r.prefix(), "https://docs.example.org/en-US/fedora/");
    assert!(in_scope(
        &r,
        "https://docs.example.org/en-US/fedora/f40/a.html"
    ));
    assert!(!in_scope(&r, "https://docs.example.org/en-US/quick-docs/"));
    assert!(!in_scope(&r, "https://docs.example.org/de/fedora/"));

    let bare = Root::new("https://docs.example.org").unwrap();
    assert_eq!(bare.prefix(), "https://docs.example.org/");
    assert!(!in_scope(&bare, "https://docs.example.org.evil.example/"));
}

#[test]
fn include_replaces_the_prefix_and_exclude_wins() {
    let c = Config::parse(
        r#"
        [[root]]
        url = "https://docs.example.org/en-US/fedora/"
        include = ["https://docs.example.org/*/fedora/**"]
        exclude = ["https://docs.example.org/*/fedora/f2?/**"]
        "#,
    )
    .unwrap();
    let r = &c.roots[0];
    assert!(in_scope(r, "https://docs.example.org/de/fedora/f40/a.html"));
    assert!(!in_scope(
        r,
        "https://docs.example.org/de/fedora/f26/a.html"
    ));
    assert!(!in_scope(r, "https://docs.example.org/en-US/quick-docs/"));
    // `*` stops at `/`.
    assert!(!in_scope(r, "https://docs.example.org/a/b/fedora/x"));
}

#[test]
fn scope_is_matched_on_the_canonical_form() {
    let c = Config::parse(
        r#"
        [[root]]
        url = "https://a.example/en-US/"
        include = ["https://a.example/en-US/", "https://a.example/en-US/~*/**"]
        exclude = ["https://a.example/en-US/~*/private/**"]
        "#,
    )
    .unwrap();
    let r = &c.roots[0];
    // A glob written with a decoded character matches the escape it decodes.
    assert!(in_scope(r, "https://a.example/en-US/%7Euser/page"));
    assert!(!in_scope(r, "https://a.example/en-US/%7Euser/%70rivate/x"));
    // The default prefix is canonical too.
    let p = Root::new("https://A.example:443/%7Ex/y.html").unwrap();
    assert_eq!(p.prefix(), "https://a.example/~x/");
    assert!(in_scope(&p, "https://a.example/%7ex/z"));
    // An escape the form keeps is not the character it encodes.
    assert!(!in_scope(&p, "https://a.example/~x%2Fz"));
}

#[test]
fn strip_query_drops_the_query_from_the_canonical_form() {
    let c = Config::parse(
        r#"
        [[root]]
        url = "https://a.example/a/"
        strip_query = true

        [[root]]
        url = "https://b.example/b/"
        "#,
    )
    .unwrap();
    let (a, b) = (&c.roots[0], &c.roots[1]);
    assert!(a.strip_query && !b.strip_query);
    let u = url("https://a.example/a/p?x=1#f");
    assert_eq!(a.canonical(&u).as_str(), "https://a.example/a/p");
    assert_eq!(b.canonical(&u).as_str(), "https://a.example/a/p?x=1");
    assert_ne!(a.canonical(&u).id(), b.canonical(&u).id());
    // The key is a boolean.
    assert!(
        Config::parse("[[root]]\nurl = \"https://a.example/\"\nstrip_query = \"yes\"").is_err()
    );
}

fn hash_scope(keys: &str) -> Result<HashScope, Error> {
    let text = format!("[[root]]\nurl = \"https://a.example/\"\n{keys}");
    Config::parse(&text).map(|c| c.roots[0].hash.clone())
}

#[test]
fn the_hash_keys_are_lists_of_single_selectors() {
    let scope = hash_scope(
        r#"hash_root = ["main, [role=main]"]
        hash_exclude = ["footer", "a[title='x, y'] > p", "p:not(.a, .b)"]"#,
    )
    .unwrap();
    assert_eq!(scope.root, ["main", "[role=main]"]);
    assert_eq!(
        scope.exclude,
        ["footer", "a[title='x, y'] > p", "p:not(.a, .b)"]
    );
    let only = hash_scope("hash_exclude = [\"footer\"]").unwrap();
    assert!(only.root.is_empty());
}

#[test]
fn a_hash_selector_lol_html_cannot_stream_is_refused_with_its_reason() {
    for (keys, want) in [
        (
            "hash_root = [\"h1 + p\"]",
            "hash_root: \"h1 + p\": Unsupported combinator `+`",
        ),
        (
            "hash_exclude = [\"a ~ b\"]",
            "hash_exclude: \"a ~ b\": Unsupported combinator `~`",
        ),
        (
            "hash_exclude = [\"div:has(p)\"]",
            "hash_exclude: \"div:has(p)\"",
        ),
        (
            "hash_root = [\"main,\"]",
            "hash_root: \"main,\" holds an empty selector",
        ),
        ("hash_root = [\"\"]", "hash_root:"),
    ] {
        let msg = match hash_scope(keys) {
            Err(Error::Invalid(msg)) => msg,
            other => panic!("{keys}: expected Invalid, got {other:?}"),
        };
        assert!(msg.starts_with("root[0]: "), "{msg}");
        assert!(msg.contains(want), "{keys}: {msg}");
    }
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

#[test]
fn every_toml_example_in_the_manual_parses() {
    let manual = include_str!("../../docs/manual/config.md");
    let mut blocks = manual.split("```toml\n").skip(1).peekable();
    assert!(blocks.peek().is_some(), "the manual has no toml example");
    for block in blocks {
        let toml = block.split("```").next().unwrap();
        if let Err(e) = Config::parse(toml) {
            panic!("{e}\nin the manual's example:\n{toml}");
        }
    }
}

/// The deployment guide's `bruce.toml`, its tagged blocks joined as `mise run
/// sim` joins them (`scripts/sim.py`).
fn guide_config() -> Config {
    let guide = include_str!("../../docs/manual/deploy.md");
    let mut joined = String::new();
    for block in guide.split("```toml file=bruce.toml\n").skip(1) {
        joined.push_str(block.split("```").next().unwrap());
        joined.push('\n');
    }
    Config::parse(&joined).unwrap_or_else(|e| panic!("{e}\nin the guide's config:\n{joined}"))
}

#[test]
fn the_guides_skip_globs_match_every_shape_of_the_forges_issue_link() {
    let c = guide_config();
    let skipped = |s: &str| c.skip.is_match(canonical(&url(s), false).as_str());
    for s in [
        "https://forge.fedoraproject.org/fedora/docs/issues/new",
        "https://forge.fedoraproject.org/a/b/issues/new?title=x",
        "https://forge.fedoraproject.org/a/b/issues/new/choose",
        "https://forge.fedoraproject.org/a/b/issues/new?title=Doc%20issue%20in%20file%20modules/ROOT/pages/index.adoc",
        "https://forge.fedoraproject.org/a/b/issues/new?title=[main]%20Doc%20issue%20in%20file%20modules/ROOT/pages/index.adoc",
        "https://forge.fedoraproject.org/user/login",
        "https://forge.fedoraproject.org/user/login?redirect_to=%2Fa%2Fb",
    ] {
        assert!(skipped(s), "{s} is not skipped");
    }
    for s in [
        "https://forge.fedoraproject.org/a/b/issues/12",
        "https://forge.fedoraproject.org/a/b/src/branch/main/modules/ROOT/pages/index.adoc",
        "https://docs.fedoraproject.org/en-US/fedora/latest/",
    ] {
        assert!(!skipped(s), "{s} is skipped");
    }
}

#[test]
fn the_guide_turns_robots_off_for_the_root_host_alone() {
    let c = guide_config();
    assert!(!c.hosts["docs.fedoraproject.org"].robots);
    assert!(c.hosts["forge.fedoraproject.org"].robots);
}

#[test]
fn a_run_records_its_roots_in_canonical_form() {
    let c = Config::parse(
        r#"
        [[root]]
        url = "https://a.example/a%7Eb/?x=1"
        strip_query = true

        [[root]]
        url = "https://b.example/b/?x=1"
        "#,
    )
    .unwrap();
    assert_eq!(
        c.root_urls(),
        ["https://a.example/a~b/", "https://b.example/b/?x=1"]
    );
}

/// The keys `T` reads, as serde's derive names them: its `deserialize_struct`
/// is handed them, and nothing is deserialized.
fn keys<'de, T: Deserialize<'de>>() -> &'static [&'static str] {
    use serde::de::{Error as _, Visitor, value::Error};

    struct Fields(&'static [&'static str]);
    impl<'de> serde::Deserializer<'de> for &mut Fields {
        type Error = Error;
        fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
            Err(Error::custom("not a struct"))
        }
        fn deserialize_struct<V: Visitor<'de>>(
            self,
            _: &'static str,
            fields: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Error> {
            self.0 = fields;
            Err(Error::custom("recorded"))
        }
        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
            enum identifier ignored_any
        }
    }
    let mut fields = Fields(&[]);
    let _ = T::deserialize(&mut fields);
    fields.0
}

/// The rows of every table in `manual` headed `| Key | Default |`: each key
/// and its default.
fn manual_keys(manual: &str) -> BTreeMap<String, String> {
    let mut rows = BTreeMap::new();
    let mut in_table = false;
    for line in manual.lines() {
        if line.starts_with("| Key | Default |") {
            in_table = true;
        } else if !line.starts_with('|') {
            in_table = false;
        } else if in_table && !line.starts_with("|---") {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let key = cells[1].trim_matches('`').to_owned();
            assert!(
                rows.insert(key, cells[2].to_owned()).is_none(),
                "{line}: a second row"
            );
        }
    }
    rows
}

/// A key is documented with its default in the change that adds it
/// (`docs/design/roadmap.md`, "Standing rules").
#[test]
fn every_key_the_config_reads_is_in_the_manual_with_its_default() {
    let documented = manual_keys(include_str!("../../docs/manual/config.md"));
    // `root`, `rate` and `host` are tables, whose keys are rows of their own.
    let mut read: Vec<String> = keys::<RawConfig>()
        .iter()
        .filter(|k| !matches!(**k, "root" | "rate" | "host"))
        .map(|k| k.to_string())
        .collect();
    read.extend(keys::<RawRoot>().iter().map(|k| k.to_string()));
    read.extend(keys::<RawRate>().iter().map(|k| format!("rate.{k}")));
    read.extend(
        keys::<RawHost>()
            .iter()
            .map(|k| format!("host.\"<host>\".{k}")),
    );
    for key in &read {
        match documented.get(key) {
            Some(default) => assert!(!default.is_empty(), "{key}: the manual states no default"),
            None => panic!("{key}: no row in docs/manual/config.md"),
        }
    }
    for key in documented.keys() {
        assert!(
            read.contains(key),
            "{key}: the manual names a key the config does not read"
        );
    }
}
