use std::path::Path;

use super::*;
use crate::config::{Config, HashScope, ProfileChoice};
use crate::extract::{Extracted, Placement, extract};
use crate::profile::Registry;
use crate::profile::generic::Generic;

fn registry(toml: &str) -> Registry {
    let mut registry = Registry::new(Box::new(Generic));
    for definition in Config::parse(toml).unwrap().policy.profiles {
        registry = registry.with(Box::new(Declarative::new(&definition, None).unwrap()));
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
[[profile]]
name = "x"
region = ["div.document"]

[[profile.fact]]
key = "version"
selector = "meta[name=readthedocs-version-slug]"
attr = "content"

[[profile.fact]]
key = "content_root"
selector = "html"
attr = "data-content_root"

[[profile.fact]]
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
    let inherited = content(
        "[[profile]]\nname = \"x\"\n",
        &ProfileChoice::Named("x".into()),
    );
    assert_eq!(inherited, generic, "no region takes generic's");
}

const SPHINX_SOURCE: &str = r#"
[[profile.source]]
from = "url"
pattern = '^https://www\.sphinx-doc\.org/en/(?P<ref>[^/]+)/(?P<page>.+)\.html$'
repo = "https://github.com/sphinx-doc/sphinx"
path = "doc/${page}.rst"
"#;

fn source(x: &Extracted) -> Option<(&str, Option<&str>, &str)> {
    x.source
        .as_ref()
        .map(|s| (s.repo.as_str(), s.git_ref.as_deref(), s.path.as_str()))
}

#[test]
fn a_source_rule_gives_a_sphinx_page_its_source_with_no_edit_link() {
    let (url, html) = sphinx_page();
    let toml = format!("[[profile]]\nname = \"x\"\n{SPHINX_SOURCE}");
    let x = extract(
        &html,
        &url,
        &ProfileChoice::Named("x".into()),
        &HashScope::default(),
        &registry(&toml),
    )
    .unwrap();
    assert_eq!(x.edit_link, None);
    assert_eq!(
        source(&x),
        Some((
            "https://github.com/sphinx-doc/sphinx",
            Some("master"),
            "doc/usage/quickstart.rst"
        ))
    );
    let plain = named("[[profile]]\nname = \"x\"\n", &html);
    assert_eq!(plain.source, None, "without a rule the page has none");
}

#[test]
fn a_source_rule_beats_the_edit_links_parse_and_a_page_it_misses_keeps_it() {
    let html =
        "<main><a rel=edit href=\"https://github.com/o/r/edit/feature/x/docs/a.md\">e</a></main>";
    let toml = "[[profile]]\nname = \"x\"\n\
                [[profile.source]]\nfrom = \"edit_link\"\n\
                pattern = '^(?P<repo>https://github\\.com/o/r)/edit/(?P<ref>feature/x)/(?P<path>.+)$'\n";
    assert_eq!(
        source(&named(toml, html)),
        Some(("https://github.com/o/r", Some("feature/x"), "docs/a.md"))
    );
    let other = html.replace("feature/x", "main");
    assert_eq!(
        source(&named(toml, &other)),
        Some(("https://github.com/o/r", Some("main"), "docs/a.md")),
        "no rule matches, so the heuristic reads it"
    );
}

#[test]
fn a_page_with_none_of_the_markup_gives_no_fact() {
    let x = named(SPHINX, "<html><body><p>nothing</p></body></html>");
    assert!(facts(&x).is_empty());
}

#[test]
fn auto_picks_the_profile_whose_generator_pattern_matches() {
    let toml = "[[profile]]\nname = \"x\"\ngenerator = \"^Sphinx \\\\d\"\n";
    let page = |g: &str| {
        let html =
            format!("<html><head><meta name=generator content=\"{g}\"></head><body></body></html>");
        read(toml, &ProfileChoice::Auto, "https://example.org/", &html).profile
    };
    assert_eq!(page("Sphinx 7.2"), "x");
    assert_eq!(page("Antora 3.1"), "generic");
    assert_eq!(
        read(
            "[[profile]]\nname = \"x\"\n",
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
    let toml = "[[profile]]\nname = \"b\"\ngenerator = \"Doc\"\n[[profile]]\nname = \"a\"\ngenerator = \"Doc\"\n";
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
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"k\"\nselector = \"i\"\nattr = \"data-k\"\n";
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
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"k\"\nselector = \"i\"\nattr = \"data-k\"\npattern = \"^v(\\\\d+)$\"\n";
    let x = named(
        toml,
        "<i data-k=\"none\"></i><i data-k=\"v12\"></i><i data-k=\"v3\"></i>",
    );
    assert_eq!(facts(&x), [("x.k", "12")]);
}

#[test]
fn an_elements_text_is_trimmed_decoded_and_taken_through_its_descendants() {
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"t\"\nselector = \"p.t\"\n";
    let x = named(
        toml,
        "<p class=t>\n  Fish &amp; <b>chips</b> \n</p><p class=t>second</p>",
    );
    assert_eq!(facts(&x), [("x.t", "Fish & chips")]);
}

#[test]
fn an_element_with_no_text_gives_the_next_one_the_fact() {
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"t\"\nselector = \"p.t\"\n";
    let x = named(
        toml,
        "<p class=t> </p><p class=t>second</p><p class=t>third</p>",
    );
    assert_eq!(facts(&x), [("x.t", "second")]);
}

#[test]
fn a_nested_match_does_not_start_a_second_reading() {
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"t\"\nselector = \"div\"\n";
    let x = named(toml, "<div>outer <div>inner</div> tail</div>");
    assert_eq!(facts(&x), [("x.t", "outer inner tail")]);
}

#[test]
fn an_element_the_parser_closes_at_the_pages_end_gives_the_text_it_held() {
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"t\"\nselector = \"p\"\n";
    let x = named(toml, "<p>unclosed");
    assert_eq!(facts(&x), [("x.t", "unclosed")]);
}

#[test]
fn a_void_element_has_no_text() {
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"t\"\nselector = \"br\"\n";
    assert!(facts(&named(toml, "<br>text")).is_empty());
}

#[test]
fn two_rules_read_the_page_independently() {
    let toml = "[[profile]]\nname = \"x\"\n[[profile.fact]]\nkey = \"a\"\nselector = \"h1\"\n\
                [[profile.fact]]\nkey = \"b\"\nselector = \"h1\"\npattern = \"^(.)\"\n";
    let x = named(toml, "<h1>Title</h1>");
    assert_eq!(facts(&x), [("x.a", "Title"), ("x.b", "T")]);
}

#[test]
fn an_edit_link_selector_names_the_link_no_tier_earns() {
    let html = "<main><div class=meta><a href=\"https://codeberg.org/o/r/src/branch/main/docs/a.rst\">source</a></div></main>";
    let plain = named("[[profile]]\nname = \"x\"\n", html);
    assert!(plain.edit_link.is_none(), "no tier earns the link");
    let toml = "[[profile]]\nname = \"x\"\nedit_link = \"div.meta a\"\n";
    let x = named(toml, html);
    let edit = x.edit_link.expect("the profile named it");
    assert_eq!(
        edit.as_str(),
        "https://codeberg.org/o/r/src/branch/main/docs/a.rst"
    );
    assert_eq!(x.source.unwrap().repo, "https://codeberg.org/o/r");
}

#[test]
fn the_first_edit_link_the_selector_matches_is_the_one() {
    let toml = "[[profile]]\nname = \"x\"\nedit_link = \"a.e\"\n";
    let html = "<a class=e>no href</a><a class=e href=\"https://example.org/one\">1</a><a class=e href=\"https://example.org/two\">2</a>";
    let x = named(toml, html);
    assert_eq!(x.edit_link.unwrap().as_str(), "https://example.org/one");
}

#[test]
fn a_selector_that_cannot_be_built_is_refused_by_name() {
    let mut definition = Config::parse("[[profile]]\nname = \"x\"\n")
        .unwrap()
        .policy
        .profiles
        .remove(0);
    definition.region = vec!["p:has(a)".to_owned()];
    let e = Declarative::new(&definition, None).err().unwrap();
    assert!(e.starts_with("region: \"p:has(a)\""), "{e}");
    definition.region.clear();
    definition.sources = vec![crate::config::SourceRule {
        from: "url".to_owned(),
        pattern: "^x$".to_owned(),
        repo: None,
        git_ref: None,
        path: None,
    }];
    let e = Declarative::new(&definition, None).err().unwrap();
    assert!(e.starts_with("source[0]: repo:"), "{e}");
}

#[test]
fn registries_built_from_one_definition_share_its_name_and_region() {
    let build = || {
        let definition = Config::parse(SPHINX).unwrap().policy.profiles.remove(0);
        Declarative::new(&definition, None).unwrap()
    };
    let (first, second) = (build(), build());
    assert!(std::ptr::eq(first.name(), second.name()));
    assert!(std::ptr::eq(first.region.unwrap(), second.region.unwrap()));
}

const ANTORA_PAGE: &str = r##"<html><head><meta name="generator" content="Antora 3.1.15"></head>
<body><div class="nav-container" data-component="guide" data-version="1.0"></div>
<main id="chrome"><p id="outside">x</p></main>
<article class="doc"><h1 id="inside">T</h1>
<div class="edit-this-page"><a href="https://forge.example/org/repo/-/edit/main/guide/modules/ROOT/pages/a.adoc">Edit this Page</a></div>
<a class="xref unresolved" href="#guide:ROOT:a.adoc">a</a>
<meta name="own" content="mine"></article></body></html>"##;

const EXTENDS: &str = r#"
[[profile]]
name = "x"
extends = "antora"

[[profile.fact]]
key = "own"
selector = "meta[name=own]"
attr = "content"
"#;

fn extended(toml: &str) -> Registry {
    let defined = Config::parse(toml).unwrap().policy.profiles;
    crate::cli::registry_for(&defined).unwrap()
}

fn page(toml: &str, choice: &ProfileChoice) -> Extracted {
    let url = Url::parse("https://example.org/guide/1.0/a/").unwrap();
    extract(
        ANTORA_PAGE,
        &url,
        choice,
        &HashScope::default(),
        &extended(toml),
    )
    .unwrap()
}

#[test]
fn an_extension_keeps_its_bases_facts_in_the_bases_namespace_and_adds_its_own() {
    let x = page(EXTENDS, &ProfileChoice::Auto);
    assert_eq!(x.profile, "x", "the extension claims what its base claims");
    assert_eq!(
        facts(&x),
        [
            ("antora.component", "guide"),
            ("antora.version", "1.0"),
            ("antora.module", "ROOT"),
            ("antora.rel_path", "a.adoc"),
            ("x.own", "mine"),
        ]
    );
    let plain = page("", &ProfileChoice::Auto);
    assert_eq!(plain.profile, "antora");
    assert_eq!(x.edit_link, plain.edit_link);
    assert_eq!(x.source, plain.source);
}

#[test]
fn an_extension_takes_its_bases_region_unless_it_states_one() {
    let content = |toml: &str| {
        let x = page(toml, &ProfileChoice::Named("x".into()));
        let mut ids: Vec<_> = x
            .anchors
            .iter()
            .filter(|a| a.placement == Placement::Content)
            .map(|a| a.value.clone())
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(content(EXTENDS), ["inside"], "article.doc, antora's");
    let own = EXTENDS.replace(
        "extends = \"antora\"",
        "extends = \"antora\"\nregion = [\"main\"]",
    );
    assert_eq!(content(&own), ["chrome", "outside"], "its own replaces it");
}

#[test]
fn an_extension_detects_by_its_pattern_else_by_its_bases() {
    let claims = |toml: &str, generator: &str| {
        extended(toml)
            .iter()
            .find(|p| p.name() == "x")
            .unwrap()
            .detect(generator)
    };
    assert!(claims(EXTENDS, "Antora 3.1.15"));
    assert!(!claims(EXTENDS, "Sphinx 7"));
    let own = EXTENDS.replace(
        "extends = \"antora\"",
        "extends = \"antora\"\ngenerator = \"^Sphinx\"",
    );
    assert!(claims(&own, "Sphinx 7"));
    assert!(
        !claims(&own, "Antora 3.1.15"),
        "its pattern replaces the base's"
    );
}

#[test]
fn an_extension_resolves_and_notes_the_sitemap_as_its_base_does() {
    struct Pages;
    impl Lookup for Pages {
        fn pages_with_facts(&self, facts: &[(&str, &str)]) -> Result<Vec<Url>, BoxError> {
            let has = |k: &str, v: &str| facts.contains(&(k, v));
            Ok(
                if has("antora.component", "guide") && has("antora.rel_path", "a.adoc") {
                    vec![Url::parse("https://example.org/guide/1.0/a/").unwrap()]
                } else {
                    Vec::new()
                },
            )
        }
        fn facts(&self, _page: &Url) -> Result<Vec<(String, String)>, BoxError> {
            Ok(vec![
                ("antora.component".into(), "guide".into()),
                ("antora.module".into(), "ROOT".into()),
                ("antora.rel_path".into(), "a.adoc".into()),
            ])
        }
    }
    let registry = extended(EXTENDS);
    let (extension, base) = (registry.get("x").unwrap(), registry.get("antora").unwrap());
    let from = Url::parse("https://example.org/guide/1.0/b/").unwrap();
    let resolve = |p: &dyn Profile| p.resolve("guide:ROOT:a.adoc", &from, &Pages).unwrap();
    assert!(resolve(base).is_some(), "the base resolves this refspec");
    assert_eq!(resolve(extension), resolve(base));
    assert_eq!(extension.sitemap_note(), base.sitemap_note());
    assert!(extension.sitemap_note().is_some());
    assert!(
        extension.views().is_empty(),
        "the base installs its own, once"
    );
}

#[test]
fn an_extension_of_generic_reads_as_generic_does_where_it_states_no_rule() {
    let toml = "[[profile]]\nname = \"x\"\nextends = \"generic\"\n";
    let (url, html) = sphinx_page();
    let content = |name: &str| {
        let x = extract(
            &html,
            &url,
            &ProfileChoice::Named(name.into()),
            &HashScope::default(),
            &extended(toml),
        )
        .unwrap();
        (x.profile, count_content(&x), x.hash)
    };
    let (extension, generic) = (content("x"), content("generic"));
    assert_eq!((extension.1, extension.2), (generic.1, generic.2));
    assert_eq!(extension.0, "x");
}

/// D67: a profile's `locale_links` names the links of a language menu, each
/// the page under another locale; a profile without the key, `antora` itself
/// included, learns none from the same menu.
#[test]
fn locale_links_name_the_menu_whose_locales_a_page_gives() {
    let menu = r#"<div class="menu"><a class="language" href="../../en/a/">en</a>
        <a class="language" href="../../fr/a/">fr</a>
        <a class="language" href="../../pt_BR/a/?q#f">pt</a>
        <a class="language" href="https://other.org/de/a/">de</a>
        <a class="language">none</a></div>
        <a class="other" href="../../es/a/">es</a>"#;
    let html = format!(
        r#"<html><head><meta name="generator" content="Antora 3.1.15"></head>
        <body><article class="doc">{menu}</article></body></html>"#
    );
    let url = Url::parse("https://example.org/en/a/").unwrap();
    let locales = |toml: &str| -> Vec<String> {
        extract(
            &html,
            &url,
            &ProfileChoice::Auto,
            &HashScope::default(),
            &extended(toml),
        )
        .unwrap()
        .other_locales
        .iter()
        .map(|u| u.to_string())
        .collect()
    };
    let with =
        "[[profile]]\nname = \"x\"\nextends = \"antora\"\nlocale_links = \"a.language[href]\"\n";
    assert_eq!(
        locales(with),
        ["https://example.org/fr/", "https://example.org/pt_BR/"]
    );
    assert!(locales(EXTENDS).is_empty(), "an extension without the key");
    assert!(locales("").is_empty(), "`antora` reads no menu");
    let alone = "[[profile]]\nname = \"x\"\ngenerator = \"^Antora\"\nlocale_links = \"a.other\"\n";
    assert_eq!(locales(alone), ["https://example.org/es/"]);
}

fn count_content(x: &Extracted) -> usize {
    x.anchors
        .iter()
        .filter(|a| a.placement == Placement::Content)
        .count()
}
