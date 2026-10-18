//! The HTTP fetcher: one hop over reqwest (D1, D11), with `roadmap-P3-crawler.md`'s
//! "Retries, timeouts and limits", "Which responses are read" and "Assets
//! and allowlisted pages" for its rules.
//!
//! reqwest's own redirect-following is off, so a 3xx is returned as an
//! answer with its `Location` (D7). TLS is rustls on ring and DNS is
//! hickory's, so a name that does not exist is a typed error (D45). A 5xx, a 429, a timeout or a lost
//! connection is retried after each [`Settings::backoff`] in turn; a 429 or
//! 503 naming a `Retry-After` waits that long instead, up to
//! [`Settings::max_retry_after`], and a longer one is returned unretried for
//! the crawler to settle `error` (D16). A fetcher that runs out of retries
//! returns the last answer, a status, or the last [`Failure`].

use std::error::Error as _;
use std::io;
use std::sync::Once;
use std::time::{Duration, SystemTime};

use hickory_resolver::net::NetError;
use reqwest::header::{CONTENT_TYPE, LOCATION, RETRY_AFTER};
use reqwest::{Client, Method, redirect};
use url::Url;

use super::charset::decode;
use super::{Body, Failure, FailureKind, Fetcher, Mode, Response};

/// The limits a fetcher holds itself to. The defaults are the spec's; they
/// are fields so a test can shrink them, not keys a deployment sets.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Sent as `User-Agent`.
    pub user_agent: String,
    pub connect_timeout: Duration,
    /// For the whole response, body included.
    pub timeout: Duration,
    /// The most an HTML body may come to, after decompression.
    pub max_body: usize,
    /// The wait before each retry, one retry per entry.
    pub backoff: Vec<Duration>,
    /// The longest `Retry-After` honoured.
    pub max_retry_after: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            user_agent: concat!("bruce/", env!("CARGO_PKG_VERSION")).to_owned(),
            connect_timeout: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
            max_body: 16 * 1024 * 1024,
            backoff: [1, 2, 4].map(Duration::from_secs).to_vec(),
            max_retry_after: Duration::from_secs(60),
        }
    }
}

/// [`Fetcher`] over HTTP. Cloning shares the connection pool.
#[derive(Debug, Clone)]
pub struct HttpFetcher {
    client: Client,
    settings: Settings,
}

impl HttpFetcher {
    pub fn new(settings: Settings) -> Result<HttpFetcher, Failure> {
        // reqwest is built without a crypto provider (D45); ring is installed
        // for the process, and a second install finding one is fine.
        static PROVIDER: Once = Once::new();
        PROVIDER.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
        let client = Client::builder()
            .user_agent(&settings.user_agent)
            .connect_timeout(settings.connect_timeout)
            .timeout(settings.timeout)
            .redirect(redirect::Policy::none())
            .hickory_dns(true)
            .build()
            .map_err(|e| classify(&e))?;
        Ok(HttpFetcher { client, settings })
    }

    /// One request, no retry. The answer comes with the wait its
    /// `Retry-After` asks for, where it has one.
    async fn once(
        &self,
        method: Method,
        url: &Url,
        read_body: bool,
    ) -> Result<(Response, Option<Duration>), Failure> {
        let mut response = self
            .client
            .request(method, url.clone())
            .send()
            .await
            .map_err(|e| classify(&e))?;
        let status = response.status();
        let header = |name| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let location = header(LOCATION);
        let content_type = header(CONTENT_TYPE);
        let retry_after = header(RETRY_AFTER).and_then(|v| retry_after(&v, SystemTime::now()));
        let mut body = Body::Unread;
        if read_body && status.is_success() && is_html(content_type.as_deref()) {
            let mut bytes = Vec::new();
            body = loop {
                match response.chunk().await.map_err(|e| classify(&e))? {
                    Some(chunk) if bytes.len() + chunk.len() > self.settings.max_body => {
                        break Body::TooLarge;
                    }
                    Some(chunk) => bytes.extend_from_slice(&chunk),
                    None => {
                        break Body::Html {
                            text: decode(&bytes, content_type.as_deref()),
                            size: bytes.len() as u64,
                        };
                    }
                }
            };
        }
        let response = Response {
            status: status.as_u16(),
            location,
            content_type,
            body,
        };
        Ok((response, retry_after))
    }

    /// A GET, retried as the module doc says.
    async fn get(&self, url: &Url, read_body: bool) -> Result<Response, Failure> {
        let mut backoff = self.settings.backoff.iter().copied();
        loop {
            let result = self.once(Method::GET, url, read_body).await;
            let Some(asked) = retry_plan(&result, self.settings.max_retry_after) else {
                return result.map(|(response, _)| response);
            };
            let Some(step) = backoff.next() else {
                return result.map(|(response, _)| response);
            };
            tokio::time::sleep(asked.unwrap_or(step)).await;
        }
    }
}

impl Fetcher for HttpFetcher {
    async fn fetch(&self, url: &Url, mode: Mode) -> Result<Response, Failure> {
        match mode {
            Mode::Page => self.get(url, true).await,
            Mode::Asset => match self.once(Method::HEAD, url, false).await {
                Ok((response, _)) if response.status < 400 => Ok(response),
                // A host refusing HEAD with 403 or 405 is common, and a false
                // `broken_url` costs more than one more request (D13).
                Ok(_) => self.get(url, false).await,
                Err(failure) if failure.kind.is_transient() => self.get(url, false).await,
                Err(failure) => Err(failure),
            },
        }
    }
}

/// Whether to retry an answer, and after how long: `None` for no retry,
/// `Some(None)` for the next backoff step, `Some(Some(wait))` for a wait the
/// server asked for.
fn retry_plan(
    result: &Result<(Response, Option<Duration>), Failure>,
    max_retry_after: Duration,
) -> Option<Option<Duration>> {
    match result {
        Ok((response, asked)) if response.status >= 500 || response.status == 429 => {
            match asked {
                // `Retry-After` is read on a 429 or 503 only; a longer one
                // settles the target at once (D16).
                Some(wait) if matches!(response.status, 429 | 503) => {
                    (*wait <= max_retry_after).then_some(Some(*wait))
                }
                _ => Some(None),
            }
        }
        Err(failure) if failure.kind.is_transient() => Some(None),
        _ => None,
    }
}

/// How long a `Retry-After` value asks for: seconds, or an HTTP date.
fn retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    let value = value.trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let at = httpdate::parse_http_date(value).ok()?;
    Some(at.duration_since(now).unwrap_or_default())
}

/// Whether a `Content-Type` is one whose body the extractor reads.
fn is_html(content_type: Option<&str>) -> bool {
    let Some(content_type) = content_type else {
        return false;
    };
    let media = content_type.split(';').next().unwrap_or("").trim();
    media.eq_ignore_ascii_case("text/html") || media.eq_ignore_ascii_case("application/xhtml+xml")
}

/// A transport error in D16's terms.
fn classify(error: &reqwest::Error) -> Failure {
    let kind = cause(error).unwrap_or(if error.is_timeout() {
        FailureKind::Timeout
    } else if error.is_connect() {
        FailureKind::Connect
    } else if error.is_body() || error.is_request() {
        FailureKind::Reset
    } else {
        FailureKind::Other
    });
    let mut detail = error.to_string();
    let mut source = error.source();
    while let Some(next) = source {
        // An error often prints its own source too; keep each phrase once.
        let words = next.to_string();
        if !detail.contains(&words) {
            detail.push_str(": ");
            detail.push_str(&words);
        }
        source = next.source();
    }
    Failure { kind, detail }
}

/// What the chain under `error` says the cause is, if it names one reqwest's
/// own flags do not. An `io::Error` is descended through `get_ref`: its
/// `source` skips the error it wraps, which is where a TLS failure sits.
fn cause(error: &(dyn std::error::Error + 'static)) -> Option<FailureKind> {
    let mut at = Some(error);
    while let Some(error) = at {
        if let Some(net) = error.downcast_ref::<NetError>()
            && net.is_nx_domain()
        {
            return Some(FailureKind::NxDomain);
        }
        if error.is::<rustls::Error>() {
            return Some(FailureKind::Tls);
        }
        if let Some(hyper) = error.downcast_ref::<hyper::Error>()
            && (hyper.is_incomplete_message() || hyper.is_closed() || hyper.is_canceled())
        {
            return Some(FailureKind::Reset);
        }
        if let Some(io) = error.downcast_ref::<io::Error>() {
            if let Some(found) = io.get_ref().and_then(|inner| cause(inner)) {
                return Some(found);
            }
            use io::ErrorKind::*;
            if matches!(
                io.kind(),
                ConnectionReset | ConnectionAborted | BrokenPipe | UnexpectedEof
            ) {
                return Some(FailureKind::Reset);
            }
        }
        at = error.source();
    }
    None
}

#[cfg(test)]
mod tests;
