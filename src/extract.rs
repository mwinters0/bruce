//! The extraction pass and its core heuristics: anchors, links, placement and
//! the edit link (D12, D13, D34, D36).
//!
//! [`extract`] is a pure function of one page: its body, its URL and the
//! root's profile choice in, an [`Extracted`] out. The generator meta is read
//! first, by a pass that stops at `<body>`, so the profile is chosen before
//! anything it extracts (D35). One `lol_html` rewriter then reads the page
//! with the core's handlers and the profile's (D11, D38). The caller decides
//! whether a response is extracted at all, and hands over decoded text.

mod anchors;
mod edit_link;
mod links;

use std::cell::RefCell;

use lol_html::errors::RewritingError;
use lol_html::{HtmlRewriter, Settings};
use lol_html::{element, text};
use url::Url;

use crate::config::ProfileChoice;
use crate::forge::EditLink;
use crate::profile::{ElementHandlers, Extract, Facts, Registry, Settled, Sink, UnknownProfile};

pub use anchors::{Anchor, Placement};
use anchors::{Anchors, attribute};
use edit_link::EditLinks;
use links::Links;
pub use links::{Kind, Link};

/// Everything the pass yields for one page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extracted {
    /// The name of the profile chosen for the page (D35).
    pub profile: &'static str,
    /// The first `<title>` outside an SVG, its whitespace collapsed; `None`
    /// where the page has none or it is empty.
    pub title: Option<String>,
    /// Each anchor in document order (D12).
    pub anchors: Vec<Anchor>,
    /// Each link in document order (D13).
    pub links: Vec<Link>,
    /// The page's edit link and what it parses to (D36); `None` where the
    /// page has none.
    pub edit_link: Option<EditLink>,
    /// The profile's facts about the page (D37).
    pub facts: Facts,
}

/// Why a page could not be extracted.
#[derive(Debug)]
pub enum Error {
    /// The root names a profile that is not compiled in.
    Profile(UnknownProfile),
    /// `lol_html` gave up on the page.
    Rewrite(RewritingError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Profile(e) => e.fmt(f),
            Error::Rewrite(e) => write!(f, "the page could not be read: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<UnknownProfile> for Error {
    fn from(e: UnknownProfile) -> Error {
        Error::Profile(e)
    }
}

impl From<RewritingError> for Error {
    fn from(e: RewritingError) -> Error {
        Error::Rewrite(e)
    }
}

/// Extracts `html`, the body of the page at `url`, under the profile
/// `choice` picks from `registry`.
pub fn extract(
    html: &str,
    url: &Url,
    choice: &ProfileChoice,
    registry: &Registry,
) -> Result<Extracted, Error> {
    let generator = match choice {
        ProfileChoice::Auto => generator(html)?,
        ProfileChoice::Named(_) => None,
    };
    let profile = registry.choose(choice, generator.as_deref())?;

    let sink = RefCell::new(Sink::new(profile.name()));
    let anchors = RefCell::new(Anchors::default());
    let title = RefCell::new(Title::default());
    let links = RefCell::new(Links::new(url));
    let edit_links = RefCell::new(EditLinks::default());
    let Extract {
        region,
        handlers,
        finish,
    } = profile.extract(&sink);

    let (read, tag) = Links::handlers(&links, &sink);
    let mut settings = Settings::new().with_strict(false);
    for handler in Title::handlers(&title)
        .into_iter()
        .chain(Anchors::handlers(&anchors, region))
        .chain(read)
        .chain(EditLinks::handlers(&edit_links, &links))
        .chain(handlers)
        .chain(tag)
    {
        settings = settings.append_element_content_handler(handler);
    }
    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    let mut sink = sink.into_inner();
    let links = links.into_inner();
    let edit_link = edit_links
        .into_inner()
        .choose(&links, sink.preferred_edit_link());
    if let Some(finish) = finish {
        let settled = Settled {
            url,
            edit_link: edit_link.as_ref(),
        };
        finish(&settled, &mut sink);
    }
    Ok(Extracted {
        profile: profile.name(),
        title: title.into_inner().text,
        anchors: anchors.into_inner().finish(),
        links: links.finish(),
        edit_link,
        facts: sink.into_facts(),
    })
}

/// The pre-pass stopped at `<body>`.
#[derive(Debug)]
struct Stop;

impl std::fmt::Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("stopped at <body>")
    }
}

impl std::error::Error for Stop {}

/// The content of the page's first `<meta name="generator">` (D35), read
/// from the head: the pass stops at `<body>`, past which no profile looks.
fn generator(html: &str) -> Result<Option<String>, Error> {
    let found = RefCell::new(None);
    let settings = Settings::new()
        .with_strict(false)
        .append_element_content_handler(element!("meta[name=generator i]", |el| {
            let mut found = found.borrow_mut();
            if found.is_none() {
                *found = attribute(el, "content");
            }
            Ok(())
        }))
        .append_element_content_handler(element!("body", |_| Err(Stop.into())));
    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
    match rewriter.write(html.as_bytes()) {
        Err(RewritingError::ContentHandlerError(e)) if e.is::<Stop>() => {}
        other => other?,
    }
    drop(rewriter);
    Ok(found.into_inner())
}

/// The first `<title>` outside an SVG. A title's text arrives in chunks,
/// undecoded, and the elements are counted so a later title's text is not
/// taken for the chosen one's.
#[derive(Default)]
struct Title {
    elements: u32,
    chosen: Option<u32>,
    in_svg: bool,
    pending: String,
    text: Option<String>,
}

impl Title {
    fn handlers<'h>(this: &'h RefCell<Title>) -> ElementHandlers<'h> {
        let mut handlers = Vec::new();
        for (selector, h) in [
            element!("svg title", |_| {
                this.borrow_mut().in_svg = true;
                Ok(())
            }),
            element!("title", |_| {
                let mut t = this.borrow_mut();
                t.elements += 1;
                if t.chosen.is_none() && !t.in_svg {
                    t.chosen = Some(t.elements);
                }
                t.in_svg = false;
                Ok(())
            }),
            text!("title", |chunk| {
                let mut t = this.borrow_mut();
                if t.chosen == Some(t.elements) {
                    t.pending.push_str(chunk.as_str());
                    if chunk.last_in_text_node() {
                        let raw = std::mem::take(&mut t.pending);
                        let decoded = htmlize::unescape(raw);
                        let collapsed = decoded.split_ascii_whitespace().collect::<Vec<_>>();
                        t.text = Some(collapsed.join(" ")).filter(|s| !s.is_empty());
                    }
                }
                Ok(())
            }),
        ] {
            handlers.push((selector, h));
        }
        handlers
    }
}

#[cfg(test)]
mod tests;
