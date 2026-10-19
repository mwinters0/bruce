//! The frontier: which URLs a run asks about, and how (D3, D7, D26).
//!
//! [`Frontier::plan`] sorts a URL into one of `roadmap-P3-crawler.md`'s
//! "What gets requested" classes and returns its canonical form, the string
//! its `url_id` hashes. A URL in a root's scope has that root's form; every
//! other URL has the plain form, `canonical(url, false)`, since no root's
//! `strip_query` governs it. [`Frontier::claim`] is the run's one
//! deduplication: a target is written by whoever claims its `url_id` first
//! (D13), and a claim is never given back.

use std::collections::HashSet;
use std::sync::Mutex;

use url::Url;

use crate::config::Config;
use crate::extract::Kind;
use crate::url::{Canonical, canonical};

/// The reason of a URL `skip` matched (D50).
pub const SKIPPED: &str = "skipped";

/// What a run does about a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// In root `root`'s scope: GET, an HTML response extracted and its links
    /// followed.
    Page { root: usize },
    /// On an allowlisted host and named by an `anchor` link: GET, extracted
    /// for its anchors only (D3).
    Anchors,
    /// On an allowlisted host and named by any other kind of link, or reached
    /// by a redirect where a link would leave it unchecked: asked for its
    /// status alone, HEAD with the GET fallback (D13).
    Status,
    /// Not requested; the target row says why.
    Unchecked(&'static str),
    /// A sitemap, `depth` files below where discovery began (1 for the file
    /// discovered or set): GET, its body read for its entries (D14). Only
    /// the crawl plans one; [`Frontier::plan`] never does.
    Sitemap { depth: u8 },
}

/// The URLs a run has met, and how each is treated.
pub struct Frontier {
    config: Config,
    seen: Mutex<HashSet<u64>>,
}

impl Frontier {
    pub fn new(config: Config) -> Frontier {
        Frontier {
            config,
            seen: Mutex::new(HashSet::new()),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Whether `url_id` is new to the run; true for the first caller only.
    pub fn claim(&self, url_id: u64) -> bool {
        self.seen
            .lock()
            .expect("no panic holds the lock")
            .insert(url_id)
    }

    /// How many URLs have been claimed.
    pub fn claimed(&self) -> u64 {
        self.seen.lock().expect("no panic holds the lock").len() as u64
    }

    /// Whether `url_id` has been claimed, claiming nothing.
    pub fn is_claimed(&self, url_id: u64) -> bool {
        self.seen
            .lock()
            .expect("no panic holds the lock")
            .contains(&url_id)
    }

    /// How `url` is treated when a link of `kind` names it, and its canonical
    /// form. The first root whose scope holds it is its root. Order is the
    /// spec's: `skip` (D50), matched on the plain form whatever the host,
    /// then a root's scope, then an allowlisted host, then the rest, an
    /// out-of-scope URL on a root's own host told apart from any other host
    /// by its reason.
    pub fn plan(&self, url: &Url, kind: Kind) -> (Plan, Canonical) {
        if !matches!(url.scheme(), "http" | "https") {
            return (Plan::Unchecked("scheme"), canonical(url, false));
        }
        let plain = canonical(url, false);
        if self.config.skip.is_match(plain.as_str()) {
            return (Plan::Unchecked(SKIPPED), plain);
        }
        for (root, r) in self.config.roots.iter().enumerate() {
            let form = r.canonical(url);
            if r.in_scope(&form) {
                return (Plan::Page { root }, form);
            }
        }
        let form = plain;
        let host = url.host().map(|h| h.to_string()).unwrap_or_default();
        let plan = if self.config.allowlist.contains(&host) {
            // deficiency: KD4 The first link to an allowlisted URL decides how
            // it is asked: an anchor link gets a GET and its anchors are
            // stored, any other kind gets a HEAD. A URL named first by an
            // image and later by an anchor link with a fragment is never
            // read, so that fragment has no anchors to be checked against.
            // Reading it again would write the target twice (D2), and
            // deciding after the whole crawl needs the frontier to hold every
            // link's kind.
            if kind == Kind::Anchor {
                Plan::Anchors
            } else {
                Plan::Status
            }
        } else if self.config.is_root_host(&host) {
            Plan::Unchecked("out_of_scope")
        } else {
            Plan::Unchecked("host_not_allowlisted")
        };
        (plan, form)
    }

    /// [`Frontier::plan`] for the URL a redirect leads to: a hop is followed
    /// wherever it goes, so what a link would leave unchecked is asked for its
    /// status alone. A skipped URL stays skipped, and ends the chain (D50).
    pub fn hop_plan(&self, url: &Url, kind: Kind) -> (Plan, Canonical) {
        match self.plan(url, kind) {
            (Plan::Unchecked(SKIPPED), form) => (Plan::Unchecked(SKIPPED), form),
            (Plan::Unchecked(_), form) => (Plan::Status, form),
            planned => planned,
        }
    }
}

#[cfg(test)]
mod tests;
