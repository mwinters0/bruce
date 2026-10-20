//! Fetchers: live HTTP and a local build directory, behind one trait (D1).
//!
//! A fetcher answers **one hop**: the response to one URL, never the chain a
//! redirect starts, because the crawler follows hops itself so each one
//! passes its host's rate limit and robots.txt (D7). It reads a body only
//! where the caller can use it: a page's 2xx `text/html` or
//! `application/xhtml+xml` response, decoded to a `String` (`charset`), or a
//! sitemap's 2xx body as bytes. Every other response is classified by its
//! status alone, so a link to a PDF costs one request and no download (D16). Classifying a status into an
//! outcome is the crawler's; what a fetcher owes it is a [`Response`], or a
//! [`Failure`] saying why there was none.
//!
//! A [`Stopper`] ends a crawl early (`roadmap-P3-crawler.md`, "An interrupted
//! run"). The crawler owns it and hands it to its fetcher, which sends
//! nothing more once it is stopped: no request, retry or fallback leaves,
//! and a request still waiting for its host's place or a backoff returns at
//! once as [`FailureKind::Stopped`]. A request already sent runs to its end.

use std::fmt;
use std::future::Future;
use std::sync::Arc;

use tokio::sync::watch;
use tokio::time::Instant;
use url::Url;

pub mod charset;
pub mod http;
pub mod local;

pub use http::host::Access;

/// The most a sitemap file may come to, uncompressed: the sitemap
/// protocol's own limit.
pub const MAX_SITEMAP: usize = 50 * 1024 * 1024;

/// The most an HTML body may come to, after decompression; a larger one is
/// [`Body::TooLarge`].
pub const MAX_BODY: usize = 16 * 1024 * 1024;

/// What the caller wants from a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A page: GET, and an HTML body read for the extractor.
    Page,
    /// Only whether it answers (D13): HEAD, asked again as a page's GET with
    /// its retries when it answers 4xx or 5xx or fails transiently, and no
    /// body read either way.
    Asset,
    /// A sitemap (D14): GET, and a 2xx body read as bytes whatever its type,
    /// a `.xml.gz` file being served under several.
    Sitemap,
}

/// What one hop answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    /// The `Location` header as written, relative or not; resolving it
    /// against the URL asked is the crawler's.
    pub location: Option<String>,
    /// The `Content-Type` header as written.
    pub content_type: Option<String>,
    pub body: Body,
}

/// A response's body, as far as it was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    /// Not read: a status-only mode, a non-2xx status, or a type the
    /// extractor cannot use.
    Unread,
    /// A 2xx HTML body, decoded.
    Html {
        text: String,
        /// The bytes, after decompression and before decoding.
        size: u64,
    },
    /// A 2xx body asked for as bytes ([`Mode::Sitemap`]), as the server sent
    /// it once any `Content-Encoding` is undone.
    Raw(Vec<u8>),
    /// A body over its cap. The page answered, so it is not an error; it is
    /// not extracted, half a page's anchors giving false verdicts.
    TooLarge,
}

/// Why a hop produced no [`Response`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    /// The error's own words, for the target's log line.
    pub detail: String,
}

/// The cause of a [`Failure`], in D16's terms: `NxDomain` is `broken_url`,
/// every other cause `error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// The name does not exist.
    NxDomain,
    /// The connect or the whole response ran out of time.
    Timeout,
    /// The handshake failed: a certificate, a protocol, or a server not
    /// speaking TLS.
    Tls,
    /// No connection was made, and not for any cause above: refused,
    /// unreachable, or a resolver that did not answer.
    Connect,
    /// A connection was made and then lost before the response was whole,
    /// or what came back was not HTTP.
    Reset,
    /// Anything else.
    Other,
    /// The crawl was stopped before the request was sent. The crawler
    /// writes nothing for it.
    Stopped,
}

impl FailureKind {
    /// The value for `targets.reason`.
    pub fn as_str(self) -> &'static str {
        match self {
            FailureKind::NxDomain => "nxdomain",
            FailureKind::Timeout => "timeout",
            FailureKind::Tls => "tls",
            FailureKind::Connect => "connect",
            FailureKind::Reset => "reset",
            FailureKind::Other => "other",
            FailureKind::Stopped => "stopped",
        }
    }

    /// Whether another attempt can answer differently: a timeout or a lost
    /// connection can, a name that does not exist or a certificate that does
    /// not verify cannot (D16).
    pub fn is_transient(self) -> bool {
        matches!(self, FailureKind::Timeout | FailureKind::Reset)
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for Failure {}

impl Failure {
    /// The failure of a request a [`Stopper`] kept from being sent.
    pub fn stopped() -> Failure {
        Failure {
            kind: FailureKind::Stopped,
            detail: "the crawl was stopped".to_owned(),
        }
    }
}

/// Ends a crawl early. Clones share one state, which records when the first
/// [`Stopper::stop`] came.
#[derive(Debug, Clone)]
pub struct Stopper(Arc<watch::Sender<Option<Instant>>>);

impl Default for Stopper {
    fn default() -> Self {
        Stopper(Arc::new(watch::Sender::new(None)))
    }
}

impl Stopper {
    /// Asks the crawl to stop; a second call changes nothing.
    pub fn stop(&self) {
        self.0.send_if_modified(|at| {
            let first = at.is_none();
            if first {
                *at = Some(Instant::now());
            }
            first
        });
    }

    /// Whether [`Stopper::stop`] has been called.
    pub fn is_stopped(&self) -> bool {
        self.0.borrow().is_some()
    }

    /// Waits for the stop, at once if it has come, and returns when it came.
    pub async fn stopped(&self) -> Instant {
        let mut seen = self.0.subscribe();
        let at = seen
            .wait_for(Option::is_some)
            .await
            .expect("the sender lives as long as self");
        at.expect("waited for")
    }
}

/// Something that answers a URL (D1). The futures are `Send` so a crawl can
/// spawn them.
pub trait Fetcher {
    fn fetch(
        &self,
        url: &Url,
        mode: Mode,
    ) -> impl Future<Output = Result<Response, Failure>> + Send;

    /// Whether `url` may be requested at all (D8), which the crawler asks
    /// before each hop. A fetcher with no robots.txt behind it allows
    /// everything.
    fn access(&self, _url: &Url) -> impl Future<Output = Access> + Send {
        async { Access::Allowed }
    }

    /// The sitemaps the robots.txt of `url`'s host names in `Sitemap:` lines,
    /// as written (D40). Empty for a fetcher with no robots.txt behind it,
    /// and for a host whose robots.txt is not read.
    fn sitemaps(&self, _url: &Url) -> impl Future<Output = Vec<String>> + Send {
        async { Vec::new() }
    }

    /// This fetcher, sending nothing more once `stop` is stopped. One that
    /// never waits, as a local build does not, may ignore it.
    fn with_stop(self, _stop: Stopper) -> Self
    where
        Self: Sized,
    {
        self
    }
}
