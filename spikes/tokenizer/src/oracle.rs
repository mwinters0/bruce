//! The reference: the workload over html5ever's tree (via scraper), which
//! builds the DOM a browser would, entities decoded and misnesting repaired.

use crate::{has_class, push_links, anchor_values, Raw, RawA, RawAnchor};
use scraper::{ElementRef, Html};

fn is_article_doc(e: &ElementRef) -> bool {
    e.value().name() == "article" && has_class(e.attr("class").unwrap_or(""), "doc")
}

fn is_main(e: &ElementRef) -> bool {
    e.value().name() == "main" || e.attr("role") == Some("main")
}

pub fn extract(html: &str) -> Raw {
    let doc = Html::parse_document(html);
    let mut raw = Raw::default();
    for node in doc.root_element().descendants() {
        let Some(e) = ElementRef::wrap(node) else { continue };
        let tag = e.value().name().to_ascii_lowercase();
        let get = |n: &str| e.attr(n).map(str::to_owned);
        let self_and_up = || std::iter::once(e).chain(e.ancestors().filter_map(ElementRef::wrap));
        let in_article_doc = self_and_up().any(|x| is_article_doc(&x));
        let in_main = self_and_up().any(|x| is_main(&x));
        raw.has_article_doc |= is_article_doc(&e);
        raw.has_main |= is_main(&e);
        if tag == "meta" && raw.generator.is_none() && e.attr("name") == Some("generator") {
            raw.generator = get("content");
        }
        if raw.nav.is_none() && has_class(e.attr("class").unwrap_or(""), "nav-container") {
            raw.nav = Some((get("data-component"), get("data-version")));
        }
        for (value, attr) in anchor_values(&tag, get("id"), get("name")) {
            raw.anchors.push(RawAnchor { value, tag: tag.clone(), attr, in_article_doc, in_main });
        }
        push_links(&mut raw.links, &tag, get);
        if tag == "a" {
            if let Some(href) = e.attr("href") {
                let parent_class = e
                    .parent()
                    .and_then(ElementRef::wrap)
                    .and_then(|p| p.attr("class"))
                    .unwrap_or("")
                    .to_owned();
                raw.a_elems.push(RawA {
                    href: href.to_owned(),
                    rel: get("rel").unwrap_or_default(),
                    class: get("class").unwrap_or_default(),
                    title: get("title").unwrap_or_default(),
                    parent_class,
                    text: e.text().collect(),
                });
            }
        }
    }
    raw
}
