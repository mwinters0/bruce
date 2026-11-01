//! The digest's model: the part a rendering reads that is derived from the
//! document and holds no words. It is each link's finding under the severity
//! map and the counts and ranking that follow from it (`standing`, D76, D82,
//! D92), the broken targets a section lists (`targets`), the policy rows to
//! show (`policy`), a figure's change since the compared run and the rows of a
//! table found by column name (`row`) and, for a template, all of it as one
//! tree of values (`context`). The reasons the policy judges are the
//! document's `Judged`, which the model reads and cannot hold, being derived
//! from the `Digest` that holds it. Every value is an enum, a number, a
//! boolean or a string the run recorded, so a rendering in any language lays
//! them out and judges nothing. The JSON counts and orders by outcome, the
//! levels being derivable from its `policy`; the words stating a model's
//! values are the view's (`view`, D95).

mod context;
mod policy;
mod row;
mod standing;
mod targets;

use super::document::{Compared, Digest, Totals};

pub(in crate::report::digest) use context::context;
#[cfg(test)]
pub(in crate::report::digest) use context::counts_value;
pub(in crate::report::digest) use policy::{Changes, differing};
pub(in crate::report::digest) use row::{Row, rows};
#[cfg(test)]
pub(in crate::report::digest) use standing::Kind;
pub(in crate::report::digest) use standing::{Counts, KINDS, LEVELS, Page, listed, pages, repos};
pub(in crate::report::digest) use targets::targets;

/// The change of each figure since the compared run, in the order the digest
/// lists them (pages, links, links to a broken URL, to a missing fragment, that
/// could not be verified), or none where there was no comparison.
pub(in crate::report::digest) fn figure_changes(d: &Digest) -> Option<[i64; 5]> {
    let Compared { totals: then, .. } = d.compared.as_ref()?;
    let now: &Totals = &d.totals;
    Some([
        now.pages - then.pages,
        now.links - then.links,
        now.broken_url - then.broken_url,
        now.broken_anchor - then.broken_anchor,
        now.error - then.error,
    ])
}
