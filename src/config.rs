//! Configuration: optional TOML and the defaults a run needs no file for
//! (D5, D35, D40), no flag overriding a key. Every key and its default is
//! documented for users in `docs/manual/config.md`, which changes with this
//! file.
//!
//! A file is read into the `Raw*` shapes serde derives, unknown keys refused,
//! and then validated into [`Config`]: URLs parsed, globs compiled, hosts
//! normalised, numbers range-checked. Nothing past this module sees a string
//! it would have to check again, but a profile's name, which only `cli` can
//! check against the registry (D39).

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use url::{Host, Url};

use crate::url::{Canonical, canonical};

/// Requests a second to one host that no `Crawl-delay`, `[host]` or `[rate]`
/// governs (D40).
pub const DEFAULT_PER_SECOND: f64 = 2.0;

/// Requests in flight to one host at once, when neither `[host]` nor `[rate]`
/// says otherwise (D8). It binds only on a slow host: at the default rate and a response of
/// a third of a second, one request is in flight.
pub const DEFAULT_CONCURRENCY: u32 = 4;

/// Sent as `User-Agent` unless the file names one (D40).
pub const DEFAULT_USER_AGENT: &str = concat!("bruce/", env!("CARGO_PKG_VERSION"));

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
    /// URL globs, on any host, whose links are never asked about: the target
    /// is `unchecked`, reason `skipped` (D50). Empty by default.
    pub skip: Patterns,
    /// The rate no host setting or `Crawl-delay` overrides (D8).
    pub rate: Rate,
    /// Sent as `User-Agent`; its robots.txt token is [`Config::robots_token`].
    pub user_agent: String,
    /// Per-host settings, keyed as [`Host`] prints a host (D8). A host not
    /// here has [`HostSettings::default`].
    pub hosts: BTreeMap<String, HostSettings>,
    /// What robots.txt calls `user_agent`: its first product name, lowercased.
    token: String,
}

/// One site crawled fully (D5).
#[derive(Debug, Clone)]
pub struct Root {
    /// Where the crawl starts, and always in the root's own scope. Its
    /// canonical form without the query, up to the last `/` of its path, is
    /// the scope's prefix when `include` is empty (D40, D44).
    pub url: Url,
    /// URL globs a page must match to be in scope; empty means the root's
    /// path prefix.
    pub include: Patterns,
    /// URL globs that put a page out of scope whatever `include` says.
    pub exclude: Patterns,
    /// Whether the query is dropped from the canonical form of URLs in this
    /// root's scope (D7, D44), so `url_id` there depends on it.
    pub strip_query: bool,
    /// The sitemap to seed from. `None` discovers it: robots.txt `Sitemap:`
    /// lines, then `sitemap.xml` under the root, then at the host (D40).
    pub sitemap: Option<Url>,
    /// The profile the root's pages are extracted with (D35).
    pub profile: ProfileChoice,
    /// What part of each page its content hash covers (D15).
    pub hash: HashScope,
    /// The scope's prefix when `include` is empty, from the canonical form.
    prefix: String,
}

/// The part of a page its content hash covers (D15), from a root's
/// `hash_root` and `hash_exclude`. Each is a list of single selectors,
/// a list written inside one string split at its commas, and each one
/// parses as `lol_html` reads it. Empty is unset; what each combination
/// covers is `extract`'s to decide.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HashScope {
    /// The elements whose content is hashed.
    pub root: Vec<String>,
    /// The elements subtracted from the scope, content and all.
    pub exclude: Vec<String>,
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

/// The default request rate (D8, D40).
#[derive(Debug, Clone)]
pub struct Rate {
    /// Requests a second to a host with no `[host]` rate. It never beats a
    /// `Crawl-delay` (D18).
    pub per_second: f64,
    /// Requests in flight to a host with no `[host]` concurrency.
    pub concurrency: u32,
}

/// What a file says about fetching one host (D8). The crawler combines this
/// with the host's robots.txt (D18): where `per_second` is written and the
/// host is one a root starts on ([`Config::is_root_host`]), it beats
/// `Crawl-delay`; on any other host the slower of the two applies; with
/// `per_second` unwritten, `Crawl-delay` governs if there is one, else
/// [`Rate::per_second`]. With `robots` off the host's robots.txt, its
/// `Crawl-delay` included, is not read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostSettings {
    /// Requests a second, if the file gave one.
    pub per_second: Option<f64>,
    /// Requests in flight at once, if the file gave a number; else
    /// [`Rate::concurrency`].
    pub concurrency: Option<u32>,
    /// Whether the host's robots.txt is obeyed. On by default (D40).
    /// Only a host a root starts on can turn it off (D8).
    pub robots: bool,
}

impl Default for HostSettings {
    fn default() -> HostSettings {
        HostSettings {
            per_second: None,
            concurrency: None,
            robots: true,
        }
    }
}

/// A list of URL globs, kept as written and compiled once. `*` and `?` stop
/// at `/`, `**` crosses it, and a glob is matched against the whole URL (D44).
#[derive(Debug, Clone, Default)]
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
    /// The file is not TOML, holds a key or type the schema refuses, or
    /// lacks a key it requires.
    Parse(toml::de::Error),
    /// A value parsed but is not usable; the message names the key, or the
    /// root.
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
            skip: Patterns::default(),
            rate: Rate::default(),
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            hosts: BTreeMap::new(),
            token: robots_token(DEFAULT_USER_AGENT).expect("the default names a product"),
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
            // A root is named by its canonical URL, as `runs.roots` and
            // `pages.root` hold it: `/~x/` and `/%7Ex/` are one root.
            let canonical = root.canonical_url();
            if roots.iter().any(|seen| seen.canonical_url() == canonical) {
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
        let skip = Patterns::new(raw.skip).map_err(|e| invalid(format!("skip: {e}")))?;
        let user_agent = raw
            .user_agent
            .unwrap_or_else(|| DEFAULT_USER_AGENT.to_owned());
        let token = robots_token(&user_agent).map_err(|e| invalid(format!("user_agent: {e}")))?;
        let config = Config {
            roots,
            allowlist,
            skip,
            rate: Rate::from_raw(raw.rate)?,
            user_agent,
            hosts: host_settings(raw.host)?,
            token,
        };
        // A file naming no root leaves one to the command line, which may
        // start on the host, so the check waits for it.
        if !config.roots.is_empty() {
            config.check_robots_hosts()?;
            config.check_skipped_roots()?;
        }
        Ok(config)
    }

    /// The name robots.txt knows this crawler by: the first product name of
    /// `user_agent`, up to the first `/` or space, lowercased (`bruce` for `bruce/0.1`).
    pub fn robots_token(&self) -> &str {
        &self.token
    }

    /// Whether a root starts on `host`, compared as [`Host`] prints it.
    pub fn is_root_host(&self, host: &str) -> bool {
        self.roots
            .iter()
            .any(|r| r.url.host().is_some_and(|h| h.to_string() == host))
    }

    /// Refuses a root whose `url` a `skip` glob matches, a crawl that could
    /// not start. Run where [`Config::check_robots_hosts`] is.
    pub fn check_skipped_roots(&self) -> Result<(), Error> {
        for (i, r) in self.roots.iter().enumerate() {
            if self.skip.is_match(canonical(&r.url, false).as_str()) {
                return Err(invalid(format!(
                    "root[{i}]: url: {} is matched by skip",
                    r.url
                )));
            }
        }
        Ok(())
    }

    /// Refuses `robots = false` on a host no root starts on, since external
    /// hosts' robots.txt is always obeyed (D8). [`Config::parse`] runs it when
    /// the file names a root; whoever adds the root from the command line, to a
    /// file naming none, runs it then.
    pub fn check_robots_hosts(&self) -> Result<(), Error> {
        for (name, h) in &self.hosts {
            if !h.robots && !self.is_root_host(name) {
                return Err(invalid(format!(
                    "host.\"{name}\".robots: no root starts on {name}, and an external \
                     host's robots.txt is always obeyed"
                )));
            }
        }
        Ok(())
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
        let hash = HashScope {
            root: selectors(&raw.hash_root).map_err(|e| invalid(format!("hash_root: {e}")))?,
            exclude: selectors(&raw.hash_exclude)
                .map_err(|e| invalid(format!("hash_exclude: {e}")))?,
        };
        let prefix = {
            let s = canonical(&url, true);
            s.as_str()[..=s
                .as_str()
                .rfind('/')
                .expect("an http URL's path starts with /")]
                .to_owned()
        };
        let root = Root {
            url,
            include: Patterns::new(raw.include).map_err(|e| invalid(format!("include: {e}")))?,
            exclude: Patterns::new(raw.exclude).map_err(|e| invalid(format!("exclude: {e}")))?,
            strip_query: raw.strip_query,
            sitemap,
            profile,
            hash,
            prefix,
        };
        if !root.in_scope(&root.canonical(&root.url)) {
            return Err(invalid(format!(
                "url: {} is outside the root's own scope",
                root.url
            )));
        }
        Ok(root)
    }

    /// The root's path prefix: its canonical URL without query, up to the
    /// last `/` of its path.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// The root's own URL in its canonical form: what `runs.roots` and
    /// `pages.root` hold, and its own target's `url`.
    pub fn canonical_url(&self) -> String {
        self.canonical(&self.url).as_str().to_owned()
    }

    /// `url` in canonical form under this root's `strip_query` (D7).
    pub fn canonical(&self, url: &Url) -> Canonical {
        canonical(url, self.strip_query)
    }

    /// Whether `url` is in the root's scope: under its prefix, or matching
    /// an `include` glob if there are any, and matching no `exclude` glob.
    /// The globs and the prefix match the canonical form, so one written
    /// with a decoded character matches the URL a browser shows (D44).
    pub fn in_scope(&self, url: &Canonical) -> bool {
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
            concurrency: DEFAULT_CONCURRENCY,
        }
    }
}

impl Rate {
    fn from_raw(raw: RawRate) -> Result<Rate, Error> {
        let per_second = raw.per_second.unwrap_or(DEFAULT_PER_SECOND);
        positive(per_second).map_err(|e| invalid(format!("rate.per_second: {e}")))?;
        let concurrency = raw.concurrency.unwrap_or(DEFAULT_CONCURRENCY);
        at_least_one(concurrency).map_err(|e| invalid(format!("rate.concurrency: {e}")))?;
        Ok(Rate {
            per_second,
            concurrency,
        })
    }
}

fn host_settings(raw: BTreeMap<String, RawHost>) -> Result<BTreeMap<String, HostSettings>, Error> {
    let mut hosts = BTreeMap::new();
    for (name, h) in raw {
        let key = host(&name).map_err(|e| invalid(format!("host: {e}")))?;
        if let Some(n) = h.per_second {
            positive(n).map_err(|e| invalid(format!("host.\"{name}\".per_second: {e}")))?;
        }
        if let Some(n) = h.concurrency {
            at_least_one(n).map_err(|e| invalid(format!("host.\"{name}\".concurrency: {e}")))?;
        }
        let settings = HostSettings {
            per_second: h.per_second,
            concurrency: h.concurrency,
            robots: h.robots.unwrap_or(true),
        };
        if hosts.insert(key, settings).is_some() {
            return Err(invalid(format!("host: \"{name}\" is named twice")));
        }
    }
    Ok(hosts)
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

/// Each selector of `list` on its own, a string holding a selector list
/// split at its commas, and each checked against what `lol_html` parses:
/// a combinator or pseudo-class it does not stream (`+`, `~`, `:has`) is
/// refused with its reason (D33).
fn selectors(list: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for written in list {
        for part in split_list(written) {
            let part = part.trim();
            if part.is_empty() {
                return Err(format!("{written:?} holds an empty selector"));
            }
            part.parse::<lol_html::Selector>()
                .map_err(|e| format!("{part:?}: {e}"))?;
            out.push(part.to_owned());
        }
    }
    Ok(out)
}

/// `s` split at each comma outside quotes, brackets and parentheses, a
/// backslash escaping the character after it.
fn split_list(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut quote, mut escaped, mut start) = (0u32, None, false, 0);
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match (c, quote) {
            ('\\', _) => escaped = true,
            (q, Some(open)) if q == open => quote = None,
            (_, Some(_)) => {}
            ('"' | '\'', None) => quote = Some(c),
            ('[' | '(', None) => depth += 1,
            (']' | ')', None) => depth = depth.saturating_sub(1),
            (',', None) if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

fn http_url(s: &str) -> Result<Url, String> {
    let url = Url::parse(s).map_err(|e| format!("{s}: {e}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(format!("{s}: scheme {other} is not http or https")),
    }
}

/// `Host::parse` refuses a URL or `host:port` only as an IDNA error, so a `/`
/// or `:` is refused first, to say why.
fn host(s: &str) -> Result<String, String> {
    if s.contains(['/', ':']) {
        return Err(format!("{s} is not a bare host"));
    }
    Host::parse(s)
        .map(|h| h.to_string())
        .map_err(|e| format!("{s}: {e}"))
}

/// The token robots.txt names this crawler by, from a `User-Agent`: its first
/// product name, up to the `/` or the first space, lowercased. It is what a
/// site operator writes after `User-agent:`, so it must be one.
fn robots_token(user_agent: &str) -> Result<String, String> {
    if user_agent.is_empty() || !user_agent.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        return Err(format!(
            "{user_agent:?} is empty or holds a character a header cannot"
        ));
    }
    let name = user_agent
        .split(['/', ' '])
        .next()
        .expect("a split yields at least one piece");
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(format!(
            "{user_agent:?} does not start with a product name (letters, digits, - _ .)"
        ));
    }
    Ok(name.to_ascii_lowercase())
}

fn at_least_one(n: u32) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err("0 is not at least 1".to_owned())
    }
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
    skip: Vec<String>,
    #[serde(default)]
    rate: RawRate,
    user_agent: Option<String>,
    #[serde(default)]
    host: BTreeMap<String, RawHost>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawRoot {
    url: String,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    strip_query: bool,
    sitemap: Option<String>,
    profile: Option<String>,
    #[serde(default)]
    hash_root: Vec<String>,
    #[serde(default)]
    hash_exclude: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawRate {
    per_second: Option<f64>,
    concurrency: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHost {
    per_second: Option<f64>,
    concurrency: Option<u32>,
    robots: Option<bool>,
}

#[cfg(test)]
mod tests;
