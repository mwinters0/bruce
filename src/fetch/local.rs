//! The local fetcher: a build directory answered as the URLs it will have in
//! production (D1, D22).
//!
//! It holds a map from URL to file and nothing about directories: which file
//! is which URL is `ci`'s concept, and `ci` hands the map over. A URL in the
//! map answers 200 with the file; one without a trailing `/` whose slashed
//! form is in the map answers 301 to it, as a static server does; any other
//! URL under the prefix answers 404, the build being the truth for what it
//! holds. A URL outside the prefix is a [`Failure`]: this fetcher has
//! nothing to say about it, and the routes that do (the baseline, then live,
//! D22) are the crawl's to open before it asks, and it asks this module's
//! [`Site`] which URLs are built. The query and the
//! fragment are not part of a file's name. Bodies are read as `fetch::http`
//! reads them: decoded HTML up to [`MAX_BODY`] for a page, bytes for a
//! sitemap, nothing for a status.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use url::Url;

use super::charset::decode;
use super::{Body, Failure, FailureKind, Fetcher, MAX_BODY, MAX_SITEMAP, Mode, Response};
use crate::url::canonical;

/// The URLs a build answers, and the file that holds each: what the fetcher
/// serves and the crawl's frontier asks about (D22).
#[derive(Debug)]
pub struct Site {
    prefix: Url,
    /// The canonical form of the prefix, which every URL answered starts with.
    key: String,
    files: HashMap<String, PathBuf>,
}

/// [`Fetcher`] over a build directory.
#[derive(Debug, Clone)]
pub struct LocalFetcher {
    site: Arc<Site>,
}

/// What the map says of a URL.
enum Lookup {
    File(PathBuf),
    /// The slashed form of the URL, as a relative `Location`.
    Moved(String),
    Missing,
    Elsewhere,
}

impl Site {
    /// The site answering `served`, each URL with the file that holds it,
    /// under `prefix`.
    pub fn new(prefix: &Url, served: impl IntoIterator<Item = (Url, PathBuf)>) -> Site {
        Site {
            prefix: prefix.clone(),
            key: key(prefix),
            files: served.into_iter().map(|(u, f)| (key(&u), f)).collect(),
        }
    }

    /// The production URL of the build directory's root.
    pub fn prefix(&self) -> &Url {
        &self.prefix
    }

    /// Whether `url` is below the prefix, built or not.
    pub fn under(&self, url: &Url) -> bool {
        key(url).starts_with(&self.key)
    }

    /// Whether the build answers `url` with a page or file, or moves it to
    /// one: what the build holds, as opposed to what it lacks.
    pub fn holds(&self, url: &Url) -> bool {
        matches!(self.lookup(url), Lookup::File(_) | Lookup::Moved(_))
    }

    fn lookup(&self, url: &Url) -> Lookup {
        let key = key(url);
        if let Some(file) = self.files.get(&key) {
            return Lookup::File(file.clone());
        }
        if !key.starts_with(&self.key) {
            return Lookup::Elsewhere;
        }
        if !key.ends_with('/') && self.files.contains_key(&format!("{key}/")) {
            let last = key.rsplit('/').next().unwrap_or_default();
            return Lookup::Moved(format!("{last}/"));
        }
        Lookup::Missing
    }
}

impl LocalFetcher {
    /// A fetcher answering `served`, each URL with the file that holds it,
    /// under `prefix`.
    pub fn new(prefix: &Url, served: impl IntoIterator<Item = (Url, PathBuf)>) -> LocalFetcher {
        LocalFetcher {
            site: Arc::new(Site::new(prefix, served)),
        }
    }

    /// The URLs this fetcher answers, to share with the crawl.
    pub fn site(&self) -> Arc<Site> {
        Arc::clone(&self.site)
    }
}

impl Fetcher for LocalFetcher {
    fn fetch(
        &self,
        url: &Url,
        mode: Mode,
    ) -> impl Future<Output = Result<Response, Failure>> + Send {
        let found = self.site.lookup(url);
        let asked = url.to_string();
        async move {
            let answer = |status, location, content_type, body| {
                Ok(Response {
                    status,
                    location,
                    content_type,
                    body,
                })
            };
            match found {
                Lookup::Elsewhere => Err(Failure {
                    kind: FailureKind::Other,
                    detail: format!("{asked} is not under the build's prefix"),
                }),
                Lookup::Missing => answer(404, None, None, Body::Unread),
                Lookup::Moved(to) => answer(301, Some(to), None, Body::Unread),
                Lookup::File(file) => {
                    let content_type = content_type(&file);
                    let html = content_type.starts_with("text/html");
                    let body = match mode {
                        Mode::Asset => Body::Unread,
                        Mode::Page if !html => Body::Unread,
                        Mode::Page | Mode::Sitemap => {
                            let bytes = read(&file).await?;
                            body_of(mode, bytes, content_type)
                        }
                    };
                    answer(200, None, Some(content_type.to_owned()), body)
                }
            }
        }
    }
}

/// The body of a 200 for `mode`, from the bytes of its file.
fn body_of(mode: Mode, bytes: Vec<u8>, content_type: &str) -> Body {
    match mode {
        Mode::Sitemap if bytes.len() > MAX_SITEMAP => Body::TooLarge,
        Mode::Sitemap => Body::Raw(bytes),
        _ if bytes.len() > MAX_BODY => Body::TooLarge,
        _ => Body::Html {
            size: bytes.len() as u64,
            text: decode(&bytes, Some(content_type)),
        },
    }
}

/// A file's bytes, read on the blocking pool.
async fn read(file: &Path) -> Result<Vec<u8>, Failure> {
    let path = file.to_owned();
    let failed = |detail: String| Failure {
        kind: FailureKind::Other,
        detail,
    };
    tokio::task::spawn_blocking(move || std::fs::read(&path).map_err(|e| (path, e)))
        .await
        .map_err(|e| failed(e.to_string()))?
        .map_err(|(path, e)| failed(format!("{}: {e}", path.display())))
}

/// The name a URL is looked up by: its canonical form without the query.
fn key(url: &Url) -> String {
    canonical(url, true).as_str().to_owned()
}

/// A file's `Content-Type`, from its extension. HTML carries no charset, so
/// the page's own `<meta>` decides, as it does behind a server that sends
/// none.
fn content_type(file: &Path) -> &'static str {
    let ext = file.extension().and_then(|e| e.to_str());
    match ext.map(str::to_ascii_lowercase).as_deref() {
        Some("html" | "htm") => "text/html",
        Some("css") => "text/css",
        Some("js" | "mjs") => "text/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("pdf") => "application/pdf",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests;
