//! Housekeeping's list of the settings this run was judged by that are not
//! `bruce`'s defaults (D82), so a judgement the reader cannot otherwise see is
//! stated. The JSON holds the whole policy.

use std::fmt::Write as _;

use super::super::document::Digest;
use super::markup::{esc, w};
use crate::config::{Entry, Policy};

/// The recorded rows that are not among the defaults' rows, in the order
/// recorded.
pub(super) fn differing(d: &Digest) -> Vec<&Entry> {
    let defaults = Policy::default().entries();
    d.policy.iter().filter(|e| !defaults.contains(e)).collect()
}

/// The list: each setting as a file writes it, or that there is none.
pub(super) fn list(out: &mut String, d: &Digest) {
    out.push_str(
        "<p>The settings this run was judged by that differ from the defaults, as the \
         configuration file writes them. A report judges the run by the policy it recorded, \
         and a comparison judges both runs by the newer one's.</p>\n",
    );
    let differing = differing(d);
    if differing.is_empty() {
        out.push_str("<p class=\"muted\">None: every setting is at its default.</p>\n");
        return;
    }
    out.push_str("<ul>\n");
    for entry in differing {
        w!(out, "<li><code>{}</code></li>\n", esc(&entry.written_as()));
    }
    out.push_str("</ul>\n");
}
