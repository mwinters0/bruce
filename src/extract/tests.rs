use std::cell::RefCell;

use super::*;
use crate::profile::{BoxError, Extract, Lookup, Profile, View};

/// A profile whose region is `main`, claiming generators starting `Stub`.
struct Stub;

impl Profile for Stub {
    fn name(&self) -> &'static str {
        "stub"
    }

    fn detect(&self, generator: &str) -> bool {
        generator.starts_with("Stub")
    }

    fn extract<'h>(&self, sink: &'h RefCell<Sink>) -> Extract<'h> {
        let mut extract = Extract::region(&["main"]);
        extract.handlers.push(element!("html", move |el| {
            if let Some(lang) = el.get_attribute("lang") {
                sink.borrow_mut().fact("lang", lang);
            }
            Ok(())
        }));
        extract.handlers.push(element!(".tagme", move |_| {
            sink.borrow_mut().tag_link("tag");
            Ok(())
        }));
        extract.finish = Some(|settled, sink| sink.fact("host", settled.url.host_str().unwrap()));
        extract
    }

    fn resolve(&self, _: &str, _: &Url, _: &dyn Lookup) -> Result<Option<Url>, BoxError> {
        Ok(None)
    }

    fn views(&self) -> &'static [View] {
        &[]
    }
}

/// Plain: claims nothing, has no region, the registry's fallback.
struct Plain;

impl Profile for Plain {
    fn name(&self) -> &'static str {
        "plain"
    }

    fn detect(&self, _: &str) -> bool {
        false
    }

    fn extract<'h>(&self, _: &'h RefCell<Sink>) -> Extract<'h> {
        Extract::region(&[])
    }

    fn resolve(&self, _: &str, _: &Url, _: &dyn Lookup) -> Result<Option<Url>, BoxError> {
        Ok(None)
    }

    fn views(&self) -> &'static [View] {
        &[]
    }
}

fn run(html: &str, choice: &ProfileChoice) -> Result<Extracted, Error> {
    let registry = Registry::new(Box::new(Plain)).with(Box::new(Stub));
    let url = Url::parse("https://example.org/a/").unwrap();
    extract(html, &url, choice, &registry)
}

fn auto(html: &str) -> Extracted {
    run(html, &ProfileChoice::Auto).unwrap()
}

fn anchors(x: &Extracted) -> Vec<(&str, &str, &str)> {
    x.anchors
        .iter()
        .map(|a| (a.value.as_str(), a.tag.as_str(), a.placement.as_str()))
        .collect()
}

#[test]
fn the_generator_meta_picks_the_profile_and_stops_at_the_body() {
    let head = r#"<head><meta name="generator" content="Stub 1"></head>"#;
    assert_eq!(auto(&format!("<html>{head}<body>")).profile, "stub");
    let late = r#"<head></head><body><meta name="generator" content="Stub 1">"#;
    assert_eq!(auto(late).profile, "plain", "past <body> no profile looks");
    let first =
        r#"<head><META NAME="Generator" content="Other"><meta name="generator" content="Stub 1">"#;
    assert_eq!(auto(first).profile, "plain", "the first meta decides");
    assert_eq!(auto("<p>no head at all").profile, "plain");
}

#[test]
fn a_named_profile_ignores_the_meta_and_an_unknown_one_is_refused() {
    let html = r#"<meta name="generator" content="Stub 1"><body>"#;
    let plain = ProfileChoice::Named("plain".into());
    assert_eq!(run(html, &plain).unwrap().profile, "plain");
    let nope = ProfileChoice::Named("nope".into());
    let err = run(html, &nope).unwrap_err();
    assert_eq!(err.to_string(), "unknown profile \"nope\"");
}

#[test]
fn the_title_is_the_first_outside_an_svg_decoded_and_collapsed() {
    let html = "<svg><title>icon</title></svg><title>\n  A &amp; B\t  C </title><title>two</title>";
    assert_eq!(auto(html).title.as_deref(), Some("A & B C"));
    assert_eq!(auto("<title>  </title>").title, None);
    assert_eq!(auto("<title></title><title>later</title>").title, None);
    assert_eq!(auto("<p>no title").title, None);
}

#[test]
fn an_anchor_is_a_nonempty_id_or_a_name_on_a_link_d12() {
    let html = r#"<main><h2 id="top-h">x</h2><a name="old">y</a><a id="both" name="both">z</a>
        <a id="i" name="n">w</a><meta name="viewport"><div name="d" id=""></div>
        <a name="">e</a><p id="a&amp;b"></p></main>"#;
    assert_eq!(
        anchors(&auto(html)),
        [
            ("top-h", "h2", "unknown"),
            ("old", "a", "unknown"),
            ("both", "a", "unknown"),
            ("i", "a", "unknown"),
            ("n", "a", "unknown"),
            ("a&b", "p", "unknown"),
        ],
        "an anchor's placement is unknown while the profile has no region"
    );
}

#[test]
fn placement_follows_the_profiles_region_d34() {
    let stub = r#"<meta name="generator" content="Stub"><body>
        <nav id="chrome"></nav>
        <main id="m"><h1 id="in"></h1><a name="n"></a><div><i id="deep"></i></div></main>
        <footer id="f"></footer><a name="late"></a>"#;
    assert_eq!(
        anchors(&auto(stub)),
        [
            ("chrome", "nav", "chrome"),
            ("m", "main", "content"),
            ("in", "h1", "content"),
            ("n", "a", "content"),
            ("deep", "i", "content"),
            ("f", "footer", "chrome"),
            ("late", "a", "chrome"),
        ]
    );
}

#[test]
fn a_page_without_the_region_has_only_unknown_anchors_d34() {
    let html = r#"<meta name="generator" content="Stub"><body><p id="a"></p><p id="b"></p>"#;
    assert!(
        auto(html)
            .anchors
            .iter()
            .all(|a| a.placement == Placement::Unknown)
    );
    assert_eq!(auto(html).anchors.len(), 2);
}

#[test]
fn a_region_that_comes_after_an_anchor_still_decides_the_page() {
    let html = r#"<meta name="generator" content="Stub"><body><p id="before"></p><main></main>"#;
    assert_eq!(anchors(&auto(html)), [("before", "p", "chrome")]);
}

#[test]
fn handlers_and_the_finish_step_write_facts() {
    let x = auto(r#"<html lang="en"><meta name="generator" content="Stub">"#);
    assert_eq!(
        x.facts.pairs(),
        [
            ("stub.lang".to_string(), "en".to_string()),
            ("stub.host".to_string(), "example.org".to_string())
        ]
    );
}

#[test]
fn markup_lol_html_cannot_place_does_not_stop_the_pass() {
    // Strict mode refuses this page; the `<xmp>` body is text either way.
    let html = r#"<select><xmp><p id="x"></xmp></select><i id="y"></i>"#;
    assert_eq!(anchors(&auto(html)), [("y", "i", "unknown")]);
}

#[test]
#[should_panic(expected = "a profile declared the region selector")]
fn a_region_selector_that_does_not_parse_is_a_bug_in_the_profile() {
    let _ = anchors::Anchors::handlers(&RefCell::default(), &["a + b"]);
}

/// A link as `(raw, resolved, kind)`, against `https://example.org/a/`.
fn links(html: &str) -> Vec<(String, String, &'static str)> {
    auto(html)
        .links
        .iter()
        .map(|l| {
            let url = l.url.as_ref().map_or("-", |u| u.as_str());
            (l.raw.clone(), url.to_string(), l.kind.as_str())
        })
        .collect()
}

fn link(raw: &str, url: &str, kind: &'static str) -> (String, String, &'static str) {
    (raw.to_string(), url.to_string(), kind)
}

#[test]
fn every_kind_is_read_in_document_order_d13() {
    let html = r#"<head>
        <link rel="canonical" href="/c/"><link rel="stylesheet" href="s.css">
        <link rel="icon" href="/i.ico"><link rel="alternate" href="/feed">
        <link rel="preconnect" href="https://cdn.example.net"><link href="/norel">
        <meta http-equiv="Refresh" content="0; url=../r/"><meta name="refresh" content="0; url=/no">
        </head><body>
        <a href="b">a</a><a>no href</a><area href="/area"><img src="p.png" srcset="p2.png 2x, p3.png 3x">
        <picture><source srcset="s.webp"><img src="p.png"></picture>
        <script src="/j.js"></script><script>inline</script><iframe src="/f"></iframe>
        <video src="v.mp4" poster="v.jpg"><source src="v.webm"><track src="v.vtt"></video>
        <audio src="a.ogg"></audio>"#;
    assert_eq!(
        links(html),
        [
            link("/c/", "https://example.org/c/", "canonical"),
            link("s.css", "https://example.org/a/s.css", "stylesheet"),
            link("/i.ico", "https://example.org/i.ico", "icon"),
            link("/feed", "https://example.org/feed", "link"),
            link("../r/", "https://example.org/r/", "refresh"),
            link("b", "https://example.org/a/b", "anchor"),
            link("/area", "https://example.org/area", "anchor"),
            link("p.png", "https://example.org/a/p.png", "image"),
            link("p2.png", "https://example.org/a/p2.png", "image"),
            link("p3.png", "https://example.org/a/p3.png", "image"),
            link("s.webp", "https://example.org/a/s.webp", "image"),
            link("p.png", "https://example.org/a/p.png", "image"),
            link("/j.js", "https://example.org/j.js", "script"),
            link("/f", "https://example.org/f", "frame"),
            link("v.mp4", "https://example.org/a/v.mp4", "media"),
            link("v.jpg", "https://example.org/a/v.jpg", "media"),
            link("v.webm", "https://example.org/a/v.webm", "media"),
            link("v.vtt", "https://example.org/a/v.vtt", "media"),
            link("a.ogg", "https://example.org/a/a.ogg", "media"),
        ]
    );
}

#[test]
fn an_href_is_kept_raw_and_resolved_against_the_page() {
    let html = r##"<a href="#frag">1</a><a href="">2</a><a href="?q=1">3</a><a href="//cdn.example.net/x">4</a>
        <a href="mailto:me@example.org">5</a><a href="http://[bad">6</a><a href="x&amp;y">7</a>
        <img src="" srcset=""><script src="  "></script>"##;
    assert_eq!(
        links(html),
        [
            link("#frag", "https://example.org/a/#frag", "anchor"),
            link("", "https://example.org/a/", "anchor"),
            link("?q=1", "https://example.org/a/?q=1", "anchor"),
            link("//cdn.example.net/x", "https://cdn.example.net/x", "anchor"),
            link("mailto:me@example.org", "mailto:me@example.org", "anchor"),
            link("http://[bad", "-", "anchor"),
            link("x&y", "https://example.org/a/x&y", "anchor"),
        ],
        "an empty src names no resource, an empty href is the page"
    );
}

#[test]
fn the_first_base_href_resolves_every_later_link() {
    let html = r##"<a href="before">0</a><base target="_blank"><base href="/b/dir/"><base href="/other/">
        <a href="x">1</a><a href="#f">2</a><a href="/abs">3</a><img src="i.png">"##;
    assert_eq!(
        links(html),
        [
            link("before", "https://example.org/a/before", "anchor"),
            link("x", "https://example.org/b/dir/x", "anchor"),
            link("#f", "https://example.org/b/dir/#f", "anchor"),
            link("/abs", "https://example.org/abs", "anchor"),
            link("i.png", "https://example.org/b/dir/i.png", "image"),
        ]
    );
    let relative = r#"<base href="sub/"><a href="x">1</a>"#;
    assert_eq!(
        links(relative),
        [link("x", "https://example.org/a/sub/x", "anchor")],
        "a relative base resolves against the page"
    );
    let broken = r#"<base href="http://[bad"><base href="/b/"><a href="x">1</a>"#;
    assert_eq!(
        links(broken),
        [link("x", "https://example.org/a/x", "anchor")],
        "a first base that does not parse is the one that counts, and counts for nothing"
    );
}

#[test]
fn a_tag_set_by_a_profiles_handler_lands_on_the_links_of_its_element() {
    let x = auto(
        r#"<meta name="generator" content="Stub"><a href="1">a</a><a class="tagme" href="2">b</a>
        <img class="tagme" src="p.png" srcset="p2.png"><a href="3">c</a><span class="tagme"></span>
        <a href="4">d</a>"#,
    );
    let tags: Vec<_> = x
        .links
        .iter()
        .map(|l| (l.raw.as_str(), l.tag.as_deref()))
        .collect();
    assert_eq!(
        tags,
        [
            ("1", None),
            ("2", Some("tag")),
            ("p.png", Some("tag")),
            ("p2.png", Some("tag")),
            ("3", None),
            ("4", None),
        ],
        "a tag set on an element that yields no link is dropped, not carried to the next"
    );
}
