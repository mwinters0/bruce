//! html5gum through its `Token` iterator: the easy API, one `BTreeMap` of
//! owned attributes per start tag. The tokenizer knows no tree, so this file
//! keeps the stack of open elements that region membership, a link's parent
//! and its text all need.

use crate::{anchor_values, has_class, push_links, Raw, RawA, RawAnchor};
use html5gum::{DefaultEmitter, Token, Tokenizer};

pub(crate) const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// One open element: what its descendants inherit.
pub(crate) struct Open {
    pub name: String,
    pub class: String,
    pub in_article_doc: bool,
    pub in_main: bool,
    pub foreign: bool,
    /// Its index in `Raw::a_elems`, for an `<a href>`.
    pub a_idx: Option<usize>,
}

/// The tree-shaped part both html5gum variants share: pushing and popping
/// open elements, and recording what a start tag carries.
#[derive(Default)]
pub(crate) struct Stack {
    pub open: Vec<Open>,
    pub open_a: Option<usize>,
}

impl Stack {
    /// Records a start tag, given its lowercased name and a lookup of its
    /// decoded attributes.
    pub fn start(&mut self, raw: &mut Raw, tag: &str, self_closing: bool, get: &dyn Fn(&str) -> Option<String>) {
        if tag == "a" {
            // The tree builder closes an open `<a>` before opening another.
            if let Some(i) = self.open.iter().rposition(|o| o.name == "a") {
                self.pop_to(i);
            }
        }
        let parent = self.open.last();
        let class = get("class").unwrap_or_default();
        let is_article_doc = tag == "article" && has_class(&class, "doc");
        let is_main = tag == "main" || get("role").as_deref() == Some("main");
        let in_article_doc = is_article_doc || parent.is_some_and(|p| p.in_article_doc);
        let in_main = is_main || parent.is_some_and(|p| p.in_main);
        let foreign = tag == "svg" || tag == "math" || parent.is_some_and(|p| p.foreign);
        raw.has_article_doc |= is_article_doc;
        raw.has_main |= is_main;
        if tag == "meta" && raw.generator.is_none() && get("name").as_deref() == Some("generator") {
            raw.generator = get("content");
        }
        if raw.nav.is_none() && has_class(&class, "nav-container") {
            raw.nav = Some((get("data-component"), get("data-version")));
        }
        for (value, attr) in anchor_values(tag, get("id"), get("name")) {
            raw.anchors.push(RawAnchor { value, tag: tag.to_owned(), attr, in_article_doc, in_main });
        }
        push_links(&mut raw.links, tag, get);
        let mut a_idx = None;
        if tag == "a" {
            if let Some(href) = get("href") {
                a_idx = Some(raw.a_elems.len());
                raw.a_elems.push(RawA {
                    href,
                    rel: get("rel").unwrap_or_default(),
                    class: class.clone(),
                    title: get("title").unwrap_or_default(),
                    parent_class: parent.map(|p| p.class.clone()).unwrap_or_default(),
                    text: String::new(),
                });
            }
        }
        if VOID.contains(&tag) || (self_closing && foreign) {
            return;
        }
        self.open.push(Open { name: tag.to_owned(), class, in_article_doc, in_main, foreign, a_idx });
        if a_idx.is_some() {
            self.open_a = a_idx;
        }
    }

    pub fn end(&mut self, tag: &str) {
        if let Some(i) = self.open.iter().rposition(|o| o.name == tag) {
            self.pop_to(i);
        }
    }

    fn pop_to(&mut self, i: usize) {
        self.open.truncate(i);
        self.open_a = self.open.iter().rev().find_map(|o| o.a_idx);
    }

    pub fn text(&self, raw: &mut Raw, s: &str) {
        if let Some(i) = self.open_a {
            raw.a_elems[i].text.push_str(s);
        }
    }
}

pub fn extract(html: &str) -> Raw {
    let mut emitter = DefaultEmitter::default();
    // Off by default: without it, `<script>` and `<style>` bodies tokenize as markup.
    emitter.naively_switch_states(true);
    let mut raw = Raw::default();
    let mut stack = Stack::default();
    for token in Tokenizer::new_with_emitter(html, emitter) {
        match token.unwrap() {
            Token::StartTag(t) => {
                let tag = String::from_utf8_lossy(&t.name);
                let get = |n: &str| {
                    t.attributes
                        .get(n.as_bytes())
                        .map(|v| String::from_utf8_lossy(&v.value).into_owned())
                };
                stack.start(&mut raw, &tag, t.self_closing, &get);
            }
            Token::EndTag(t) => stack.end(&String::from_utf8_lossy(&t.name)),
            Token::String(s) => stack.text(&mut raw, &String::from_utf8_lossy(&s.value)),
            _ => {}
        }
    }
    raw
}

/// Start tags, and nothing else: the tokenizer's own cost, for the bench.
pub fn count(html: &str) -> usize {
    let mut emitter = DefaultEmitter::default();
    emitter.naively_switch_states(true);
    Tokenizer::new_with_emitter(html, emitter)
        .filter(|t| matches!(t, Ok(Token::StartTag(_))))
        .count()
}
