//! The broken targets "By: Link Target" lists: those more than one link names,
//! which the map makes a finding, and the counts the section states of the rest.

use super::super::document::Digest;
use super::row::{Row, rows};
use super::standing::listed;

/// The broken targets split by what the section does with each.
pub(in crate::report::digest) struct Targets<'a> {
    /// The findings named by more than one link, in the table's order.
    pub(in crate::report::digest) many: Vec<Row<'a>>,
    /// The findings named by one link each, which a page's drill-down lists.
    pub(in crate::report::digest) single: usize,
    /// A target the map makes no finding is named by more than one link.
    pub(in crate::report::digest) hidden_many: bool,
    /// The links `many` hold.
    pub(in crate::report::digest) held: i64,
    /// The broken links that are findings.
    pub(in crate::report::digest) all: i64,
}

pub(in crate::report::digest) fn targets(d: &Digest) -> Targets<'_> {
    let (shown, hidden): (Vec<Row>, Vec<Row>) = rows(&d.broken_targets)
        .partition(|r| listed(&d.severity, r.text("outcome"), r.text("reason")));
    let (many, single): (Vec<Row>, Vec<Row>) = shown.into_iter().partition(|r| r.int("links") > 1);
    Targets {
        held: many.iter().map(|r| r.int("links")).sum(),
        all: d.totals.broken_url + d.totals.broken_anchor
            - hidden.iter().map(|r| r.int("links")).sum::<i64>(),
        hidden_many: hidden.iter().any(|r| r.int("links") > 1),
        single: single.len(),
        many,
    }
}
