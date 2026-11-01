//! What the run's recorded policy makes of Housekeeping's settings (D91): the
//! rows off the defaults and the changes between two runs split by whether
//! they changed a verdict here.

use super::super::changes::Change;
use super::super::document::Digest;
use crate::config::{Entry, Policy};

/// The recorded rows that are not among the defaults' rows, in the order
/// the digest holds them (`Entry::listed_order`).
pub(in crate::report::digest) fn differing(d: &Digest) -> Vec<&Entry> {
    let defaults = Policy::default().entries();
    d.policy.iter().filter(|e| !defaults.contains(e)).collect()
}

/// The changes between the two runs compared (D91): those a comparison judges
/// by, which both runs being judged by one policy changed no verdict of, and the
/// rest.
pub(in crate::report::digest) struct Changes<'a> {
    pub(in crate::report::digest) judging: Vec<&'a Change>,
    pub(in crate::report::digest) rest: Vec<&'a Change>,
}

impl<'a> Changes<'a> {
    pub(in crate::report::digest) fn of(d: &'a Digest) -> Changes<'a> {
        let (judging, rest) = d.policy_changes.iter().partition(|c| c.judges());
        Changes { judging, rest }
    }
}
