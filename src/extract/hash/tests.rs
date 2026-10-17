use lol_html::{HtmlRewriter, Settings};

use super::*;

/// The hash of `html` under a root `root` and exclusions `exclude`, for a
/// profile whose region is `main`.
fn hash(html: &str, root: &[&str], exclude: &[&str]) -> u128 {
    let scope = HashScope {
        root: root.iter().map(|s| s.to_string()).collect(),
        exclude: exclude.iter().map(|s| s.to_string()).collect(),
    };
    let state = RefCell::new(Hash::default());
    let mut settings = Settings::new().with_strict(false);
    for handler in Hash::handlers(&state, &scope, &["main"]) {
        settings = settings.append_element_content_handler(handler);
    }
    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
    rewriter.write(html.as_bytes()).unwrap();
    rewriter.end().unwrap();
    state.into_inner().finish()
}

fn page(head: &str, main: &str, footer: &str) -> String {
    format!(
        "<html><head>{head}</head><body><nav>menu</nav><main>{main}</main><footer>{footer}</footer></body></html>"
    )
}

const HEAD: &str = r#"<link rel="stylesheet" href="a.css">"#;

#[test]
fn by_default_the_region_is_hashed_and_nothing_outside_it() {
    let base = hash(&page(HEAD, "<p>text</p>", "built 1"), &[], &[]);
    assert_eq!(base, hash(&page("", "<p>text</p>", "built 2"), &[], &[]));
    assert_ne!(base, hash(&page(HEAD, "<p>text.</p>", "built 1"), &[], &[]));
    assert_ne!(
        base,
        hash(&page(HEAD, "<p id=x>text</p>", "built 1"), &[], &[])
    );
}

#[test]
fn a_page_with_no_root_element_is_hashed_whole() {
    let none = "<html><body><div>text</div><footer>built 1</footer>";
    let base = hash(none, &[], &[]);
    assert_ne!(base, hash(&none.replace("built 1", "built 2"), &[], &[]));
    let root = ["article"];
    assert_eq!(base, hash(none, &root, &[]), "a root matching nothing");
}

#[test]
fn hash_root_replaces_the_region() {
    let root = ["nav"];
    let base = hash(&page(HEAD, "a", "built 1"), &root, &[]);
    assert_eq!(base, hash(&page("", "b", "built 2"), &root, &[]));
    let other = page(HEAD, "a", "built 1").replace("menu", "menu 2");
    assert_ne!(base, hash(&other, &root, &[]));
}

#[test]
fn hash_exclude_alone_hashes_the_whole_page_but_the_exclusions() {
    let exclude = ["footer"];
    let base = hash(&page(HEAD, "a", "built 1"), &[], &exclude);
    let footer = page(HEAD, "a", "built 2").replace("<footer>", "<footer class=x>");
    assert_eq!(base, hash(&footer, &[], &exclude));
    assert_ne!(
        base,
        hash(&page("", "a", "built 1"), &[], &exclude),
        "a stylesheet link"
    );
    assert_ne!(base, hash(&page(HEAD, "b", "built 1"), &[], &exclude));
    let nav = page(HEAD, "a", "built 1").replace("menu", "menu 2");
    assert_ne!(base, hash(&nav, &[], &exclude), "outside the region too");
}

#[test]
fn both_keys_hash_the_root_less_the_exclusions() {
    let (root, exclude) = (["body"], ["footer", "nav"]);
    let base = hash(&page(HEAD, "a", "built 1"), &root, &exclude);
    let chrome = page("", "a", "built 2").replace("menu", "menu 2");
    assert_eq!(base, hash(&chrome, &root, &exclude));
    assert_ne!(base, hash(&page(HEAD, "b", "built 1"), &root, &exclude));
}

#[test]
fn text_and_attributes_are_hashed_decoded() {
    let a = page("", r#"<p title="&amp;">a &amp; b</p>"#, "");
    let b = page("", r#"<p title="&#38;">a &#x26; b</p>"#, "");
    assert_eq!(hash(&a, &[], &[]), hash(&b, &[], &[]));
}

#[test]
fn where_a_tag_falls_in_the_text_counts() {
    let a = page("", "<b>x</b>y", "");
    let b = page("", "<b>xy</b>", "");
    assert_ne!(hash(&a, &[], &[]), hash(&b, &[], &[]));
}

#[test]
fn the_roots_own_tag_is_hashed() {
    let a = page("", "x", "");
    let b = a.replace("<main>", "<main class=wide>");
    assert_ne!(hash(&a, &[], &[]), hash(&b, &[], &[]));
}

#[test]
fn a_text_node_split_across_chunks_hashes_as_one() {
    let html = page(HEAD, "<p>a &amp; b, and more text</p>", "built");
    let state = RefCell::new(Hash::default());
    let mut settings = Settings::new().with_strict(false);
    for handler in Hash::handlers(&state, &HashScope::default(), &["main"]) {
        settings = settings.append_element_content_handler(handler);
    }
    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
    for byte in html.as_bytes().chunks(1) {
        rewriter.write(byte).unwrap();
    }
    rewriter.end().unwrap();
    assert_eq!(state.into_inner().finish(), hash(&html, &[], &[]));
}
