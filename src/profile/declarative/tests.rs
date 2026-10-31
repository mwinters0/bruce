use std::path::Path;

use super::*;
use crate::config::{Config, HashScope, ProfileChoice};
use crate::extract::{Extracted, Placement, extract};
use crate::profile::Registry;
use crate::profile::generic::Generic;

fn registry(toml: &str) -> Registry {
    let mut registry = Registry::new(Box::new(Generic));
    for definition in Config::parse(toml).unwrap().policy.profiles {
        registry = registry.with(Box::new(Declarative::new(&definition).unwrap()));
    }
    registry
}

fn read(toml: &str, choice: &ProfileChoice, url: &str, html: &str) -> Extracted {
    let url = Url::parse(url).unwrap();
    extract(html, &url, choice, &HashScope::default(), &registry(toml)).unwrap()
}

fn named(toml: &str, html: &str) -> Extracted {
    read(
        toml,
        &ProfileChoice::Named("x".into()),
        "https://example.org/a/b.html",
        html,
    )
}

fn facts(x: &Extracted) -> Vec<(&str, &str)> {
    x.facts
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect()
}

const SPHINX: &str = r#"
[profile.x]
region = ["div.document"]

[[profile.x.fact]]
key = "version"
selector = "meta[name=readthedocs-version-slug]"
attr = "content"

[[profile.x.fact]]
key = "content_root"
selector = "html"
attr = "data-content_root"

[[profile.x.fact]]
key = "heading"
selector = "h1"
pattern = "^(.*?)\\s*¶?$"
"#;

fn sphinx_page() -> (Url, String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/sphinx-doc-org/en/master/usage/quickstart.html");
    (
        Url::parse("https://www.sphinx-doc.org/en/master/usage/quickstart.html").unwrap(),
        std::fs::read_to_string(path).unwrap(),
    )
}

#[test]
fn a_sphinx_page_gives_its_facts_to_a_profile_the_root_names() {
    let (url, html) = sphinx_page();
    let x = extract(
        &html,
        &url,
        &ProfileChoice::Named("x".into()),
        &HashScope::default(),
        &registry(SPHINX),
    )
    .unwrap();
    assert_eq!(x.profile, "x");
    assert_eq!(
        facts(&x),
        [
            ("x.version", "master"),
            ("x.content_root", "../"),
            ("x.heading", "Getting started"),
        ]
    );
}

#[test]
fn a_regions_selector_replaces_generics() {
    let (url, html) = sphinx_page();
    let content = |toml: &str, choice: &ProfileChoice| {
        let x = extract(&html, &url, choice, &HashScope::default(), &registry(toml)).unwrap();
        x.anchors
            .iter()
            .filter(|a| a.placement == Placement::Content)
            .count()
    };
    let generic = content(SPHINX, &ProfileChoice::Named("generic".into()));
    let declared = content(SPHINX, &ProfileChoice::Named("x".into()));
    assert!(
        declared > generic,
        "div.document holds the sidebar's ids, which `[role=main]` does not: {declared} against {generic}"
    );
    let inherited = content("[profile.x]\n", &ProfileChoice::Named("x".into()));
    assert_eq!(inherited, generic, "no region takes generic's");
}

#[test]
fn a_page_with_none_of_the_markup_gives_no_fact() {
    let x = named(SPHINX, "<html><body><p>nothing</p></body></html>");
    assert!(facts(&x).is_empty());
}

#[test]
fn auto_picks_the_profile_whose_generator_pattern_matches() {
    let toml = "[profile.x]\ngenerator = \"^Sphinx \\\\d\"\n";
    let page = |g: &str| {
        let html =
            format!("<html><head><meta name=generator content=\"{g}\"></head><body></body></html>");
        read(toml, &ProfileChoice::Auto, "https://example.org/", &html).profile
    };
    assert_eq!(page("Sphinx 7.2"), "x");
    assert_eq!(page("Antora 3.1"), "generic");
    assert_eq!(
        read(
            "[profile.x]\n",
            &ProfileChoice::Auto,
            "https://example.org/",
            "<meta name=generator content=\"Sphinx 7\">"
        )
        .profile,
        "generic",
        "a profile with no generator is chosen only by a root"
    );
}

#[test]
fn the_profiles_in_file_order_are_tried_in_turn_under_auto() {
    let toml = "[profile.b]\ngenerator = \"Doc\"\n[profile.a]\ngenerator = \"Doc\"\n";
    let x = read(
        toml,
        &ProfileChoice::Auto,
        "https://example.org/",
        "<meta name=generator content=\"Doc 1\">",
    );
    assert_eq!(x.profile, "b");
}

#[test]
fn an_attribute_is_kept_as_written_and_an_element_without_it_is_passed_over() {
    let toml = "[[profile.x.fact]]\nkey = \"k\"\nselector = \"i\"\nattr = \"data-k\"\n";
    let x = named(
        toml,
        "<i>no</i><i data-k=\" a b \">1</i><i data-k=\"later\">2</i>",
    );
    assert_eq!(facts(&x), [("x.k", " a b ")]);
    let x = named(toml, "<i data-k=\"\">1</i><i data-k=\"later\">2</i>");
    assert_eq!(facts(&x), [("x.k", "")], "an empty attribute is a value");
}

#[test]
fn a_pattern_that_does_not_match_leaves_the_next_element_to_give_the_fact() {
    let toml = "[[profile.x.fact]]\nkey = \"k\"\nselector = \"i\"\nattr = \"data-k\"\npattern = \"^v(\\\\d+)$\"\n";
    let x = named(
        toml,
        "<i data-k=\"none\"></i><i data-k=\"v12\"></i><i data-k=\"v3\"></i>",
    );
    assert_eq!(facts(&x), [("x.k", "12")]);
}

#[test]
fn an_elements_text_is_trimmed_decoded_and_taken_through_its_descendants() {
    let toml = "[[profile.x.fact]]\nkey = \"t\"\nselector = \"p.t\"\n";
    let x = named(
        toml,
        "<p class=t>\n  Fish &amp; <b>chips</b> \n</p><p class=t>second</p>",
    );
    assert_eq!(facts(&x), [("x.t", "Fish & chips")]);
}

#[test]
fn an_element_with_no_text_gives_the_next_one_the_fact() {
    let toml = "[[profile.x.fact]]\nkey = \"t\"\nselector = \"p.t\"\n";
    let x = named(
        toml,
        "<p class=t> </p><p class=t>second</p><p class=t>third</p>",
    );
    assert_eq!(facts(&x), [("x.t", "second")]);
}

#[test]
fn a_nested_match_does_not_start_a_second_reading() {
    let toml = "[[profile.x.fact]]\nkey = \"t\"\nselector = \"div\"\n";
    let x = named(toml, "<div>outer <div>inner</div> tail</div>");
    assert_eq!(facts(&x), [("x.t", "outer inner tail")]);
}

#[test]
fn an_element_the_parser_closes_at_the_pages_end_gives_the_text_it_held() {
    let toml = "[[profile.x.fact]]\nkey = \"t\"\nselector = \"p\"\n";
    let x = named(toml, "<p>unclosed");
    assert_eq!(facts(&x), [("x.t", "unclosed")]);
}

#[test]
fn a_void_element_has_no_text() {
    let toml = "[[profile.x.fact]]\nkey = \"t\"\nselector = \"br\"\n";
    assert!(facts(&named(toml, "<br>text")).is_empty());
}

#[test]
fn two_rules_read_the_page_independently() {
    let toml = "[[profile.x.fact]]\nkey = \"a\"\nselector = \"h1\"\n\
                [[profile.x.fact]]\nkey = \"b\"\nselector = \"h1\"\npattern = \"^(.)\"\n";
    let x = named(toml, "<h1>Title</h1>");
    assert_eq!(facts(&x), [("x.a", "Title"), ("x.b", "T")]);
}

#[test]
fn an_edit_link_selector_names_the_link_no_tier_earns() {
    let html = "<main><div class=meta><a href=\"https://codeberg.org/o/r/src/branch/main/docs/a.rst\">source</a></div></main>";
    let plain = named("[profile.x]\n", html);
    assert!(plain.edit_link.is_none(), "no tier earns the link");
    let toml = "[profile.x]\nedit_link = \"div.meta a\"\n";
    let x = named(toml, html);
    let edit = x.edit_link.expect("the profile named it");
    assert_eq!(
        edit.url.as_str(),
        "https://codeberg.org/o/r/src/branch/main/docs/a.rst"
    );
    assert_eq!(edit.source.unwrap().repo, "https://codeberg.org/o/r");
}

#[test]
fn the_first_edit_link_the_selector_matches_is_the_one() {
    let toml = "[profile.x]\nedit_link = \"a.e\"\n";
    let html = "<a class=e>no href</a><a class=e href=\"https://example.org/one\">1</a><a class=e href=\"https://example.org/two\">2</a>";
    let x = named(toml, html);
    assert_eq!(x.edit_link.unwrap().url.as_str(), "https://example.org/one");
}

#[test]
fn a_selector_that_cannot_be_built_is_refused_by_name() {
    let mut definition = Config::parse("[profile.x]\n")
        .unwrap()
        .policy
        .profiles
        .remove(0);
    definition.region = vec!["p:has(a)".to_owned()];
    let e = Declarative::new(&definition).err().unwrap();
    assert!(e.starts_with("region: \"p:has(a)\""), "{e}");
}

#[test]
fn registries_built_from_one_definition_share_its_name_and_region() {
    let build = || {
        let definition = Config::parse(SPHINX).unwrap().policy.profiles.remove(0);
        Declarative::new(&definition).unwrap()
    };
    let (first, second) = (build(), build());
    assert!(std::ptr::eq(first.name(), second.name()));
    assert!(std::ptr::eq(first.region, second.region));
}
