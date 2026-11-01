//! The `antora` profile: Antora's generator meta, nav container, content
//! region and xrefs (D19, D20). Resolve is the `resolve` child's.
//!
//! Extract records a page's coordinates as facts (D19, I8): `component` and
//! `version` from the nav container, verbatim; `module` and `rel_path` from
//! the source path's `modules/<module>/pages/<rel_path>` tail, or a bare
//! `pages/<rel_path>` as `ROOT`'s (I26), the source a rule's or the edit
//! URL's (D93). A page with no
//! nav container (a `without_menu` layout) takes its `component` from the first
//! directory under the site root the search script names (I13), and records
//! no `version`. A nav container carrying both attributes, as docs.fp.o's
//! does (I8), makes a component with no version fact a URL-derived one; one
//! lacking `data-component` (I12) gives no component at all. An unresolved xref is tagged with its raw refspec (I6, D20).
//!
//! It reads no language menu: the default UI renders none, and a site's own
//! is a declarative profile's `locale_links` (D67).

use std::cell::RefCell;
use std::rc::Rc;

use percent_encoding::percent_decode_str;
use url::Url;

use super::{BoxError, Extract, Lookup, Profile, Settled, Sink, SitemapNote, View, element};

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
    /// The page has a nav container, with or without a component.
    nav_container: bool,
    /// `data-site-root-path` on the search script (I13).
    site_root_path: Option<String>,
}

impl Page {
    /// Records what only the settled page can give: the component of a page
    /// with no nav container, from its URL, the module and `rel_path`
    /// from the source.
    fn finish(&self, settled: &Settled, sink: &mut Sink) {
        if !self.nav_container {
            let root = self.site_root_path.as_deref();
            if let Some(component) = root.and_then(|root| component_under(settled.url, root)) {
                sink.fact("component", component);
            }
        }
        if let Some((module, rel_path)) = settled.source.and_then(|s| module_and_path(&s.path)) {
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

/// `(module, rel_path)` from an edit path holding `modules/<module>/pages/<rel_path>`
/// somewhere below a start path, as Antora lays a repo out, or else
/// `pages/<rel_path>` below one, the `ROOT` module Antora 3.2 implies where a
/// content source has no `modules/` (I26).
///
/// A `modules/<module>/pages/` tail takes precedence, and the first `pages`
/// segment is the implicit one's, so a start path with a directory named
/// `pages` above the real one is misread: the path cannot say where the start
/// path ends.
fn module_and_path(path: &str) -> Option<(String, String)> {
    let segments: Vec<&str> = path.split('/').collect();
    let (module, rel_path) = match segments
        .windows(3)
        .position(|w| w[0] == "modules" && w[2] == "pages")
    {
        Some(at) => (decode(segments[at + 1]), &segments[at + 3..]),
        None => {
            let at = segments.iter().position(|s| *s == "pages")?;
            ("ROOT".to_string(), &segments[at + 1..])
        }
    };
    if rel_path.is_empty() || rel_path.iter().any(|s| s.is_empty()) {
        return None;
    }
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
        extract.handlers.push(element(".nav-container", move |el| {
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
        extract.handlers.push(element(
            "script#search-ui-script[data-site-root-path]",
            move |el| {
                let mut page = seen.borrow_mut();
                if page.site_root_path.is_none() {
                    page.site_root_path = el.get_attribute("data-site-root-path");
                }
                Ok(())
            },
        ));
        extract
            .handlers
            .push(element("a.xref.unresolved[href]", move |el| {
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

    fn sitemap_note(&self) -> Option<SitemapNote> {
        Some(SitemapNote {
            key: "antora.sitemap_from_build",
            text: "Antora writes a sitemap from what it publishes, so an entry that did not answer is a build or deploy fault.",
        })
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
        assert_eq!(
            module_and_path("start/pages/index.adoc"),
            pair("ROOT", "index.adoc")
        );
        assert_eq!(
            module_and_path("docs/assembler/pages/a/b.adoc"),
            pair("ROOT", "a/b.adoc")
        );
        assert_eq!(
            module_and_path("pages/index.adoc"),
            pair("ROOT", "index.adoc")
        );
        assert_eq!(
            module_and_path("pages/modules/user/pages/a.adoc"),
            pair("user", "a.adoc"),
            "modules/ takes precedence"
        );
        assert_eq!(
            module_and_path("start/pages/x/pages/a.adoc"),
            pair("ROOT", "x/pages/a.adoc"),
            "the first pages segment"
        );
        assert_eq!(module_and_path("start/index.adoc"), None);
        assert_eq!(module_and_path("start/pages/"), None);
        assert_eq!(module_and_path("start/pages"), None);
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
