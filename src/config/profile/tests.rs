use super::*;
use crate::config::{Config, Entry, Error};

fn profiles(toml: &str) -> Vec<Definition> {
    Config::parse(toml).unwrap().policy.profiles
}

fn refused(toml: &str) -> String {
    match Config::parse(toml) {
        Err(Error::Invalid(msg)) => msg,
        other => panic!("expected Invalid, got {other:?}"),
    }
}

const SPHINX: &str = r#"
[[profile]]
name = "sphinx"
generator = "^Sphinx"
region = ["div.document", "main"]
edit_link = "a.edit"
locale_links = "a.language[href]"

[[profile.fact]]
key = "version"
selector = "meta[name=readthedocs-version-slug]"
attr = "content"

[[profile.fact]]
key = "heading"
selector = "h1"
pattern = "^(.*?)\\s*$"
"#;

#[test]
fn a_profile_is_read_with_its_rules_in_order() {
    let defined = profiles(SPHINX);
    assert_eq!(defined.len(), 1);
    let d = &defined[0];
    assert_eq!(d.name, "sphinx");
    assert_eq!(d.generator.as_deref(), Some("^Sphinx"));
    assert_eq!(d.region, ["div.document", "main"]);
    assert_eq!(d.edit_link.as_deref(), Some("a.edit"));
    assert_eq!(d.locale_links.as_deref(), Some("a.language[href]"));
    let keys: Vec<_> = d.facts.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["version", "heading"]);
    assert_eq!(d.facts[0].attr.as_deref(), Some("content"));
    assert_eq!(d.facts[1].attr, None);
}

#[test]
fn profiles_keep_the_order_the_file_wrote_them_in() {
    let names =
        |toml: &str| -> Vec<String> { profiles(toml).into_iter().map(|p| p.name).collect() };
    assert_eq!(
        names(
            "[[profile]]\nname = \"zeta\"\n[[profile]]\nname = \"alpha\"\n[[profile]]\nname = \"mid\"\n"
        ),
        ["zeta", "alpha", "mid"]
    );
}

#[test]
fn a_file_without_profiles_defines_none() {
    assert!(profiles("").is_empty());
}

#[test]
fn a_definition_is_recorded_as_rows_under_its_name() {
    let rows: Vec<(String, String)> = Config::parse(SPHINX)
        .unwrap()
        .policy
        .entries()
        .into_iter()
        .filter(Entry::is_profile)
        .map(|e| (e.key, e.value))
        .collect();
    let row = |k: &str, v: &str| (k.to_owned(), v.to_owned());
    assert_eq!(
        rows,
        [
            row("profile.sphinx.generator", "^Sphinx"),
            row("profile.sphinx.region", "div.document"),
            row("profile.sphinx.region", "main"),
            row("profile.sphinx.edit_link", "a.edit"),
            row("profile.sphinx.locale_links", "a.language[href]"),
            row(
                "profile.sphinx.fact.version.selector",
                "meta[name=readthedocs-version-slug]"
            ),
            row("profile.sphinx.fact.version.attr", "content"),
            row("profile.sphinx.fact.heading.selector", "h1"),
            row("profile.sphinx.fact.heading.pattern", "^(.*?)\\s*$"),
        ]
    );
}

const SOURCED: &str = r#"
[[profile]]
name = "sphinx"

[[profile.source]]
from = "url"
pattern = '^https://s\.org/en/(?P<ref>[^/]+)/(?P<page>.+)\.html$'
repo = "https://github.com/o/r"
path = "doc/${page}.rst"

[[profile.source]]
from = "edit_link"
pattern = '^(?P<repo>https://g\.org/o/r)/edit/(?P<ref>feature/x)/(?P<path>.+)$'
"#;

#[test]
fn source_rules_are_read_in_order_and_recorded_as_rows_by_place() {
    let d = &profiles(SOURCED)[0];
    assert_eq!(d.sources.len(), 2);
    assert_eq!(d.sources[0].from, "url");
    assert_eq!(d.sources[0].repo.as_deref(), Some("https://github.com/o/r"));
    assert_eq!(d.sources[0].git_ref, None);
    assert_eq!(d.sources[1].from, "edit_link");
    let rows: Vec<(String, String)> = d.rows();
    let keys: Vec<&str> = rows.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        keys,
        [
            "profile.sphinx.generator",
            "profile.sphinx.source.0.from",
            "profile.sphinx.source.0.pattern",
            "profile.sphinx.source.0.repo",
            "profile.sphinx.source.0.path",
            "profile.sphinx.source.1.from",
            "profile.sphinx.source.1.pattern",
        ]
    );
}

#[test]
fn what_makes_no_source_rule_is_refused_where_it_is() {
    let rule = |body: &str| format!("[[profile]]\nname = \"x\"\n[[profile.source]]\n{body}\n");
    for (body, expect) in [
        (
            "from = \"page\"\npattern = \"^x$\"",
            "profile.x: source[0]: from: \"page\" is not `url` or `edit_link`",
        ),
        (
            "from = \"url\"\npattern = \"(\"\nrepo = \"https://h.org/o/r\"\npath = \"a\"",
            "profile.x: source[0]: pattern:",
        ),
        (
            "from = \"url\"\npattern = \"^x$\"\npath = \"a\"",
            "source[0]: repo: the pattern has no capture named `repo`",
        ),
        (
            "from = \"url\"\npattern = \"^(?P<path>.)$\"\nrepo = \"github.com/o/r\"",
            "source[0]: repo: \"github.com/o/r\" is not an http(s) URL",
        ),
        (
            "from = \"url\"\npattern = \"^(?P<path>.)$\"\nrepo = \"https://h.org/o/r\"\nref = \"${x}\"",
            "source[0]: ref: \"${x}\" names `x`",
        ),
    ] {
        let msg = refused(&rule(body));
        assert!(msg.contains(expect), "{body}: {msg}");
    }
}

#[test]
fn a_profile_with_nothing_but_a_name_is_still_a_row() {
    let rows = Config::parse("[[profile]]\nname = \"bare\"\n")
        .unwrap()
        .policy
        .entries();
    let bare: Vec<_> = rows.iter().filter(|e| e.is_profile()).collect();
    assert_eq!(bare.len(), 1);
    assert_eq!(bare[0].key, "profile.bare.generator");
    assert_eq!(bare[0].value, "");
}

#[test]
fn a_profile_row_is_not_judged_and_is_written_as_its_recorded_key() {
    let policy = Config::parse(SPHINX).unwrap().policy;
    let rows = policy.entries();
    assert!(
        rows.iter()
            .filter(|e| e.is_profile())
            .all(|e| !e.is_judged())
    );
    assert!(rows.iter().any(Entry::is_judged));
    let written: Vec<String> = rows
        .iter()
        .filter(|e| e.is_profile())
        .map(Entry::written_as)
        .collect();
    assert!(written.contains(&"profile.sphinx.generator = \"^Sphinx\"".to_owned()));
    assert!(written.contains(&"profile.sphinx.region += \"main\"".to_owned()));
    assert!(written.contains(&"profile.sphinx.fact.version.attr = \"content\"".to_owned()));
}

#[test]
fn what_the_loader_refuses_it_says_where() {
    for (toml, expect) in [
        ("[[profile]]\nname = \"auto\"\n", "profile.auto: `auto`"),
        ("[[profile]]\nname = \"1st\"\n", "not a profile name"),
        ("[[profile]]\nname = \"a.b\"\n", "not a profile name"),
        (
            "[[profile]]\nname = \"x\"\n[[profile]]\nname = \"y\"\n[[profile]]\nname = \"x\"\n",
            "profile.x: the name of an earlier profile",
        ),
        (
            "[[profile]]\nname = \"x\"\ngenerator = \"(\"\n",
            "profile.x: generator:",
        ),
        (
            "[[profile]]\nname = \"x\"\nextends = \"x\"\n",
            "profile.x: extends: \"x\" is not the name of a compiled profile",
        ),
        (
            "[[profile]]\nname = \"x\"\nextends = \"a b\"\n",
            "profile.x: extends: \"a b\"",
        ),
        (
            "[[profile]]\nname = \"x\"\nregion = [\"div p\"]\n",
            "profile.x: region: \"div p\" holds a combinator",
        ),
        (
            "[[profile]]\nname = \"x\"\nregion = [\"div > p\"]\n",
            "holds a combinator",
        ),
        (
            "[[profile]]\nname = \"x\"\nregion = [\"p:has(a)\"]\n",
            "profile.x: region:",
        ),
        (
            "[[profile]]\nname = \"x\"\nedit_link = \"p:has(a)\"\n",
            "profile.x: edit_link:",
        ),
        (
            "[[profile]]\nname = \"x\"\nedit_link = \"\"\n",
            "profile.x: edit_link: empty",
        ),
        (
            "[[profile]]\nname = \"x\"\nlocale_links = \"p:has(a)\"\n",
            "profile.x: locale_links:",
        ),
        (
            "[[profile]]\nname = \"x\"\nlocale_links = \" \"\n",
            "profile.x: locale_links: empty",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a b\"\nselector = \"p\"\n",
            "profile.x: fact[0]: key: \"a b\" is not a fact key",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"p\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"q\"\n",
            "profile.x: fact[1]: key: \"a\" is the key of another fact",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"p:has(a)\"\n",
            "profile.x: fact[0]: selector:",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"p\"\nattr = \"a b\"\n",
            "fact[0]: attr: \"a b\" is not an attribute name",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"p\"\npattern = \"(\"\n",
            "fact[0]: pattern:",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"p\"\npattern = \"\\\\d+\"\n",
            "fact[0]: pattern: \"\\\\d+\" holds 0 capture groups",
        ),
        (
            "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"p\"\npattern = \"(a)(b)\"\n",
            "holds 2 capture groups",
        ),
    ] {
        let msg = refused(toml);
        assert!(msg.contains(expect), "{toml}: {msg}");
    }
}

#[test]
fn a_compound_selector_may_hold_a_combinator_inside_brackets_and_quotes() {
    let d = profiles(
        "[[profile]]\nname = \"x\"\nregion = [\"div[data-a='b c > d']\", \"a[title=\\\"x y\\\"]\"]\n",
    );
    assert_eq!(d[0].region.len(), 2);
}

#[test]
fn a_region_list_in_one_string_is_split_at_its_commas() {
    let d = profiles("[[profile]]\nname = \"x\"\nregion = [\"div.a, main\"]\n");
    assert_eq!(d[0].region, ["div.a", "main"]);
}

#[test]
fn a_key_the_table_does_not_know_is_refused() {
    let e = Config::parse("[[profile]]\nname = \"x\"\nrole = \"y\"\n")
        .unwrap_err()
        .to_string();
    assert!(e.contains("unknown field `role`"), "{e}");
}

#[test]
fn a_profile_without_a_name_is_refused() {
    let e = Config::parse("[[profile]]\ngenerator = \"^Sphinx\"\n")
        .unwrap_err()
        .to_string();
    assert!(e.contains("missing field `name`"), "{e}");
}

#[test]
fn the_table_form_of_a_profile_is_refused() {
    assert!(Config::parse("[profile.sphinx]\ngenerator = \"^Sphinx\"\n").is_err());
}

#[test]
fn each_profile_keeps_the_fact_rules_written_under_it() {
    let toml = "[[profile]]\nname = \"a\"\n[[profile.fact]]\nkey = \"k\"\nselector = \"p\"\n\
                [[profile]]\nname = \"b\"\n[[profile.fact]]\nkey = \"k\"\nselector = \"q\"\n\
                [[profile.fact]]\nkey = \"l\"\nselector = \"r\"\n";
    let d = profiles(toml);
    let shape: Vec<(&str, Vec<&str>)> = d
        .iter()
        .map(|p| {
            (
                p.name.as_str(),
                p.facts.iter().map(|f| f.selector.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(shape, [("a", vec!["p"]), ("b", vec!["q", "r"])]);
}

#[test]
fn an_extension_is_recorded_as_a_row_after_its_generator() {
    let defined =
        profiles("[[profile]]\nname = \"x\"\nextends = \"antora\"\nregion = [\"main\"]\n");
    assert_eq!(defined[0].extends.as_deref(), Some("antora"));
    let rows = defined[0].rows();
    let row = |k: &str, v: &str| (format!("profile.x.{k}"), v.to_owned());
    assert_eq!(
        rows,
        [
            row("generator", ""),
            row("extends", "antora"),
            row("region", "main")
        ]
    );
    assert!(
        profiles(SPHINX)[0]
            .rows()
            .iter()
            .all(|(k, _)| !k.ends_with(".extends"))
    );
}
