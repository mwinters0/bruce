//! The ask that tells a broken URL from a wrong final slash (D72): which URL
//! to ask for a target that answered 4xx, and whether it answers.

use url::Url;

use super::frontier::{Frontier, Plan};
use super::outcome;
use crate::extract::Kind;
use crate::fetch::{Access, Body, Fetcher, Mode, Stopper};
use crate::store::Outcome;
use crate::url::toggle_slash;

/// Whether a link to a URL planned `plan` is requested.
fn is_requested(plan: Plan) -> bool {
    matches!(plan, Plan::Page { .. } | Plan::Anchors | Plan::Status)
}

/// The URL to ask about for `url`, where a target's hops ended and which
/// answered `status`: its own with the final `/` toggled, where the 4xx makes
/// it `broken_url`, a link to it is requested, and so is a link to the toggled
/// URL. For a redirected target `url` is its landing, whose reason the rows of
/// every URL that lands there share, so they share its fix.
pub(super) fn toggled(frontier: &Frontier, url: &Url, plan: Plan, status: u16) -> Option<Url> {
    let broken = outcome::answered(status, &Body::Unread, false) == (Outcome::BrokenUrl, None);
    let toggled = toggle_slash(url).filter(|_| broken && is_requested(plan))?;
    is_requested(frontier.plan(&toggled, Kind::Anchor).0).then_some(toggled)
}

/// Whether `url`, planned `plan`, where a target's hops ended with `status`,
/// has a toggled URL (D72) that answers 2xx when asked for its status alone,
/// if its host's robots.txt allows it; a redirect is not an answer.
/// A stop that comes first leaves it unasked, which cuts nothing: the target
/// is whole without it.
pub(super) async fn ask<F: Fetcher>(
    fetcher: &F,
    frontier: &Frontier,
    stop: &Stopper,
    url: &Url,
    plan: Plan,
    status: u16,
) -> bool {
    let Some(toggled) = toggled(frontier, url, plan, status) else {
        return false;
    };
    if stop.is_stopped() || fetcher.access(&toggled).await != Access::Allowed {
        return false;
    }
    let asked = fetcher.fetch(&toggled, Mode::Asset).await;
    asked.is_ok_and(|response| (200..300).contains(&response.status))
}
