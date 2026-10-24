//! The `antora` profile: Antora's generator meta, nav container, content
//! region and xrefs (D19, D20). Resolve is the `resolve` child's.
//!
//! Extract records a page's coordinates as facts (D19, I8): `component` and
//! `version` from the nav container, verbatim; `module` and `rel_path` from
//! the edit URL's `modules/<module>/pages/<rel_path>` tail. A page with no
//! nav container (a `without_menu` layout) takes its `component` from the first
//! directory under the site root the search script names (I13), and records
//! no `version`. A nav container carrying both attributes, as docs.fp.o's
//! does (I8), makes a component with no version fact a URL-derived one; one
//! lacking `data-component` (I12) gives no component at all. An unresolved xref is tagged with its raw refspec (I6, D20).
//!
//! The language menu's links (`a.language`, I25) say where the site's other
//! locales are: each is the page under another locale, so the segment where its
//! path departs from the page's own is that locale's prefix (D67).

use std::cell::RefCell;
use std::rc::Rc;

use lol_html::element;
use percent_encoding::percent_decode_str;
use url::Url;

use super::{BoxError, Extract, Lookup, Profile, Settled, Sink, View};

mod hints;
mod resolve;

/// The profile for sites built by Antora.
pub struct Antora;

/// A page's Antora coordinates as columns (D19, D37). `version` is the nav
/// container's, verbatim, a placeholder (`""`, `master`) included, and NULL
/// where it has no `data-version` or the page has no nav container, whose
/// `component` then came from its URL. Resolve ranks a NULL and `""` as a
/// bare version, which Antora lists first, and `master` as any other name
/// (`resolve::compare`), and passes a NULL over where it needs a prefix.
const PAGES: View = View {
    name: "antora_pages",
    select: "SELECT run_id, url_id, \
        max(value) FILTER (WHERE key = 'antora.component') AS component, \
        max(value) FILTER (WHERE key = 'antora.version') AS version, \
        max(value) FILTER (WHERE key = 'antora.module') AS module, \
        max(value) FILTER (WHERE key = 'antora.rel_path') AS rel_path \
        FROM page_facts WHERE key LIKE 'antora.%' GROUP BY run_id, url_id",
};

/// What the pass read that the finish step needs.
#[derive(Default)]
struct Page {
    /// The page has a nav container, which carries its component.
    nav_container: bool,
    /// `data-site-root-path` on the search script (I13).
    site_root_path: Option<String>,
    /// The language menu's hrefs, as written (I25).
    languages: Vec<String>,
}

impl Page {
    /// Records what only the settled page can give: the component of a page
    /// with no nav container, from its URL, the module and `rel_path`
    /// from the edit link, and the other locales its language menu names.
    fn finish(&self, settled: &Settled, sink: &mut Sink) {
        for href in &self.languages {
            let link = (settled.resolve)(href);
            if let Some(prefix) = link.and_then(|link| other_locale(settled.url, &link)) {
                sink.other_locale(prefix);
            }
        }
        if !self.nav_container {
            let root = self.site_root_path.as_deref();
            if let Some(component) = root.and_then(|root| component_under(settled.url, root)) {
                sink.fact("component", component);
            }
        }
        let source = settled.edit_link.and_then(|e| e.source.as_ref());
        if let Some((module, rel_path)) = source.and_then(|s| module_and_path(&s.path)) {
            sink.fact("module", module);
            sink.fact("rel_path", rel_path);
        }
    }
}

/// The first directory of `page` under the site root `root_path` names: the
/// search script's `data-site-root-path`, relative to the page's directory or
/// absolute (I13). `None` where the page is not inside a directory under the
/// root, or the root is on another origin.
fn component_under(page: &Url, root_path: &str) -> Option<String> {
    let root = page.join(root_path).ok()?;
    if root.origin() != page.origin() {
        return None;
    }
    let root = root.path().trim_end_matches('/');
    let below = page.path().strip_prefix(root)?.strip_prefix('/')?;
    let (component, _) = below.split_once('/')?;
    (!component.is_empty()).then(|| decode(component))
}

/// The prefix of the locale a language menu's `link` on `page` names: the
/// link's path up to the first segment where it departs from the page's, as a
/// directory (I25). `None` for a link to the page itself, on another origin,
/// or whose path the page's begins with.
fn other_locale(page: &Url, link: &Url) -> Option<Url> {
    if link.origin() != page.origin() {
        return None;
    }
    let mut mine = page.path().split('/');
    let mut prefix = String::new();
    for segment in link.path().split('/') {
        prefix.push_str(segment);
        prefix.push('/');
        if mine.next() != Some(segment) {
            if segment.is_empty() {
                return None;
            }
            let mut locale = link.clone();
            locale.set_path(&prefix);
            locale.set_query(None);
            locale.set_fragment(None);
            return Some(locale);
        }
    }
    None
}

/// `(module, rel_path)` from an edit path holding `modules/<module>/pages/<rel_path>`
/// somewhere below a start path, as Antora lays a repo out.
fn module_and_path(path: &str) -> Option<(String, String)> {
    let segments: Vec<&str> = path.split('/').collect();
    let at = segments
        .windows(3)
        .position(|w| w[0] == "modules" && w[2] == "pages")?;
    let rel_path = &segments[at + 3..];
    if rel_path.is_empty() || rel_path.iter().any(|s| s.is_empty()) {
        return None;
    }
    let module = decode(segments[at + 1]);
    Some((module, decode(&rel_path.join("/"))))
}

fn decode(segment: &str) -> String {
    percent_decode_str(segment).decode_utf8_lossy().into_owned()
}

impl Profile for Antora {
    fn name(&self) -> &'static str {
        "antora"
    }

    /// Antora's generator meta reads `Antora 3.1.15` (I12): the first word.
    fn detect(&self, generator: &str) -> bool {
        generator
            .split_whitespace()
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("antora"))
    }

    fn extract<'h>(&self, sink: &'h RefCell<Sink>) -> Extract<'h> {
        let page = Rc::new(RefCell::new(Page::default()));
        let mut extract = Extract::region(&["article.doc"]);
        let seen = Rc::clone(&page);
        extract.handlers.push(element!(".nav-container", move |el| {
            let mut page = seen.borrow_mut();
            if !std::mem::replace(&mut page.nav_container, true) {
                let mut sink = sink.borrow_mut();
                if let Some(component) = el.get_attribute("data-component") {
                    sink.fact("component", component);
                }
                if let Some(version) = el.get_attribute("data-version") {
                    sink.fact("version", version);
                }
            }
            Ok(())
        }));
        let seen = Rc::clone(&page);
        extract.handlers.push(element!(
            "script#search-ui-script[data-site-root-path]",
            move |el| {
                let mut page = seen.borrow_mut();
                if page.site_root_path.is_none() {
                    page.site_root_path = el.get_attribute("data-site-root-path");
                }
                Ok(())
            }
        ));
        let seen = Rc::clone(&page);
        extract
            .handlers
            .push(element!("a.language[href]", move |el| {
                let href = el.get_attribute("href").unwrap_or_default();
                seen.borrow_mut().languages.push(href);
                Ok(())
            }));
        extract
            .handlers
            .push(element!("a.xref.unresolved[href]", move |el| {
                let href = el.get_attribute("href").unwrap_or_default();
                if let Some(refspec) = href.strip_prefix('#') {
                    sink.borrow_mut().tag_link(refspec);
                }
                Ok(())
            }));
        extract.finish = Some(Box::new(move |settled, sink| {
            page.borrow().finish(settled, sink);
        }));
        extract
    }

    /// A refspec naming no component is the page's own and is not resolved
    /// (`resolve`), so the page it is on is not read.
    fn resolve(
        &self,
        reference: &str,
        _from: &Url,
        lookup: &dyn Lookup,
    ) -> Result<Option<Url>, BoxError> {
        resolve::resolve(reference, lookup)
    }

    fn views(&self) -> &'static [View] {
        &[PAGES, hints::HINTS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_claims_antora_generators_only() {
        assert!(Antora.detect("Antora 3.1.15"));
        assert!(Antora.detect("antora"));
        assert!(!Antora.detect("Docusaurus v4.0.0"));
        assert!(!Antora.detect("Antoraesque 1.0"));
        assert!(!Antora.detect(""));
    }

    fn component(page: &str, root: &str) -> Option<String> {
        component_under(&Url::parse(page).unwrap(), root)
    }

    /// The search script's root is relative to the page's directory on a
    /// page and absolute on the 404 (I13).
    #[test]
    fn a_component_is_the_first_directory_under_the_site_root() {
        let arm = "https://docs.fedoraproject.org/en-US/arm-sbc/";
        assert_eq!(component(arm, "..").as_deref(), Some("arm-sbc"));
        let deep = "https://docs.fedoraproject.org/en-US/guide/a/b/";
        assert_eq!(component(deep, "../../..").as_deref(), Some("guide"));
        assert_eq!(component(deep, "/en-US").as_deref(), Some("guide"));
        assert_eq!(component(deep, "/en-US/").as_deref(), Some("guide"));
        let file = "https://example.org/site/tools/page.html";
        assert_eq!(component(file, "..").as_deref(), Some("tools"));
        let encoded = "https://example.org/a%20b/";
        assert_eq!(component(encoded, "..").as_deref(), Some("a b"));
        assert_eq!(component(encoded, "."), None, "the page is the root");
        assert_eq!(component("https://example.org/", "."), None);
        assert_eq!(component("https://example.org/page.html", "."), None);
        assert_eq!(component(arm, "/elsewhere"), None, "not under the root");
        assert_eq!(component(arm, "/en-U"), None, "a prefix is no directory");
        assert_eq!(component(arm, "https://other.org/en-US"), None);
    }

    fn locale(page: &str, href: &str) -> Option<String> {
        let page = Url::parse(page).unwrap();
        other_locale(&page, &page.join(href).unwrap()).map(String::from)
    }

    /// docs.fp.o's menu names the page under each locale, its own first, and
    /// the 404's names each locale's home (I25).
    #[test]
    fn a_locale_is_where_a_menu_link_departs_from_the_page() {
        let a11y = "https://docs.fedoraproject.org/en-US/a11y/";
        let ar = Some("https://docs.fedoraproject.org/ar/".to_owned());
        assert_eq!(locale(a11y, "../../ar/a11y/"), ar);
        assert_eq!(locale(a11y, "../../en-US/a11y/"), None, "its own locale");
        let deep = "https://docs.fedoraproject.org/en-US/fedora/f39/";
        assert_eq!(locale(deep, "../../../ar/fedora/f39/"), ar);
        let missing = "https://docs.fedoraproject.org/en-US/no-such/";
        assert_eq!(locale(missing, "/en-US/../ar"), ar, "the 404's");
        assert_eq!(locale(missing, "/en-US/../en-US"), None);
        assert_eq!(
            locale("https://e.org/docs/en/a/?q#f", "/docs/pt_BR/a/?q#f").as_deref(),
            Some("https://e.org/docs/pt_BR/"),
            "below the host's root, with no query or fragment"
        );
        assert_eq!(locale("https://e.org/en/a", "/en/a/"), None);
        assert_eq!(locale(a11y, "https://other.org/ar/a11y/"), None);
        assert_eq!(locale(a11y, "http://docs.fedoraproject.org/ar/a11y/"), None);
    }

    #[test]
    fn module_and_rel_path_are_the_tail_under_a_start_path() {
        let pair = |m: &str, p: &str| Some((m.to_string(), p.to_string()));
        assert_eq!(
            module_and_path("arm-sbc/modules/ROOT/pages/index.adoc"),
            pair("ROOT", "index.adoc")
        );
        assert_eq!(
            module_and_path("modules/user/pages/a/b%20c.adoc"),
            pair("user", "a/b c.adoc")
        );
        assert_eq!(module_and_path("start/pages/index.adoc"), None);
        assert_eq!(module_and_path("modules/ROOT/pages/"), None);
        assert_eq!(module_and_path("modules/ROOT/pages"), None);
        assert_eq!(module_and_path("modules/ROOT/pages/a//b.adoc"), None);
    }

    #[test]
    fn view_names_carry_the_profile_name() {
        for view in Antora.views() {
            assert!(view.name.starts_with("antora_"), "{}", view.name);
        }
    }
}
