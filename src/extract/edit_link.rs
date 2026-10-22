//! The page's edit link: the core's ranking of every candidate (D36).
//!
//! A candidate is an `<a href>` that earns a tier, highest first:
//!
//! 1. `rel=edit`;
//! 2. a `title` or link text reading as an edit phrase;
//! 3. a known class (I12).
//!
//! Inside a tier a URL that parses as a forge shape comes first, then document
//! order. A link in no tier is no candidate, so a plain repository link never
//! wins by being the only link to a forge. A profile may prefer a link over
//! the ranking, through the sink (D38), which no profile does today.

use std::cell::RefCell;

use lol_html::html_content::Element;
use lol_html::{element, text};
use url::Url;

use super::attribute;
use super::links::Links;
use crate::forge::EditLink;
use crate::profile::ElementHandlers;

/// The classes that mark an edit link in a theme that gives it none of
/// `rel=edit`, a `title` or a phrase (I12).
const CLASSES: [&str; 3] = [
    "edit-this-page",
    "td-page-meta--edit",
    "theme-edit-this-page",
];

/// How much of a link's text is kept, enough for any phrase: text is
/// appended a chunk at a time until it reaches this many bytes.
const TEXT_LIMIT: usize = 200;

/// One `<a href>` as the ranking will read it.
struct Candidate {
    /// Where its link sits in the page's links.
    link: usize,
    rel_edit: bool,
    class: bool,
    title: Option<String>,
    /// Its text, undecoded, appended a chunk at a time until [`TEXT_LIMIT`] bytes.
    text: String,
}

/// What the edit-link handlers collect over one pass.
#[derive(Default)]
pub(super) struct EditLinks {
    candidates: Vec<Candidate>,
}

impl EditLinks {
    /// The handlers: one noting each `<a href>` and one collecting its text.
    /// They go after the link reader, whose last link is the anchor's own.
    pub(super) fn handlers<'h>(
        this: &'h RefCell<EditLinks>,
        links: &'h RefCell<Links>,
    ) -> ElementHandlers<'h> {
        vec![
            element!("a[href]", move |el| {
                if let Some(link) = links.borrow().last_index() {
                    this.borrow_mut().candidates.push(candidate(el, link));
                }
                Ok(())
            }),
            text!("a[href]", move |chunk| {
                if let Some(last) = this.borrow_mut().candidates.last_mut()
                    && last.text.len() < TEXT_LIMIT
                {
                    last.text.push_str(chunk.as_str());
                }
                Ok(())
            }),
        ]
    }

    /// The page's edit link: `preferred`, an href resolved as any other, if
    /// the profile gave one that names a web page; else the best candidate.
    pub(super) fn choose(self, links: &Links, preferred: Option<&str>) -> Option<EditLink> {
        let page = |url: Url| matches!(url.scheme(), "http" | "https").then(|| EditLink::new(url));
        if let Some(link) = preferred.and_then(|raw| links.resolve(raw)).and_then(page) {
            return Some(link);
        }
        let found = links.found();
        self.candidates
            .iter()
            .enumerate()
            .filter_map(|(order, c)| {
                let tier = c.tier()?;
                let link = page(found[c.link].url.clone()?)?;
                Some((tier, link.source.is_some(), std::cmp::Reverse(order), link))
            })
            .max_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)))
            .map(|(.., link)| link)
    }
}

fn candidate(el: &Element, link: usize) -> Candidate {
    let tokens = |name: &str| -> Vec<String> {
        attribute(el, name)
            .unwrap_or_default()
            .split_ascii_whitespace()
            .map(str::to_ascii_lowercase)
            .collect()
    };
    Candidate {
        link,
        rel_edit: tokens("rel").iter().any(|t| t == "edit"),
        class: tokens("class")
            .iter()
            .any(|t| CLASSES.contains(&t.as_str())),
        title: attribute(el, "title"),
        text: String::new(),
    }
}

impl Candidate {
    /// The candidate's tier, 3 the best; `None` for a link that is no
    /// candidate.
    fn tier(&self) -> Option<u8> {
        if self.rel_edit {
            Some(3)
        } else if self.title.as_deref().is_some_and(is_edit_phrase)
            || is_edit_phrase(&htmlize::unescape(&self.text))
        {
            Some(2)
        } else if self.class {
            Some(1)
        } else {
            None
        }
    }
}

/// Whether `text` reads as an edit phrase: "edit this page", "edit page" or
/// "edit on …", in any case, whitespace collapsed and any icon, arrow or
/// punctuation around it ignored.
fn is_edit_phrase(text: &str) -> bool {
    let words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    matches!(
        words.as_slice(),
        ["edit", "this", "page"] | ["edit", "page"] | ["edit", "on", ..]
    )
}

#[cfg(test)]
mod tests;
