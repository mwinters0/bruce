//! The rows the writer appends, one type per table, in the table's column
//! order (D26 as D34–D37, D43 and D51 amend it). A row holds what the table holds and
//! nothing derived: the crawler builds them, and the store knows no
//! extraction type, so a link's kind and an anchor's placement arrive as the
//! strings the table stores.

use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::types::{TimeUnit, Value};

use crate::config::Config;

/// A row of `pages` (D26, D34–D36).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRow {
    pub run_id: u64,
    pub url_id: u64,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub content_hash: Option<Vec<u8>>,
    /// What the hash covered: `root`, `region` or `whole` (D15).
    pub hash_scope: &'static str,
    /// The name of the profile that extracted the page (D35).
    pub profile: String,
    pub source_url: Option<String>,
    pub source_repo: Option<String>,
    pub source_ref: Option<String>,
    pub source_path: Option<String>,
    pub in_sitemap: bool,
    /// Whether a link, not a seed or a sitemap, claimed the page. A listed
    /// page is claimed by its sitemap first (D47), so this is false even where
    /// links name it; whether anything links to a page is `links`' to say.
    pub via_links: bool,
    /// The root whose scope claimed the page, as the root's canonical URL
    /// (`Root::canonical_url`); `None` for an allowlisted page, which no
    /// root's scope holds (D3).
    pub root: Option<String>,
}

/// A root as a run records it (D15): its canonical URL, which `runs.roots` and
/// `pages.root` hold, and the `hash_root` and `hash_exclude` its pages were
/// hashed under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRoot {
    pub url: String,
    pub hash_root: Vec<String>,
    pub hash_exclude: Vec<String>,
}

impl RunRoot {
    /// Every root `config` names, in its order.
    pub fn of(config: &Config) -> Vec<RunRoot> {
        config
            .roots
            .iter()
            .map(|r| RunRoot {
                url: r.canonical_url(),
                hash_root: r.hash.root.clone(),
                hash_exclude: r.hash.exclude.clone(),
            })
            .collect()
    }
}

impl From<&str> for RunRoot {
    /// A root with the default hash scope.
    fn from(url: &str) -> RunRoot {
        RunRoot {
            url: url.to_owned(),
            hash_root: Vec::new(),
            hash_exclude: Vec::new(),
        }
    }
}

/// A row of `page_facts` (D37): `key` is the full `<profile>.<key>`, as
/// [`crate::profile::Facts::record`] builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactRow {
    pub run_id: u64,
    pub url_id: u64,
    pub key: String,
    pub value: String,
}

/// One of D16's five outcomes, as `targets.outcome` stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    BrokenUrl,
    BrokenAnchor,
    Error,
    Unchecked,
}

impl Outcome {
    /// The value the tables hold.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::BrokenUrl => "broken_url",
            Outcome::BrokenAnchor => "broken_anchor",
            Outcome::Error => "error",
            Outcome::Unchecked => "unchecked",
        }
    }
}

/// One response in a target's redirect chain (D7): the URL asked, the status
/// it answered, and whether the hop is a page's `refresh` of delay 0 rather
/// than a 3xx. A 301 or 308 says to update the link, a 302 or 307 is often the
/// site's design (I5), and a refresh is an alias, which the status alone
/// cannot tell from the 200 of a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hop {
    pub url: String,
    pub status: u16,
    pub refresh: bool,
}

/// Where a target's answer came from (D51), as `targets.answered_by` stores
/// it: a file read from a local build, a row copied from a baseline run, or a
/// request the run made (a target it did not request, `unchecked`, included).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsweredBy {
    Local,
    Baseline,
    Live,
}

impl AnsweredBy {
    /// The value the table holds.
    pub fn as_str(self) -> &'static str {
        match self {
            AnsweredBy::Local => "local",
            AnsweredBy::Baseline => "baseline",
            AnsweredBy::Live => "live",
        }
    }
}

/// A row of `targets` (D26): one URL's HTTP-level outcome in one run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRow {
    pub run_id: u64,
    pub url_id: u64,
    pub url: String,
    pub status: Option<u16>,
    pub final_url: Option<String>,
    /// The hops from `url` to `final_url`, each URL with the response it
    /// gave; empty, and stored NULL, where the target was not redirected.
    pub redirect_chain: Vec<Hop>,
    pub outcome: Outcome,
    pub reason: Option<String>,
    /// `None` where no request was made for the URL, its host's robots.txt
    /// request that got no response counting as one (D8).
    pub checked_at: Option<SystemTime>,
    /// From the target's start to its last hop's end, robots.txt, the host's
    /// pace and the body's extraction included, so it is no response time.
    pub duration_ms: Option<u32>,
    pub answered_by: AnsweredBy,
}

/// A row of `anchors` (D12, D34).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorRow {
    pub run_id: u64,
    pub url_id: u64,
    pub anchor: String,
    pub tag: String,
    /// `content`, `chrome` or `unknown` (D34).
    pub placement: &'static str,
}

/// A row of `links` (D13, D26): one occurrence of a reference on a page, one
/// entry of a sitemap, or a root's link to a sitemap file it read (D47).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRow {
    pub run_id: u64,
    pub src_url_id: u64,
    pub target_url_id: u64,
    pub fragment: Option<String>,
    pub raw_href: String,
    /// One of D13's kinds, `sitemap` and `sitemap_file` included.
    pub kind: &'static str,
    pub xref_refspec: Option<String>,
}

/// One row for the writer to append.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    Page(PageRow),
    Fact(FactRow),
    Target(TargetRow),
    Anchor(AnchorRow),
    Link(LinkRow),
}

/// `at` as a TIMESTAMPTZ value, microseconds from the epoch.
pub(super) fn timestamp(at: SystemTime) -> Value {
    let micros = match at.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_micros() as i64,
        Err(e) => -(e.duration().as_micros() as i64),
    };
    Value::Timestamp(TimeUnit::Microsecond, micros)
}
