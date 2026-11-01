//! Housekeeping's list of the settings this run was judged by that are not
//! `bruce`'s defaults (D91), so a judgement the reader cannot otherwise see is
//! stated, with how the run asked (D85), and in a comparison the list of the
//! rows that differ between its runs. The JSON holds the whole policy.

use std::fmt::Write as _;

use super::super::changes::Change;
use super::super::document::Digest;
use super::markup::{esc, w};
use crate::config::{Entry, Policy};

/// The recorded rows that are not among the defaults' rows, in the order
/// the digest holds them (`Entry::listed_order`).
pub(super) fn differing(d: &Digest) -> Vec<&Entry> {
    let defaults = Policy::default().entries();
    d.policy.iter().filter(|e| !defaults.contains(e)).collect()
}

/// The list: each setting as a file writes it, or that there is none.
pub(super) fn list(out: &mut String, d: &Digest) {
    if let Some(file) = &d.trial {
        w!(
            out,
            "<p><strong>A trial.</strong> This report was judged by the judging settings of \
             <code>{}</code>, in place of those the runs recorded; nothing was crawled again, \
             so the settings for asking are the run's own. The settings it was judged by and \
             asked with that differ from the defaults, as the configuration file writes \
             them:</p>\n",
            esc(file)
        );
    } else {
        out.push_str(
            "<p>The settings this run was judged by and asked with that differ from the \
             defaults, as the configuration file writes them. A report judges the run by the policy it recorded, \
             and a comparison judges both runs by the policy of the run it reports on.</p>\n",
        );
    }
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

/// The list of the rows that differ between the two runs compared (D91): a row
/// only one run recorded, and a row whose value, or place in a profile's list
/// of source rules, changed, each as the configuration file writes it.
pub(super) fn changes(out: &mut String, d: &Digest) {
    out.push_str(
        "<p>The rows of the policy the two runs recorded that differ, as the configuration \
         file writes them: <strong>\u{2212}</strong> a row only the compared run recorded, \
         <strong>+</strong> a row only this run recorded, and an arrow where a row changed. \
         A declared profile's rows are compared when a page of either run was extracted by it, \
         and a source rule is named by its place in each run's list.</p>\n",
    );
    if d.policy_changes.is_empty() {
        out.push_str("<p class=\"muted\">None: the runs recorded the same policy.</p>\n");
        return;
    }
    let (judging, rest): (Vec<&Change>, Vec<&Change>) =
        d.policy_changes.iter().partition(|c| c.judges());
    changed(out, &rest);
    if judging.is_empty() {
        return;
    }
    let by = d.trial.as_deref().map_or_else(
        || "this run's settings".to_owned(),
        |file| format!("the judging settings of <code>{}</code>", esc(file)),
    );
    w!(
        out,
        "<p>These judging settings differ too, but both runs were judged by {by}, so they \
         changed no verdict here:</p>\n"
    );
    changed(out, &judging);
}

/// One list of changes.
fn changed(out: &mut String, changes: &[&Change]) {
    if changes.is_empty() {
        return;
    }
    out.push_str("<ul>\n");
    for change in changes {
        let written = |e: &Option<Entry>| e.as_ref().map(|e| esc(&e.written_as()));
        match (written(&change.from), written(&change.to)) {
            (Some(was), Some(now)) => w!(
                out,
                "<li><code>{was}</code> \u{2192} <code>{now}</code></li>\n"
            ),
            (Some(was), None) => w!(out, "<li>\u{2212} <code>{was}</code></li>\n"),
            (None, Some(now)) => w!(out, "<li>+ <code>{now}</code></li>\n"),
            (None, None) => {}
        }
    }
    out.push_str("</ul>\n");
}
