//! Which selectors lol_html compiles: the vocabulary D33's rules would get.

#[test]
fn supported_selectors() {
    let ok = |s: &str| s.parse::<lol_html::Selector>().is_ok();
    for s in [
        "a", "*", ".doc", "#x", "[rel=edit]", "[rel~=edit]", "[class*=edit]", "[href^=http]",
        "[title=\"Edit this Page\" i]", "article.doc a", "div > a", "a:not(.xref)",
        "li:nth-child(2)", "li:first-child", "main, [role=main]",
    ] {
        assert!(ok(s), "{s} should compile");
    }
    for s in ["h2 + p", "h2 ~ p", "div:has(> a)", "a:last-child", "li:nth-last-child(1)", "p::before", "a:hover"] {
        assert!(!ok(s), "{s} should be refused");
    }
}
