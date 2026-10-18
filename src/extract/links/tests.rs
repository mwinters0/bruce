use super::*;

#[test]
fn srcset_urls_are_the_candidates_not_the_descriptors() {
    assert_eq!(srcset_urls("a.png 1x, b.png 2x"), ["a.png", "b.png"]);
    assert_eq!(
        srcset_urls("a.png,b.png"),
        ["a.png,b.png"],
        "commas inside a run of non-whitespace are the URL's"
    );
    assert_eq!(srcset_urls("a.png, b.png"), ["a.png", "b.png"]);
    assert_eq!(srcset_urls("  a.png  ,  b.png  "), ["a.png", "b.png"]);
    assert_eq!(srcset_urls("a.png, b.png 100w"), ["a.png", "b.png"]);
    assert_eq!(
        srcset_urls("data:image/png;base64,AAA 1x, b.png"),
        ["data:image/png;base64,AAA", "b.png"],
        "a URL may hold a comma"
    );
    assert_eq!(
        srcset_urls("a.png calc(1x, 2x), b.png"),
        ["a.png", "b.png"],
        "a comma inside parentheses ends nothing"
    );
    assert!(srcset_urls("").is_empty());
    assert!(srcset_urls(" , ,").is_empty());
}

#[test]
fn a_refresh_names_a_url_after_its_delay() {
    let url = |c: &str| refresh_url(c);
    assert_eq!(url("0; url=../x/").as_deref(), Some("../x/"));
    assert_eq!(url("0;URL=/x").as_deref(), Some("/x"));
    assert_eq!(url(" 5 , url = /x").as_deref(), Some("/x"));
    assert_eq!(url("0;url='a b'").as_deref(), Some("a b"));
    assert_eq!(url("0; url=\"/x\"").as_deref(), Some("/x"));
    assert_eq!(url("0; /x").as_deref(), Some("/x"), "url= is optional");
    assert_eq!(url("0 /x").as_deref(), Some("/x"));
    assert_eq!(url("1.5;url=/x").as_deref(), Some("/x"));
    assert_eq!(url("30"), None, "a plain reload");
    assert_eq!(url("0;"), None);
    assert_eq!(url("0; url="), None);
    assert_eq!(url("soon; url=/x"), None);
    assert_eq!(url("0x; url=/x"), None);
    assert_eq!(url(""), None);
}

#[test]
fn a_links_rel_decides_its_kind() {
    assert_eq!(link_kind("stylesheet"), Some(Kind::Stylesheet));
    assert_eq!(link_kind("Canonical"), Some(Kind::Canonical));
    assert_eq!(link_kind("shortcut icon"), Some(Kind::Icon));
    assert_eq!(link_kind("alternate stylesheet"), Some(Kind::Stylesheet));
    assert_eq!(link_kind("apple-touch-icon"), Some(Kind::Link));
    assert_eq!(link_kind("preload"), Some(Kind::Link));
    assert_eq!(link_kind("preconnect"), None);
    assert_eq!(link_kind("dns-prefetch preconnect"), None);
    assert_eq!(link_kind("preconnect preload"), Some(Kind::Link));
    assert_eq!(link_kind(""), None);
}

#[test]
fn a_sitemap_entry_has_a_kind_no_page_yields() {
    assert_eq!(Kind::Sitemap.as_str(), "sitemap");
}
