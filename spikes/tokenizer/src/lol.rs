//! lol_html: the workload as CSS selectors. Region membership and a link's
//! parent are descendant and child combinators, so no stack is kept here; the
//! price is that lol_html hands attribute values and text over undecoded, and
//! text in chunks split anywhere, so this file decodes them itself.
//!
//! Several handlers matching one element run in registration order, which
//! the flag handlers below rely on: each sets a flag the last one reads.

use crate::{anchor_values, push_links, Raw, RawA, RawAnchor};
use lol_html::html_content::Element;
use lol_html::{element, text, HtmlRewriter, Settings};
use std::cell::RefCell;

#[derive(Default)]
struct State {
    raw: Raw,
    in_article_doc: bool,
    in_main: bool,
    parent_edit: bool,
    parent_etp: bool,
    /// Undecoded text of the current text node inside an `<a href>`.
    pending: String,
}

fn attr(el: &Element, name: &str) -> Option<String> {
    el.get_attribute(name).map(|v| htmlize::unescape_attribute(v).into_owned())
}

pub fn extract(html: &str) -> Raw {
    let st = RefCell::new(State::default());
    // Strict mode aborts the page on markup it cannot place without the tree
    // (`<select><xmp>`); read-only, a guess there costs less than the page.
    let settings = Settings::new()
        .with_strict(false)
        .append_element_content_handler(element!("meta[name=generator]", |el| {
            let mut s = st.borrow_mut();
            if s.raw.generator.is_none() {
                s.raw.generator = attr(el, "content");
            }
            Ok(())
        }))
        .append_element_content_handler(element!(".nav-container", |el| {
            let mut s = st.borrow_mut();
            if s.raw.nav.is_none() {
                s.raw.nav = Some((attr(el, "data-component"), attr(el, "data-version")));
            }
            Ok(())
        }))
        .append_element_content_handler(element!("article.doc", |_| {
            st.borrow_mut().raw.has_article_doc = true;
            Ok(())
        }))
        .append_element_content_handler(element!("main, [role=main]", |_| {
            st.borrow_mut().raw.has_main = true;
            Ok(())
        }))
        .append_element_content_handler(element!(
            "article.doc[id], article.doc [id], article.doc a[name]",
            |_| {
                st.borrow_mut().in_article_doc = true;
                Ok(())
            }
        ))
        .append_element_content_handler(element!(
            "main[id], main [id], main a[name], [role=main][id], [role=main] [id], [role=main] a[name]",
            |_| {
                st.borrow_mut().in_main = true;
                Ok(())
            }
        ))
        .append_element_content_handler(element!("[id], a[name]", |el| {
            let mut s = st.borrow_mut();
            let tag = el.tag_name();
            let (in_article_doc, in_main) = (s.in_article_doc, s.in_main);
            for (value, attr) in anchor_values(&tag, attr(el, "id"), attr(el, "name")) {
                s.raw.anchors.push(RawAnchor { value, tag: tag.clone(), attr, in_article_doc, in_main });
            }
            s.in_article_doc = false;
            s.in_main = false;
            Ok(())
        }))
        .append_element_content_handler(element!(
            "a[href], link[href], img, source, script[src], iframe[src]",
            |el| {
                let tag = el.tag_name();
                push_links(&mut st.borrow_mut().raw.links, &tag, |n| attr(el, n));
                Ok(())
            }
        ))
        .append_element_content_handler(element!("[class*=edit] > a[href]", |_| {
            st.borrow_mut().parent_edit = true;
            Ok(())
        }))
        .append_element_content_handler(element!(".edit-this-page > a[href]", |_| {
            st.borrow_mut().parent_etp = true;
            Ok(())
        }))
        .append_element_content_handler(element!("a[href]", |el| {
            let mut s = st.borrow_mut();
            // No parent access: the two flags stand in for its class.
            let parent_class = match (s.parent_etp, s.parent_edit) {
                (true, _) => "edit-this-page",
                (false, true) => "edit",
                _ => "",
            };
            let a = RawA {
                href: attr(el, "href").unwrap_or_default(),
                rel: attr(el, "rel").unwrap_or_default(),
                class: attr(el, "class").unwrap_or_default(),
                title: attr(el, "title").unwrap_or_default(),
                parent_class: parent_class.to_owned(),
                text: String::new(),
            };
            s.raw.a_elems.push(a);
            s.parent_edit = false;
            s.parent_etp = false;
            Ok(())
        }))
        .append_element_content_handler(text!("a[href]", |t| {
            let mut s = st.borrow_mut();
            s.pending.push_str(t.as_str());
            if t.last_in_text_node() {
                let decoded = htmlize::unescape(std::mem::take(&mut s.pending)).into_owned();
                if let Some(a) = s.raw.a_elems.last_mut() {
                    a.text.push_str(&decoded);
                }
            }
            Ok(())
        }));
    let mut rw = HtmlRewriter::new(settings, |_: &[u8]| {});
    rw.write(html.as_bytes()).unwrap();
    rw.end().unwrap();
    st.into_inner().raw
}

/// Start tags, and nothing else: the tokenizer's own cost, for the bench.
/// A `*` handler, as lol_html with no handler barely parses.
pub fn count(html: &str) -> usize {
    let n = std::cell::Cell::new(0);
    let settings = Settings::new().with_strict(false).append_element_content_handler(element!("*", |_| {
        n.set(n.get() + 1);
        Ok(())
    }));
    let mut rw = HtmlRewriter::new(settings, |_: &[u8]| {});
    rw.write(html.as_bytes()).unwrap();
    rw.end().unwrap();
    n.get()
}
