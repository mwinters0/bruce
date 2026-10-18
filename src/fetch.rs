//! Fetchers: live HTTP and a local build directory, behind one trait (D1).
//!
//! A fetcher answers **one hop**: the response to one URL, never the chain a
//! redirect starts, because the crawler follows hops itself so each one
//! passes its host's rate limit and robots.txt (D7). It reads a body only
//! where the extractor can use it: a 2xx `text/html` or
//! `application/xhtml+xml` response, decoded to a `String` (`charset`).
//! Every other response is classified by its status alone, so a link to a
//! PDF costs one request and no download (D16). Classifying a status into an
//! outcome is the crawler's; what a fetcher owes it is a [`Response`], or a
//! [`Failure`] saying why there was none.

use std::fmt;
use std::future::Future;

use url::Url;

pub mod charset;
pub mod http;

pub use http::host::Access;

/// What the caller wants from a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A page: GET, and an HTML body read for the extractor.
    Page,
    /// Only whether it answers (D13): HEAD, retried once as a GET when it
    /// answers 4xx or 5xx, and no body read either way.
    Asset,
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
    /// An HTML body over the cap. The page answered, so it is not an error;
    /// it is not extracted, half a page's anchors giving false verdicts.
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
    /// A connection was made and then lost before the response was whole.
    Reset,
    /// Anything else, a malformed response included.
    Other,
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
}
