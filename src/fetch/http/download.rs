//! Downloading one file whole, for a baseline given as a URL (D21).
//!
//! It is one request outside the crawl's pacing and robots.txt: the file is
//! the deployment's own, named by whoever runs the check. Redirects are
//! followed here, a published file often being served from elsewhere, and the
//! body is written as it arrives.

use std::io::Write;
use std::time::Duration;

use url::Url;

use super::{Failure, FailureKind, HttpFetcher, classify};

/// Redirects followed to reach the file.
const REDIRECTS: usize = 5;

/// Each request's time, body included: the file is some megabytes.
const TIMEOUT: Duration = Duration::from_secs(600);

impl HttpFetcher {
    /// Writes the body of a 2xx answer for `url` to `out`.
    pub async fn download(&self, url: &Url, out: &mut (impl Write + Send)) -> Result<(), Failure> {
        let other = |detail: String| Failure {
            kind: FailureKind::Other,
            detail,
        };
        let mut at = url.clone();
        for _ in 0..=REDIRECTS {
            let mut response = self
                .client
                .get(at.clone())
                .timeout(TIMEOUT)
                .send()
                .await
                .map_err(|e| classify(&e))?;
            let status = response.status();
            if status.is_redirection() {
                let next = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|l| at.join(l).ok())
                    .filter(|u| matches!(u.scheme(), "http" | "https"));
                at = next.ok_or_else(|| other(format!("{at} redirected to nowhere")))?;
                continue;
            }
            if !status.is_success() {
                return Err(other(format!("{at} answered {status}")));
            }
            while let Some(chunk) = response.chunk().await.map_err(|e| classify(&e))? {
                out.write_all(&chunk).map_err(|e| other(e.to_string()))?;
            }
            return Ok(());
        }
        Err(other(format!(
            "{url} redirected more than {REDIRECTS} times"
        )))
    }
}
