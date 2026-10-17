//! Anchors: what a browser would scroll to (D12), and where on the page each
//! sits (D34).
//!
//! The pass reads `[id], a[name]` in document order. Whether an anchor is in
//! the content region is a flag a handler on the region and its descendants
//! sets just before it (handlers on one element run in registration order, and
//! none sees an ancestor). Whether the page has a region at all is known only
//! when it ends, which is why placement is decided there.

use std::borrow::Cow;
use std::cell::RefCell;

use lol_html::html_content::Element;
use lol_html::{ElementContentHandlers, HandlerResult, Selector};

use crate::profile::ElementHandlers;

/// Where on its page an anchor sits (D34), as `anchors.placement` stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Inside the profile's content region.
    Content,
    /// On a page that has a content region, outside it.
    Chrome,
    /// On a page with no content region, where nothing can be said.
    Unknown,
}

impl Placement {
    /// The value the `anchors` table holds.
    pub fn as_str(self) -> &'static str {
        match self {
            Placement::Content => "content",
            Placement::Chrome => "chrome",
            Placement::Unknown => "unknown",
        }
    }
}

/// One fragment a link to the page may name (D12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    /// The decoded `id` or `name`.
    pub value: String,
    /// The element's lower-case tag name.
    pub tag: String,
    pub placement: Placement,
}

/// An anchor read before its placement is known.
struct Found {
    value: String,
    tag: String,
    in_region: bool,
}

/// What the anchor handlers collect over one pass.
#[derive(Default)]
pub(super) struct Anchors {
    found: Vec<Found>,
    has_region: bool,
    in_region: bool,
}

fn selector(text: &str) -> Selector {
    text.parse()
        .unwrap_or_else(|e| panic!("a profile declared the region selector {text:?}: {e}"))
}

fn with_element<'h>(
    text: &str,
    handler: impl FnMut(&mut Element<'_, '_>) -> HandlerResult + 'h,
) -> (Cow<'static, Selector>, ElementContentHandlers<'h>) {
    (
        Cow::Owned(selector(text)),
        ElementContentHandlers::default().element(handler),
    )
}

/// An attribute's decoded value.
pub(super) fn attribute(el: &Element, name: &str) -> Option<String> {
    el.get_attribute(name)
        .map(|v| htmlize::unescape_attribute(v).into_owned())
}

impl Anchors {
    /// The handlers for a profile whose content region is `region`
    /// (selectors from `Extract::region`). Panics on a selector lol_html
    /// cannot parse, which is a bug in the profile.
    pub(super) fn handlers<'h>(this: &'h RefCell<Anchors>, region: &[&str]) -> ElementHandlers<'h> {
        let mut handlers = ElementHandlers::new();
        let mut inside = Vec::new();
        for r in region {
            handlers.push(with_element(r, move |_| {
                this.borrow_mut().has_region = true;
                Ok(())
            }));
            inside.extend([
                format!("{r}[id]"),
                format!("{r} [id]"),
                format!("{r} a[name]"),
            ]);
        }
        if !inside.is_empty() {
            handlers.push(with_element(&inside.join(", "), move |_| {
                this.borrow_mut().in_region = true;
                Ok(())
            }));
        }
        handlers.push(with_element("[id], a[name]", move |el| {
            let mut state = this.borrow_mut();
            let in_region = std::mem::take(&mut state.in_region);
            let tag = el.tag_name();
            let id = attribute(el, "id").filter(|v| !v.is_empty());
            // `name` anchors only on `<a>`: elsewhere it would make
            // `#viewport` and `#generator` pass on every page (D12).
            let name = attribute(el, "name")
                .filter(|v| !v.is_empty() && tag == "a" && id.as_ref() != Some(v));
            for value in id.into_iter().chain(name) {
                state.found.push(Found {
                    value,
                    tag: tag.clone(),
                    in_region,
                });
            }
            Ok(())
        }));
        handlers
    }

    /// The anchors with their placements, once the page has ended.
    pub(super) fn finish(self) -> Vec<Anchor> {
        let has_region = self.has_region;
        self.found
            .into_iter()
            .map(|f| Anchor {
                value: f.value,
                tag: f.tag,
                placement: match (has_region, f.in_region) {
                    (false, _) => Placement::Unknown,
                    (true, true) => Placement::Content,
                    (true, false) => Placement::Chrome,
                },
            })
            .collect()
    }
}
