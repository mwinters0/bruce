//! D16's classification of what a hop came to, with the reason that tells
//! two causes of one outcome apart.

use crate::fetch::{Body, Failure, FailureKind};
use crate::store::{Outcome, WRONG_TRAILING_SLASH};

/// An outcome and the `targets.reason` beside it.
pub type Verdict = (Outcome, Option<&'static str>);

/// What the last response of a target says. A 3xx here is one that led
/// nowhere (no `Location`, or not a web URL), which no browser could follow.
/// A 4xx that is broken has the reason [`WRONG_TRAILING_SLASH`] where
/// `slash_answers`: the URL with its final `/` toggled answered (D72).
pub fn answered(status: u16, body: &Body, slash_answers: bool) -> Verdict {
    match status {
        // The page answered: calling an over-cap one an error reads as a
        // false positive.
        200..=299 if *body == Body::TooLarge => (Outcome::Ok, Some("too_large")),
        200..=299 => (Outcome::Ok, None),
        300..=399 => (Outcome::BrokenUrl, Some("bad_redirect")),
        // Access is refused or limited, which says nothing of the page.
        401 | 403 | 429 => (Outcome::Error, None),
        400..=499 if slash_answers => (Outcome::BrokenUrl, Some(WRONG_TRAILING_SLASH)),
        400..=499 => (Outcome::BrokenUrl, None),
        _ => (Outcome::Error, None),
    }
}

/// A request that produced no response: a name that does not exist is a
/// broken URL, and every other cause may pass.
pub fn failed(failure: &Failure) -> Verdict {
    let outcome = match failure.kind {
        FailureKind::NxDomain => Outcome::BrokenUrl,
        _ => Outcome::Error,
    };
    (outcome, Some(failure.kind.as_str()))
}

#[cfg(test)]
mod tests;
