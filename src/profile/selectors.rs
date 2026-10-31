//! Selectors compiled once per run (D11, D38). lol_html parses a selector's
//! text into a [`Selector`] and builds a rewriter's matching program from the
//! parsed ones; the parse is what this module keeps. A text is parsed the first time anything asks for it
//! and the result lives to the process's end, so a page's handlers borrow
//! what the first page parsed. The cache holds one entry per distinct text, which is bounded by
//! the compiled code's selectors and the config's, so it is leaked rather
//! than reference-counted.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use lol_html::errors::SelectorError;
use lol_html::html_content::{Element, TextChunk};
use lol_html::{ElementContentHandlers, HandlerResult, Selector};

static COMPILED: LazyLock<RwLock<HashMap<Box<str>, &'static Selector>>> =
    LazyLock::new(Default::default);

/// The selector `text` parses to, parsed on the first ask. A text lol_html
/// cannot parse or stream is not remembered and is refused every time.
pub fn compiled(text: &str) -> Result<&'static Selector, SelectorError> {
    if let Some(found) = COMPILED.read().unwrap_or_else(|e| e.into_inner()).get(text) {
        return Ok(found);
    }
    let parsed: &'static Selector = Box::leak(Box::new(text.parse::<Selector>()?));
    COMPILED
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .insert(text.into(), parsed);
    Ok(parsed)
}

/// [`compiled`] for a selector written in the code, which parses or is a bug.
fn written(text: &str) -> Cow<'static, Selector> {
    Cow::Borrowed(compiled(text).unwrap_or_else(|e| panic!("the selector {text:?}: {e}")))
}

/// `lol_html::element!` with the selector [`compiled`]: `handler` runs on each
/// element `selector` matches.
pub fn element<'h>(
    selector: &str,
    handler: impl FnMut(&mut Element<'_, '_>) -> HandlerResult + 'h,
) -> (Cow<'static, Selector>, ElementContentHandlers<'h>) {
    (
        written(selector),
        ElementContentHandlers::default().element(handler),
    )
}

/// `lol_html::text!` with the selector [`compiled`]: `handler` runs on each
/// chunk of text inside an element `selector` matches.
pub fn text<'h>(
    selector: &str,
    handler: impl FnMut(&mut TextChunk<'_>) -> HandlerResult + 'h,
) -> (Cow<'static, Selector>, ElementContentHandlers<'h>) {
    (
        written(selector),
        ElementContentHandlers::default().text(handler),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_parsed_once_and_asked_for_again() {
        let first = compiled("p.once-per-run > a[href]").unwrap();
        let second = compiled("p.once-per-run > a[href]").unwrap();
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn a_text_lol_html_cannot_parse_is_refused_each_time() {
        assert!(compiled("p:has(a)").is_err());
        assert!(compiled("p:has(a)").is_err());
    }

    #[test]
    fn the_handlers_borrow_the_compiled_selector() {
        let (selector, _) = element("div.once-per-run-too", |_| Ok(()));
        let Cow::Borrowed(borrowed) = selector else {
            panic!("the selector was parsed for the handler alone");
        };
        assert!(std::ptr::eq(
            borrowed,
            compiled("div.once-per-run-too").unwrap()
        ));
    }
}
