//! The fetcher of a CI run: the build directory answers every URL it holds,
//! and another fetcher every other URL (D1, D22).
//!
//! Which route a URL takes is the crawl's to decide before it asks
//! (`crawl`'s frontier); this only sends a request the way the build's
//! contents say, so a page the build holds never leaves the machine and
//! nothing else is read from the directory. A URL under the prefix that the
//! build lacks, and the baseline does not answer, is asked of production.
//! robots.txt and the rate are the live fetcher's and apply to its URLs
//! alone, but for `sitemaps`, which always asks the live host's robots.txt.

use std::future::Future;

use url::Url;

use super::local::LocalFetcher;
use super::{Access, Failure, Fetcher, Mode, Response, Stopper};

/// A [`LocalFetcher`] for the URLs its build holds and `live` for the rest.
#[derive(Debug, Clone)]
pub struct Routed<L> {
    local: LocalFetcher,
    live: L,
}

impl<L> Routed<L> {
    pub fn new(local: LocalFetcher, live: L) -> Routed<L> {
        Routed { local, live }
    }

    fn is_local(&self, url: &Url) -> bool {
        self.local.site().holds(url)
    }
}

impl<L: Fetcher + Sync> Fetcher for Routed<L> {
    fn fetch(
        &self,
        url: &Url,
        mode: Mode,
    ) -> impl Future<Output = Result<Response, Failure>> + Send {
        let local = self.is_local(url);
        async move {
            if local {
                self.local.fetch(url, mode).await
            } else {
                self.live.fetch(url, mode).await
            }
        }
    }

    fn access(&self, url: &Url) -> impl Future<Output = Access> + Send {
        let local = self.is_local(url);
        async move {
            if local {
                Access::Allowed
            } else {
                self.live.access(url).await
            }
        }
    }

    fn sitemaps(&self, url: &Url) -> impl Future<Output = Vec<String>> + Send {
        self.live.sitemaps(url)
    }

    fn with_stop(self, stop: Stopper) -> Self {
        Routed {
            local: self.local,
            live: self.live.with_stop(stop),
        }
    }
}

#[cfg(test)]
mod tests;
