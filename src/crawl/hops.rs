//! Redirect hops (D7): the chain of one target, its limit, and where a hop
//! leads.
//!
//! A hop is a 3xx with a `Location`, or a 2xx HTML page whose `refresh` has
//! delay 0 (`roadmap-P3-crawler.md`, "Redirects"). [`Chain`] records them
//! in the shape the `targets` row stores; the crawler decides what to ask
//! next and reads these for the rest.

use std::collections::HashSet;

use url::Url;

use crate::extract::{Kind, Link};
use crate::fetch::Response;
use crate::store::Hop;

/// The most hops a target may take (D7); one more is `broken_url`.
pub const MAX_HOPS: usize = 10;

/// Why a chain cannot go on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// A hop led to a URL the chain already held.
    Loop,
    /// The chain is [`MAX_HOPS`] long and the last response redirected again.
    TooMany,
}

impl Stop {
    /// The value for `targets.reason`.
    pub fn as_str(self) -> &'static str {
        match self {
            Stop::Loop => "redirect_loop",
            Stop::TooMany => "too_many_redirects",
        }
    }
}

/// The hops one target has taken, each URL in its canonical form so a loop
/// is a repeated string.
#[derive(Debug)]
pub struct Chain {
    hops: Vec<Hop>,
    visited: HashSet<String>,
    at: String,
}

impl Chain {
    /// A chain that starts at `first`.
    pub fn new(first: &str) -> Chain {
        Chain {
            hops: Vec::new(),
            visited: HashSet::from([first.to_owned()]),
            at: first.to_owned(),
        }
    }

    /// Records that the URL the chain is at answered `status` and led to
    /// `to`; `refresh` marks a page's own refresh rather than a 3xx. A loop
    /// keeps the hop that closed it, the hop past the limit is not kept.
    pub fn follow(&mut self, status: u16, refresh: bool, to: &str) -> Result<(), Stop> {
        if self.hops.len() >= MAX_HOPS {
            return Err(Stop::TooMany);
        }
        self.hops.push(Hop {
            url: std::mem::replace(&mut self.at, to.to_owned()),
            status,
            refresh,
        });
        if self.visited.insert(to.to_owned()) {
            Ok(())
        } else {
            Err(Stop::Loop)
        }
    }

    pub fn into_hops(self) -> Vec<Hop> {
        self.hops
    }
}

/// Where a 3xx `response` to the request for `from` leads: its `Location`
/// resolved against `from`, fragment dropped, if that is an `http` or
/// `https` URL. `None` for any other response, and for a 3xx that leads
/// nowhere usable.
pub fn location(from: &Url, response: &Response) -> Option<Url> {
    if !(300..=399).contains(&response.status) {
        return None;
    }
    web_url(from.join(response.location.as_deref()?).ok()?)
}

/// Where a page sends the browser at once: its first `refresh`,
/// if the delay is 0 and the URL is an `http` or `https` one. A page whose
/// first refresh waits is an ordinary page, whatever follows it.
pub fn refresh_target(links: &[Link]) -> Option<Url> {
    let first = links.iter().find(|l| l.kind == Kind::Refresh)?;
    if !first.immediate {
        return None;
    }
    web_url(first.url.clone()?)
}

fn web_url(mut url: Url) -> Option<Url> {
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    url.set_fragment(None);
    Some(url)
}

#[cfg(test)]
mod tests;
