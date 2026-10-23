//! The content hash (D15): xxh3-128 over a scope of the page's stream, its
//! start tags with their attributes and its text, both decoded.
//!
//! The scope is a root less its exclusions ([`HashScope`]). With neither key
//! set the root is the profile's region; with only `hash_exclude` there is
//! none. A page where no root element matches is hashed whole, less the
//! exclusions, so a page with no region still hashes its content. Both
//! hashes are kept through the pass, since whether a root matched is known
//! only at its end.
//!
//! No ancestry reaches a handler, so scope is a pair of flags that earlier
//! handlers set on each element and text chunk and the last one reads and
//! clears, `lol_html` running one item's handlers in registration order.
//! End tags are not hashed: `lol_html` misses the implied ones, and text
//! nodes split where a tag falls, which keeps most moves of markup visible.

use std::borrow::Cow;
use std::cell::RefCell;

use lol_html::html_content::{Element, TextChunk};
use lol_html::{ElementContentHandlers, Selector};
use xxhash_rust::xxh3::Xxh3;

use crate::config::HashScope;
use crate::profile::ElementHandlers;

/// Which scope a page's hash covered (D15): what `hash_root` or the
/// profile's region matched, or the whole page where nothing did. A reader
/// comparing two hashes needs it, since the same page hashes differently
/// under each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Covered {
    /// An element `hash_root` names matched.
    Root,
    /// The profile's region matched, `hash_root` being unset.
    Region,
    /// No root element matched, or none was set: the page less its
    /// exclusions.
    Whole,
}

impl Covered {
    /// The value `pages.hash_scope` holds.
    pub fn as_str(self) -> &'static str {
        match self {
            Covered::Root => "root",
            Covered::Region => "region",
            Covered::Whole => "whole",
        }
    }
}

/// One page's two running hashes and the flags that route each item.
pub(super) struct Hash {
    /// Everything in scope in a root element, its own start tag included.
    root: Xxh3,
    /// Everything not excluded, the fallback where no root matched.
    page: Xxh3,
    root_seen: bool,
    /// What a matched root is, set by [`Hash::handlers`].
    rooted: Covered,
    in_root: bool,
    excluded: bool,
    /// The current text node's chunks, undecoded, until its last.
    text_root: String,
    text_page: String,
}

/// The root a page's hash is scoped to under `scope`, given the profile's
/// `region` (D15): `hash_root` if set, else the region if `hash_exclude`
/// is unset too, else none.
fn roots<'a>(scope: &'a HashScope, region: &[&'a str]) -> Vec<&'a str> {
    if !scope.root.is_empty() {
        scope.root.iter().map(String::as_str).collect()
    } else if scope.exclude.is_empty() {
        region.to_vec()
    } else {
        Vec::new()
    }
}

fn selector(text: &str) -> Cow<'static, Selector> {
    // The loader parsed each one (config::selectors), and a region is a
    // profile's, so a failure is a bug.
    Cow::Owned(
        text.parse()
            .unwrap_or_else(|e| panic!("the hash scope's selector {text:?}: {e}")),
    )
}

/// A field, length first, so no two sequences of fields hash alike.
fn field(h: &mut Xxh3, bytes: &[u8]) {
    h.update(&(bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

impl Default for Hash {
    fn default() -> Hash {
        Hash {
            root: Xxh3::new(),
            page: Xxh3::new(),
            root_seen: false,
            rooted: Covered::Whole,
            in_root: false,
            excluded: false,
            text_root: String::new(),
            text_page: String::new(),
        }
    }
}

impl Hash {
    /// The handlers for `scope` on a page whose profile's region is
    /// `region`. They read nothing another handler sets, so they may go
    /// anywhere in the pass, but in the order given.
    pub(super) fn handlers<'h>(
        this: &'h RefCell<Hash>,
        scope: &HashScope,
        region: &[&str],
    ) -> ElementHandlers<'h> {
        let mut handlers = ElementHandlers::new();
        let roots = roots(scope, region);
        this.borrow_mut().rooted = match () {
            _ if roots.is_empty() => Covered::Whole,
            _ if scope.root.is_empty() => Covered::Region,
            _ => Covered::Root,
        };
        for r in roots {
            handlers.push((
                selector(r),
                ElementContentHandlers::default()
                    .element(move |_: &mut Element<'_, '_>| {
                        let mut h = this.borrow_mut();
                        h.root_seen = true;
                        h.in_root = true;
                        Ok(())
                    })
                    .text(move |_: &mut TextChunk<'_>| {
                        this.borrow_mut().in_root = true;
                        Ok(())
                    }),
            ));
            handlers.push((
                selector(&format!("{r} *")),
                ElementContentHandlers::default().element(move |_: &mut Element<'_, '_>| {
                    this.borrow_mut().in_root = true;
                    Ok(())
                }),
            ));
        }
        for x in &scope.exclude {
            handlers.push((
                selector(x),
                ElementContentHandlers::default()
                    .element(move |_: &mut Element<'_, '_>| {
                        this.borrow_mut().excluded = true;
                        Ok(())
                    })
                    .text(move |_: &mut TextChunk<'_>| {
                        this.borrow_mut().excluded = true;
                        Ok(())
                    }),
            ));
            handlers.push((
                selector(&format!("{x} *")),
                ElementContentHandlers::default().element(move |_: &mut Element<'_, '_>| {
                    this.borrow_mut().excluded = true;
                    Ok(())
                }),
            ));
        }
        handlers.push((
            selector("*"),
            ElementContentHandlers::default()
                .element(move |el: &mut Element<'_, '_>| {
                    this.borrow_mut().element(el);
                    Ok(())
                })
                .text(move |chunk: &mut TextChunk<'_>| {
                    this.borrow_mut().text(chunk);
                    Ok(())
                }),
        ));
        handlers
    }

    fn element(&mut self, el: &Element) {
        let (in_root, excluded) = self.take_flags();
        if excluded {
            return;
        }
        let feed = |h: &mut Xxh3| {
            h.update(b"<");
            field(h, el.tag_name().as_bytes());
            for a in el.attributes() {
                field(h, a.name().as_bytes());
                field(h, htmlize::unescape_attribute(a.value()).as_bytes());
            }
        };
        feed(&mut self.page);
        if in_root {
            feed(&mut self.root);
        }
    }

    fn text(&mut self, chunk: &TextChunk) {
        let (in_root, excluded) = self.take_flags();
        if !excluded {
            self.text_page.push_str(chunk.as_str());
            if in_root {
                self.text_root.push_str(chunk.as_str());
            }
        }
        if chunk.last_in_text_node() {
            for (h, text) in [
                (&mut self.page, &mut self.text_page),
                (&mut self.root, &mut self.text_root),
            ] {
                if !text.is_empty() {
                    h.update(b"t");
                    field(h, htmlize::unescape(std::mem::take(text)).as_bytes());
                }
            }
        }
    }

    fn take_flags(&mut self) -> (bool, bool) {
        (
            std::mem::take(&mut self.in_root),
            std::mem::take(&mut self.excluded),
        )
    }

    /// The page's hash and the scope it covered, once the page has ended.
    pub(super) fn finish(self) -> (u128, Covered) {
        if self.root_seen {
            (self.root.digest128(), self.rooted)
        } else {
            (self.page.digest128(), Covered::Whole)
        }
    }
}

#[cfg(test)]
mod tests;
