//! Configuration: optional TOML, CLI overrides and the defaults a run needs no
//! file for (D5, D35, D40). Every key and its default is documented for users
//! in `docs/manual/config.md`, which changes with this file.
//!
//! A file is read into the `Raw*` shapes serde derives, unknown keys refused,
//! and then validated into [`Config`]: URLs parsed, globs compiled, hosts
//! normalised, numbers range-checked. Nothing past this module sees a string
//! it would have to check again.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use url::{Host, Url};

/// Requests a second to one host when no `Crawl-delay` governs it (D40).
pub const DEFAULT_PER_SECOND: f64 = 2.0;

/// The profile value that picks one per page by its generator meta (D35).
pub const AUTO_PROFILE: &str = "auto";

/// A validated configuration: what a file said, every key it left out at its
/// default.
#[derive(Debug, Clone)]
pub struct Config {
    /// The sites crawled fully. Empty only when the file named none, which
    /// leaves the URL on the command line to supply one (D40).
    pub roots: Vec<Root>,
    /// Hosts whose links are checked one hop deep and never crawled (D3),
    /// normalised as [`Host`] prints them. Empty by default.
    pub allowlist: Vec<String>,
    /// Request rates per host (D8).
    pub rate: Rate,
}

/// One site crawled fully (D5).
#[derive(Debug, Clone)]
pub struct Root {
    /// Where the crawl starts. Its path, up to the last `/`, is the scope
    /// when `include` is empty (D40).
    pub url: Url,
    /// URL globs a page must match to be in scope; empty means the root's
    /// path prefix.
    pub include: Patterns,
    /// URL globs that put a page out of scope whatever `include` says.
    pub exclude: Patterns,
    /// The sitemap to seed from. `None` discovers it: robots.txt `Sitemap:`
    /// lines, then `sitemap.xml` under the root, then at the host (D40).
    pub sitemap: Option<Url>,
    /// The profile the root's pages are extracted with (D35).
    pub profile: ProfileChoice,
    /// Whether the root's robots.txt is obeyed (D8). On by default (D40).
    pub robots: bool,
    /// Seconds between requests to the root's host, overriding robots.txt's
    /// `Crawl-delay` and the rate (D18). `None` by default.
    pub crawl_delay: Option<f64>,
}

/// A root's profile (D35). A name is checked against the profiles compiled
/// in by `cli`, which builds the registry: this module names none (D39).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileChoice {
    /// Pick one per page by its generator meta, falling back to `generic`.
    Auto,
    /// Use the named profile for every page of the root.
    Named(String),
}

/// Request rates (D8, D40).
#[derive(Debug, Clone)]
pub struct Rate {
    /// Requests a second to any host no override names.
    pub per_second: f64,
    /// Per-host overrides of `per_second`, keyed as [`Host`] prints them.
    pub hosts: BTreeMap<String, f64>,
}

/// A list of URL globs, kept as written and compiled once. `*` and `?` stop
/// at `/`, `**` crosses it, and a glob is matched against the whole URL (D44).
#[derive(Debug, Clone)]
pub struct Patterns {
    globs: Vec<String>,
    set: GlobSet,
}

/// Why a configuration could not be loaded.
#[derive(Debug)]
pub enum Error {
    /// The file could not be read.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The file is not TOML, or holds a key or type the schema refuses.
    Parse(toml::de::Error),
    /// A value parsed but is not usable; the message names the key.
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Read { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Parse(e) => write!(f, "{e}"),
            Error::Invalid(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Read { source, .. } => Some(source),
            Error::Parse(e) => Some(e),
            Error::Invalid(_) => None,
        }
    }
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

impl Config {
    /// The configuration of a run given no file: `url` is the only root, and
    /// every other key is at its default (D40).
    pub fn for_url(url: &str) -> Result<Config, Error> {
        Ok(Config {
            roots: vec![Root::new(url)?],
            allowlist: Vec::new(),
            rate: Rate::default(),
        })
    }

    /// Reads and validates the file at `path`.
    pub fn load(path: &Path) -> Result<Config, Error> {
        let text = std::fs::read_to_string(path).map_err(|source| Error::Read {
            path: path.to_owned(),
            source,
        })?;
        Config::parse(&text)
    }

    /// Validates a configuration given as TOML text.
    pub fn parse(text: &str) -> Result<Config, Error> {
        let raw: RawConfig = toml::from_str(text).map_err(Error::Parse)?;
        let mut roots: Vec<Root> = Vec::with_capacity(raw.root.len());
        for (i, r) in raw.root.into_iter().enumerate() {
            let root = Root::from_raw(r).map_err(|e| invalid(format!("root[{i}]: {e}")))?;
            if roots.iter().any(|seen| seen.url == root.url) {
                return Err(invalid(format!(
                    "root[{i}]: {} is already a root",
                    root.url
                )));
            }
            roots.push(root);
        }
        let allowlist = raw
            .allowlist
            .iter()
            .map(|h| host(h).map_err(|e| invalid(format!("allowlist: {e}"))))
            .collect::<Result<_, _>>()?;
        Ok(Config {
            roots,
            allowlist,
            rate: Rate::from_raw(raw.rate)?,
        })
    }
}

impl Root {
    /// A root at `url` with every other key at its default (D40).
    pub fn new(url: &str) -> Result<Root, Error> {
        Root::from_raw(RawRoot {
            url: url.to_owned(),
            ..RawRoot::default()
        })
    }

    fn from_raw(raw: RawRoot) -> Result<Root, Error> {
        let url = http_url(&raw.url).map_err(|e| invalid(format!("url: {e}")))?;
        if url.fragment().is_some() {
            return Err(invalid(format!("url: {url} has a fragment")));
        }
        let sitemap = match raw.sitemap {
            Some(s) => Some(http_url(&s).map_err(|e| invalid(format!("sitemap: {e}")))?),
            None => None,
        };
        let profile = match raw.profile.as_deref() {
            None | Some(AUTO_PROFILE) => ProfileChoice::Auto,
            Some("") => return Err(invalid("profile: empty")),
            Some(name) => ProfileChoice::Named(name.to_owned()),
        };
        if let Some(d) = raw.crawl_delay
            && !(d.is_finite() && d >= 0.0)
        {
            return Err(invalid(format!(
                "crawl_delay: {d} is not a number of seconds"
            )));
        }
        Ok(Root {
            url,
            include: Patterns::new(raw.include).map_err(|e| invalid(format!("include: {e}")))?,
            exclude: Patterns::new(raw.exclude).map_err(|e| invalid(format!("exclude: {e}")))?,
            sitemap,
            profile,
            robots: raw.robots.unwrap_or(true),
            crawl_delay: raw.crawl_delay,
        })
    }

    /// The root's path prefix: its URL without query, up to the last `/` of
    /// its path.
    pub fn prefix(&self) -> &str {
        let s = &self.url[..url::Position::AfterPath];
        &s[..=s.rfind('/').expect("an http URL's path starts with /")]
    }

    /// Whether `url` is in the root's scope: under its prefix, or matching
    /// an `include` glob if there are any, and matching no `exclude` glob
    /// (D44).
    pub fn in_scope(&self, url: &Url) -> bool {
        let s = url.as_str();
        let included = if self.include.is_empty() {
            s.starts_with(self.prefix())
        } else {
            self.include.is_match(s)
        };
        included && !self.exclude.is_match(s)
    }
}

impl Default for Rate {
    fn default() -> Rate {
        Rate {
            per_second: DEFAULT_PER_SECOND,
            hosts: BTreeMap::new(),
        }
    }
}

impl Rate {
    fn from_raw(raw: RawRate) -> Result<Rate, Error> {
        let per_second = raw.per_second.unwrap_or(DEFAULT_PER_SECOND);
        positive(per_second).map_err(|e| invalid(format!("rate.per_second: {e}")))?;
        let mut hosts = BTreeMap::new();
        for (name, h) in raw.host {
            let key = host(&name).map_err(|e| invalid(format!("rate.host: {e}")))?;
            positive(h.per_second)
                .map_err(|e| invalid(format!("rate.host.\"{name}\".per_second: {e}")))?;
            if hosts.insert(key, h.per_second).is_some() {
                return Err(invalid(format!("rate.host: \"{name}\" is named twice")));
            }
        }
        Ok(Rate { per_second, hosts })
    }

    /// Requests a second to `host`: its override if it has one, else the
    /// default. `host` is compared as [`Host`] prints it.
    pub fn per_second_for(&self, host: &str) -> f64 {
        self.hosts.get(host).copied().unwrap_or(self.per_second)
    }
}

impl Patterns {
    fn new(globs: Vec<String>) -> Result<Patterns, String> {
        let mut set = GlobSetBuilder::new();
        for g in &globs {
            let glob = GlobBuilder::new(g)
                .literal_separator(true)
                .build()
                .map_err(|e| e.to_string())?;
            set.add(glob);
        }
        let set = set.build().map_err(|e| e.to_string())?;
        Ok(Patterns { globs, set })
    }

    /// The globs as the file wrote them.
    pub fn globs(&self) -> &[String] {
        &self.globs
    }

    pub fn is_empty(&self) -> bool {
        self.globs.is_empty()
    }

    pub fn is_match(&self, url: &str) -> bool {
        self.set.is_match(url)
    }
}

fn http_url(s: &str) -> Result<Url, String> {
    let url = Url::parse(s).map_err(|e| format!("{s}: {e}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(format!("{s}: scheme {other} is not http or https")),
    }
}

fn host(s: &str) -> Result<String, String> {
    if s.contains(['/', ':']) {
        return Err(format!("{s} is not a bare host"));
    }
    Host::parse(s)
        .map(|h| h.to_string())
        .map_err(|e| format!("{s}: {e}"))
}

fn positive(n: f64) -> Result<(), String> {
    if n.is_finite() && n > 0.0 {
        Ok(())
    } else {
        Err(format!("{n} is not a positive number"))
    }
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    root: Vec<RawRoot>,
    #[serde(default)]
    allowlist: Vec<String>,
    #[serde(default)]
    rate: RawRate,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawRoot {
    url: String,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    sitemap: Option<String>,
    profile: Option<String>,
    robots: Option<bool>,
    crawl_delay: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawRate {
    per_second: Option<f64>,
    #[serde(default)]
    host: BTreeMap<String, RawHostRate>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHostRate {
    per_second: f64,
}

#[cfg(test)]
mod tests;
