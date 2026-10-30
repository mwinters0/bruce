//! The guide closing the document, "How to read this report": an entry for
//! each term a row uses that the legend does not explain, and the ⓘ beside
//! the term, linking to its entry. HTML only, prose for a reader (D64). The
//! section is open, so an ⓘ lands on its entry in every browser, one that
//! opens no closed `<details>` for a fragment included. The ⓘ is an `i` in a
//! ring the stylesheet draws, as it draws the severity dots, a font lacking
//! U+24D8 showing that as a box.

use std::fmt::Write as _;

use super::super::document::Judged;
use super::markup::w;
use crate::config::Reach;
use crate::store::{DOUBLED_SLASH, PEDANTIC_TRAILING_SLASH};

/// An entry of the guide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Entry {
    /// `doubled_slash`, and the links broken through it (D71).
    DoubledSlash,
    /// `pedantic_trailing_slash` (D70).
    Pedantic,
}

impl Entry {
    /// The entry explaining a link's reason, where the guide has one.
    fn of_reason(reason: Option<&str>) -> Option<Entry> {
        match reason {
            Some(DOUBLED_SLASH) => Some(Entry::DoubledSlash),
            Some(PEDANTIC_TRAILING_SLASH) => Some(Entry::Pedantic),
            _ => None,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Entry::DoubledSlash => "guide-doubled-slash",
            Entry::Pedantic => "guide-pedantic",
        }
    }

    fn question(self) -> &'static str {
        match self {
            Entry::DoubledSlash => {
                "What do &quot;broken through it&quot; and &quot;doubled slash&quot; mean?"
            }
            Entry::Pedantic => "What is a &quot;pedantic&quot; link?",
        }
    }

    /// Whether the run's policy judges the reason the entry explains, so that
    /// the guide has the entry (D78).
    fn judged(self, judged: Judged) -> bool {
        match self {
            Entry::DoubledSlash => judged.doubled_slash,
            Entry::Pedantic => judged.pedantic_trailing_slash,
        }
    }

    /// The ⓘ for text that states the entry's subject whatever the policy
    /// says, nothing where the guide drops the entry.
    pub(super) fn info_if(self, judged: Judged) -> String {
        if self.judged(judged) {
            self.info()
        } else {
            String::new()
        }
    }

    /// The ⓘ linking to the entry, its question its tooltip and what a screen
    /// reader reads.
    pub(super) fn info(self) -> String {
        format!(
            " <a class=\"info\" href=\"#{}\" title=\"{q}\" aria-label=\"{q}\">i</a>",
            self.id(),
            q = self.question()
        )
    }
}

/// The ⓘ for a link's reason, nothing where the guide has no entry for it.
pub(super) fn info_of(reason: Option<&str>) -> String {
    Entry::of_reason(reason)
        .map(Entry::info)
        .unwrap_or_default()
}

/// The guide, the document's last section: an entry for each reason the run's
/// policy judges, there being no link to explain it otherwise.
pub(super) fn guide(out: &mut String, judged: Judged) {
    out.push_str("<section id=\"guide\">\n<h2>How to read this report</h2>\n");
    for (entry, body) in [
        (Entry::DoubledSlash, doubled_slash_entry(judged.reach)),
        (Entry::Pedantic, PEDANTIC_ENTRY.to_owned()),
    ] {
        if !entry.judged(judged) {
            continue;
        }
        w!(
            out,
            "<h3 id=\"{}\">{}</h3>\n{body}",
            entry.id(),
            entry.question()
        );
    }
    out.push_str("</section>\n");
}

/// The doubled-slash entry, its last paragraph stating what the run's
/// `doubled_slash_reach` counts on the page that wrote the link (D80).
fn doubled_slash_entry(reach: Reach) -> String {
    let counted = match reach {
        Reach::Full => {
            "the links on B that the extra slash breaks, and those on every other page at \
             a doubled URL that B's relative links lead on to, are counted on page A as \
             \"broken through it\""
        }
        Reach::Immediate => {
            "the links on B that the extra slash breaks are counted on page A as \"broken \
             through it\", though not those of the pages B's relative links lead on to"
        }
        Reach::Off => {
            "the links on B that the extra slash breaks are counted nowhere, this report \
             being set to blame page A's link alone"
        }
    };
    format!(
        "<p>Sometimes page B is fine if you go there the right way, but \
    page A links to it in a way that adds an extra slash, e.g. \
    <code>example.com/foo//page_B/</code> or <code>example.com/foo/page_B//</code>, or links to a \
    URL that redirects to one like it.</p>\n\
    <p>If you navigate from A to B that way, page B will work, but all of its relative links \
    (<code>../../blah</code>) are broken because of the extra slash in the path. They are not \
    counted against page B, because there's nothing wrong there: the problem is in page A. So \
    A's link is listed as broken, with the reason <code>doubled_slash</code>, and {counted}.</p>\n"
    )
}

const PEDANTIC_ENTRY: &str = "<p>Humans generally write URLs like <code>example.com/foo</code> \
    but <em>technically</em> the correct form is <code>example.com/foo/</code>: notice the \
    trailing slash. These are actually two separate web pages as far as most web servers are \
    concerned, and if you visit the first one the web server will helpfully tell your browser, \
    \"Hey dummy, there's no page here, it's over at <code>example.com/foo/</code>\". So your \
    browser has to make a second request.</p>\n\
    <p>Should the \"wrong\" URL count as broken, considering that the reader eventually ends up \
    in the right place? It's debatable, but it is certainly lower priority than an actually \
    broken link (a 404, say). These are not marked as broken: they are under Housekeeping's \
    redirected links, marked as pedantic with a ring, after the other redirects.</p>\n";
