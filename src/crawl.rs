//! The crawl: the frontier, hops, outcomes and the rows a run writes (D2,
//! D3, D7, D13, D14, D16, D28; `roadmap-P3-crawler.md`, "What gets
//! requested", "Redirects", "Assets and allowlisted pages" and "Sitemaps").
//!
//! A run reads the roots' sitemaps first, so a page a sitemap lists is
//! claimed as listed before any link can name it (D47), and then seeds the
//! roots' URLs and the pages the sitemaps listed. It asks about each target
//! once, at most [`WORKERS`] at a time, a host's own pace held by the
//! fetcher (D8). One task follows one target's hops itself, so each passes
//! robots.txt and its host's rate (D7), reads the answer, and then, on the
//! blocking pool, extracts the page and sends its rows to the writer from
//! there, so the writer's bounded channel pushes back on the pool and not on
//! the runtime (D28). A URL is asked about by whoever claims its `url_id` first
//! (`frontier`); a page a redirect lands on is recorded by whichever of the
//! redirecting target and the landing URL's own discovery claims it first.
//! The run writes every target's HTTP-level outcome and leaves
//! `links.outcome` NULL, the verdicts being SQL's after the crawl (D28).
//!
//! A sitemap is a target, and each of its entries a link from it, of kind
//! `sitemap` for a page and `sitemap_file` for an index's sitemap (D13). A
//! root's sitemap is the one it sets, or else those its host's robots.txt
//! names, or else the first of `sitemap.xml` under the root and at the host
//! that answers with one (D40), and the root links to each of them, kind
//! `sitemap_file`, once though two roots share the file; a file probed and not
//! found writes nothing, since nothing named it (D47). A root with no sitemap
//! goes on by links alone, and has no `sitemap_file` link to show for it.
//!
//! A [`Stopper`] ends a run early (`roadmap-P3-crawler.md`, "An interrupted
//! run"): no target starts after it, a started one takes no further hop, and
//! the fetcher sends no retry or fallback (`fetch`'s doc). A target whose
//! hops have not ended [`STOP_GRACE`] after the stop is dropped there. What a
//! stopped run wrote is whole targets, and every target claimed and not
//! written is counted as cut.

mod frontier;
mod hops;
mod outcome;
mod sitemap;

use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use tokio::task::JoinSet;
use url::Url;
use xxhash_rust::xxh3::xxh3_64;

use crate::config::{Config, HashScope, ProfileChoice};
use crate::extract::{self, Extracted, Kind};
use crate::fetch::{Access, Body, Failure, FailureKind, Fetcher, MAX_SITEMAP, Mode};
use crate::profile::Registry;
use crate::store::{
    AnchorRow, FactRow, Hop, LinkRow, Outcome, PageRow, Row, RunId, TargetRow, WriteHandle,
    WriterGone,
};
use crate::url::Canonical;
use frontier::{Frontier, Plan};
use hops::{Chain, Stop};
use sitemap::Sitemap;

pub use crate::fetch::Stopper;

/// Targets in flight at once, across hosts. A host's own cap and pace are
/// the fetcher's (D8); this only bounds the tasks a run holds.
const WORKERS: usize = 64;

/// How many files deep sitemap indexes are followed, the first counting as 1.
const MAX_SITEMAP_DEPTH: u8 = 3;

/// How long after a stop a target may go on with its hops; one not done by
/// then is dropped unwritten.
pub const STOP_GRACE: Duration = Duration::from_secs(30);

/// What a finished run wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub targets: u64,
    pub pages: u64,
    /// Sitemap files read; 0 when the run had none to go on.
    pub sitemaps: u64,
    /// Sitemap entries outside every root's scope, which are dropped.
    pub dropped: u64,
    /// A [`Stopper`] came while targets were still to be asked about, so the
    /// run is not whole.
    pub stopped: bool,
    /// Targets found and never written, the stop having come first: a lower
    /// bound on what the run lacks (D48).
    pub cut: u64,
}

/// Why a run stopped before it was done.
#[derive(Debug)]
pub enum Error {
    /// The writer has stopped; [`crate::store::Writer::finish`] has the cause.
    Writer(WriterGone),
    /// A task panicked.
    Panicked,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Writer(e) => e.fmt(f),
            Error::Panicked => f.write_str("a crawl task panicked"),
        }
    }
}

impl std::error::Error for Error {}

impl From<WriterGone> for Error {
    fn from(e: WriterGone) -> Error {
        Error::Writer(e)
    }
}

/// One run of the crawl.
pub struct Crawl<F> {
    shared: Arc<Shared<F>>,
}

struct Shared<F> {
    fetcher: F,
    frontier: Frontier,
    registry: Registry,
    run: RunId,
    rows: WriteHandle,
    targets: AtomicU64,
    pages: AtomicU64,
    sitemaps: AtomicU64,
    dropped: AtomicU64,
    stop: Stopper,
    /// How long a started target may go on after the stop.
    grace: Duration,
    /// Work was left undone when the stop came.
    cut: AtomicBool,
}

/// A target claimed and waiting to be asked about.
struct Item {
    url: Url,
    /// The kind of the link that found it; a seed counts as an anchor.
    kind: Kind,
    /// Found by a link, not seeded.
    via_links: bool,
    /// Listed by a sitemap.
    in_sitemap: bool,
    plan: Plan,
    form: Canonical,
}

/// A page read, before its rows are written.
struct Page {
    extracted: Extracted,
    /// The decompressed bytes of the body.
    size: u64,
}

/// The last response of a target.
struct Answer {
    status: u16,
    content_type: Option<String>,
    /// `Unread`, or `TooLarge`: the page's text went to the extractor.
    body: Body,
    page: Option<Page>,
    /// What a sitemap file listed.
    sitemap: Option<Sitemap>,
}

/// How a target's hops ended.
enum End {
    Answered(Box<Answer>),
    Failed(Failure),
    /// robots.txt forbids the URL, or could not be read: not requested.
    Blocked(&'static str),
    Stopped(Stop),
    /// The page answered and the extractor could not read it.
    Unreadable,
}

/// A target whose hops have ended, with the plan and form of the URL they
/// ended at.
struct Done {
    item: Item,
    chain: Vec<Hop>,
    plan: Plan,
    form: Canonical,
    end: End,
    started: SystemTime,
    /// Whether any request left, robots.txt aside.
    asked: bool,
    elapsed: std::time::Duration,
}

impl<F> Crawl<F>
where
    F: Fetcher + Send + Sync + 'static,
{
    /// A run `run` of `config`, writing through `rows`.
    pub fn new(
        fetcher: F,
        config: Config,
        registry: Registry,
        run: RunId,
        rows: WriteHandle,
    ) -> Crawl<F> {
        let stop = Stopper::default();
        Crawl {
            shared: Arc::new(Shared {
                fetcher: fetcher.with_stop(stop.clone()),
                frontier: Frontier::new(config),
                registry,
                run,
                rows,
                targets: AtomicU64::new(0),
                pages: AtomicU64::new(0),
                sitemaps: AtomicU64::new(0),
                dropped: AtomicU64::new(0),
                stop,
                grace: STOP_GRACE,
                cut: AtomicBool::new(false),
            }),
        }
    }

    /// What stops this run early, to be called from elsewhere while it runs.
    pub fn stopper(&self) -> Stopper {
        self.shared.stop.clone()
    }

    /// This run, with `grace` in place of [`STOP_GRACE`].
    #[cfg(test)]
    fn with_grace(mut self, grace: Duration) -> Crawl<F> {
        Arc::get_mut(&mut self.shared)
            .expect("not yet shared")
            .grace = grace;
        self
    }

    /// Crawls from the roots' sitemaps and URLs until nothing is left to ask
    /// about.
    pub async fn run(self) -> Result<Summary, Error> {
        let shared = self.shared;
        let mut listed = Vec::new();
        let sitemaps = shared.discover().await?;
        shared.drive(sitemaps, Some(&mut listed)).await?;
        for root in &shared.frontier.config().roots {
            let (plan, form) = shared.frontier.plan(&root.url, Kind::Anchor);
            if shared.frontier.claim(form.id()) {
                listed.push(Item {
                    url: root.url.clone(),
                    kind: Kind::Anchor,
                    via_links: false,
                    in_sitemap: false,
                    plan,
                    form,
                });
            }
        }
        shared.drive(listed, None).await?;
        let load = |n: &AtomicU64| n.load(Ordering::Relaxed);
        let targets = load(&shared.targets);
        Ok(Summary {
            targets,
            pages: load(&shared.pages),
            sitemaps: load(&shared.sitemaps),
            dropped: load(&shared.dropped),
            stopped: shared.cut.load(Ordering::Relaxed),
            // Every claim writes one target row unless the stop cut it.
            cut: shared.frontier.claimed().saturating_sub(targets),
        })
    }
}

impl<F> Shared<F>
where
    F: Fetcher + Send + Sync + 'static,
{
    /// Whether the run has been stopped, asked only where work is left, so
    /// a yes marks the run cut short.
    fn stopping(&self) -> bool {
        let stopped = self.stop.is_stopped();
        if stopped {
            self.cut.store(true, Ordering::Relaxed);
        }
        stopped
    }

    /// `work`, unless the stop's grace runs out first: `None` then, the
    /// stop marking the run cut short. The bound is per target (D48).
    async fn in_grace<T>(&self, work: impl Future<Output = T>) -> Option<T> {
        let out = tokio::select! {
            biased;
            () = async { tokio::time::sleep_until(self.stop.stopped().await + self.grace).await } => None,
            out = work => Some(out),
        };
        if out.is_none() {
            self.stopping();
        }
        out
    }

    /// Asks about `start` and every target it leads to, at most [`WORKERS`]
    /// at a time. With `hold`, only sitemaps are asked about and every other
    /// target found goes there instead. Once stopped it starts nothing, and
    /// returns when the targets started are done.
    async fn drive(
        self: &Arc<Self>,
        start: Vec<Item>,
        mut hold: Option<&mut Vec<Item>>,
    ) -> Result<(), Error> {
        let mut queue = VecDeque::new();
        let mut tasks = JoinSet::new();
        let mut found = start;
        loop {
            for item in found.drain(..) {
                match &mut hold {
                    Some(held) if !matches!(item.plan, Plan::Sitemap { .. }) => held.push(item),
                    _ => queue.push_back(item),
                }
            }
            if !queue.is_empty() && self.stopping() {
                queue.clear();
            }
            while tasks.len() < WORKERS
                && let Some(item) = queue.pop_front()
            {
                tasks.spawn(Arc::clone(self).process(item));
            }
            let Some(joined) = tasks.join_next().await else {
                return Ok(());
            };
            found = joined.map_err(|_| Error::Panicked)??;
        }
    }

    /// Each root's sitemaps, claimed (D40): the one the root sets, or those
    /// its host's robots.txt names, each a target whatever it answers; or
    /// else the first of the probes that answers with a sitemap, read here,
    /// in which case what it listed is returned in its place.
    async fn discover(self: &Arc<Self>) -> Result<Vec<Item>, Error> {
        let mut found = Vec::new();
        let mut links = Vec::new();
        for root in &self.frontier.config().roots {
            if self.stopping() {
                break;
            }
            let (_, root_form) = self.frontier.plan(&root.url, Kind::Anchor);
            // Each file the root read is a link from it, once (D13, D47).
            let mut read = Vec::new();
            let mut link = |raw: &str, form: &Canonical| {
                if !read.contains(&form.id()) {
                    read.push(form.id());
                    links.push(self.link(&root_form, form.id(), None, raw, Kind::SitemapFile));
                }
            };
            let named: Vec<(String, Url)> = match &root.sitemap {
                Some(url) => vec![(url.to_string(), url.clone())],
                None => {
                    let Some(lines) = self.in_grace(self.fetcher.sitemaps(&root.url)).await else {
                        break;
                    };
                    lines
                }
                .into_iter()
                .filter_map(|line| Some((line.clone(), root.url.join(&line).ok()?)))
                .filter(|(_, url)| matches!(url.scheme(), "http" | "https"))
                .collect(),
            };
            if !named.is_empty() {
                for (raw, url) in named {
                    let item = self.sitemap_item(url, 1);
                    link(&raw, &item.form);
                    if self.frontier.claim(item.form.id()) {
                        found.push(item);
                    }
                }
                continue;
            }
            let mut probed = Vec::new();
            for probe in ["sitemap.xml", "/sitemap.xml"] {
                let Ok(url) = root.url.join(probe) else {
                    continue;
                };
                if probed.contains(&url) {
                    continue;
                }
                probed.push(url.clone());
                let item = self.sitemap_item(url, 1);
                if self.frontier.is_claimed(item.form.id()) {
                    // Another root's sitemap, read already or to be.
                    link(probe, &item.form);
                    break;
                }
                let Some(done) = Arc::clone(self).follow(item).await? else {
                    break;
                };
                let is_sitemap = matches!(&done.end, End::Answered(a)
                    if a.sitemap.is_some() || a.body == Body::TooLarge);
                if is_sitemap {
                    link(probe, &done.item.form);
                }
                if is_sitemap && self.frontier.claim(done.item.form.id()) {
                    found.extend(Arc::clone(self).write_blocking(done).await?);
                    break;
                }
            }
        }
        let shared = Arc::clone(self);
        tokio::task::spawn_blocking(move || {
            links
                .into_iter()
                .try_for_each(|row| shared.rows.send(Row::Link(row)))
        })
        .await
        .map_err(|_| Error::Panicked)??;
        Ok(found)
    }

    /// A link from the URL `src` names to the target `target_url_id`, as a
    /// crawl writes it rather than a page yields it.
    fn link(
        &self,
        src: &Canonical,
        target_url_id: u64,
        fragment: Option<String>,
        raw: &str,
        kind: Kind,
    ) -> LinkRow {
        LinkRow {
            run_id: self.run.0,
            src_url_id: src.id(),
            target_url_id,
            fragment,
            raw_href: raw.to_owned(),
            kind: kind.as_str(),
            xref_refspec: None,
            outcome: None,
            reason: None,
            // KD3, as for a page's links.
            redirected: false,
        }
    }

    /// A sitemap at `url`, `depth` files deep, not yet claimed.
    fn sitemap_item(&self, url: Url, depth: u8) -> Item {
        let (_, form) = self.frontier.plan(&url, Kind::SitemapFile);
        Item {
            url,
            kind: Kind::SitemapFile,
            via_links: false,
            in_sitemap: false,
            plan: Plan::Sitemap { depth },
            form,
        }
    }

    /// Follows one target's hops and writes what they came to. Returns the
    /// targets its page or sitemap named that this run has not met.
    async fn process(self: Arc<Self>, item: Item) -> Result<Vec<Item>, Error> {
        match Arc::clone(&self).follow(item).await? {
            Some(done) => self.write_blocking(done).await,
            None => Ok(Vec::new()),
        }
    }

    /// [`Shared::write`], on the blocking pool, so a full writer channel
    /// blocks a pool thread and never the runtime (D28).
    async fn write_blocking(self: Arc<Self>, done: Done) -> Result<Vec<Item>, Error> {
        tokio::task::spawn_blocking(move || self.write(done))
            .await
            .map_err(|_| Error::Panicked)?
    }

    /// The plan and form of `next`, where a hop from a URL planned `plan`
    /// leads: a sitemap stays one wherever it is moved to.
    fn hop(&self, next: &Url, plan: Plan, kind: Kind) -> (Plan, Canonical) {
        match plan {
            Plan::Sitemap { .. } => (plan, self.frontier.plan(next, kind).1),
            _ => self.frontier.hop_plan(next, kind),
        }
    }

    /// Follows one target's hops to where they end; `None` if the stop
    /// cut them short, or its grace ran out first.
    async fn follow(self: Arc<Self>, item: Item) -> Result<Option<Done>, Error> {
        let shared = Arc::clone(&self);
        let walked = shared.in_grace(self.walk(item)).await;
        Ok(walked.transpose()?.flatten())
    }

    /// [`Shared::follow`], with no bound in time.
    async fn walk(self: Arc<Self>, item: Item) -> Result<Option<Done>, Error> {
        let started = SystemTime::now();
        let clock = Instant::now();
        let mut chain = Chain::new(item.form.as_str());
        let (mut url, mut plan, mut form) = (item.url.clone(), item.plan, item.form.clone());
        let mut asked = false;
        let end = loop {
            // No target starts, and none takes a hop, once stopped; a
            // robots.txt the stop kept from being read says nothing.
            if self.stopping() {
                return Ok(None);
            }
            let access = self.fetcher.access(&url).await;
            if self.stopping() {
                return Ok(None);
            }
            match access {
                Access::Allowed => {}
                Access::Disallowed => break End::Blocked("robots"),
                Access::Unavailable => break End::Blocked("robots_unavailable"),
            }
            asked = true;
            let mode = match plan {
                Plan::Status => Mode::Asset,
                Plan::Sitemap { .. } => Mode::Sitemap,
                _ => Mode::Page,
            };
            let mut response = match self.fetcher.fetch(&url, mode).await {
                Ok(response) => response,
                Err(failure) if failure.kind == FailureKind::Stopped => {
                    self.stopping();
                    return Ok(None);
                }
                Err(failure) => break End::Failed(failure),
            };
            if let Some(next) = hops::location(&url, &response) {
                let (next_plan, next_form) = self.hop(&next, plan, item.kind);
                if let Err(stop) = chain.follow(response.status, false, next_form.as_str()) {
                    break End::Stopped(stop);
                }
                (url, plan, form) = (next, next_plan, next_form);
                continue;
            }
            let (mut page, mut listed) = (None, None);
            match std::mem::replace(&mut response.body, Body::Unread) {
                Body::Html { text, size } => {
                    let Ok(extracted) = self.read(&url, plan, text).await? else {
                        break End::Unreadable;
                    };
                    if let Some(next) = hops::refresh_target(&extracted.links) {
                        let (next_plan, next_form) = self.hop(&next, plan, item.kind);
                        if let Err(stop) = chain.follow(response.status, true, next_form.as_str()) {
                            break End::Stopped(stop);
                        }
                        (url, plan, form) = (next, next_plan, next_form);
                        continue;
                    }
                    page = Some(Page { extracted, size });
                }
                Body::Raw(bytes) => {
                    let read =
                        tokio::task::spawn_blocking(move || sitemap::read(&bytes, MAX_SITEMAP));
                    match read.await.map_err(|_| Error::Panicked)? {
                        Ok(map) => listed = Some(map),
                        Err(sitemap::Unread::TooLarge) => response.body = Body::TooLarge,
                        Err(sitemap::Unread::NotASitemap) => break End::Unreadable,
                    }
                }
                unread => response.body = unread,
            }
            break End::Answered(Box::new(Answer {
                status: response.status,
                content_type: response.content_type,
                body: response.body,
                page,
                sitemap: listed,
            }));
        };
        Ok(Some(Done {
            item,
            chain: chain.into_hops(),
            plan,
            form,
            end,
            started,
            asked,
            elapsed: clock.elapsed(),
        }))
    }

    /// Extracts `text`, the body of the page at `url`, on the blocking pool
    /// (`lol_html` is synchronous). A page of a root is read under that
    /// root's profile and hash scope, and any other under neither.
    async fn read(
        self: &Arc<Self>,
        url: &Url,
        plan: Plan,
        text: String,
    ) -> Result<Result<Extracted, String>, Error> {
        let shared = Arc::clone(self);
        let url = url.clone();
        tokio::task::spawn_blocking(move || {
            let (auto, whole) = (ProfileChoice::Auto, HashScope::default());
            let (choice, scope) = match plan {
                Plan::Page { root } => {
                    let root = &shared.frontier.config().roots[root];
                    (&root.profile, &root.hash)
                }
                _ => (&auto, &whole),
            };
            extract::extract(&text, &url, choice, scope, &shared.registry)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|_| Error::Panicked)
    }

    /// Writes the rows of a target whose hops have ended, and returns the
    /// targets its page named that are new to the run.
    fn write(&self, done: Done) -> Result<Vec<Item>, Error> {
        let Done {
            item,
            chain,
            plan,
            form,
            end,
            started,
            asked,
            elapsed,
        } = done;
        let (outcome, reason) = match &end {
            End::Answered(a) => outcome::answered(a.status, &a.body),
            End::Failed(failure) => outcome::failed(failure),
            End::Blocked(reason) => (Outcome::Unchecked, Some(*reason)),
            End::Stopped(stop) => (Outcome::BrokenUrl, Some(stop.as_str())),
            End::Unreadable => (Outcome::Error, Some("unreadable")),
        };
        let row =
            |form: &Canonical, redirect_chain: Vec<Hop>, final_url: Option<String>| TargetRow {
                run_id: self.run.0,
                url_id: form.id(),
                url: form.as_str().to_owned(),
                status: match &end {
                    End::Answered(a) => Some(a.status),
                    _ => None,
                },
                final_url,
                redirect_chain,
                outcome,
                reason: reason.map(str::to_owned),
                checked_at: asked.then_some(started),
                duration_ms: asked.then(|| u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX)),
            };

        // Where a chain ends is a target of its own (D46).
        let lands = !chain.is_empty() && !matches!(end, End::Stopped(_));
        let owns = !lands || self.frontier.claim(form.id());
        if lands && owns {
            self.send_target(row(&form, Vec::new(), None))?;
        }
        let final_url = lands.then(|| form.as_str().to_owned());
        self.send_target(row(&item.form, chain, final_url))?;

        let mut found = Vec::new();
        if owns && let End::Answered(answer) = &end {
            if let Some(page) = &answer.page {
                let content_type = answer.content_type.as_deref();
                self.write_page(&item, plan, &form, content_type, page, &mut found)?;
            }
            if let (Some(listed), Plan::Sitemap { depth }) = (&answer.sitemap, plan) {
                self.write_sitemap(depth, &form, listed, &mut found)?;
            }
        }
        Ok(found)
    }

    /// Writes a link from the sitemap `form` names to each entry it lists, of
    /// kind `sitemap` for a page and `sitemap_file` for an index's sitemap,
    /// and claims what is new. A page outside every root's
    /// scope is dropped, a sitemap being broader than its root (D10); an
    /// index's sitemaps are followed wherever they are, down to
    /// [`MAX_SITEMAP_DEPTH`].
    fn write_sitemap(
        &self,
        depth: u8,
        form: &Canonical,
        listed: &Sitemap,
        found: &mut Vec<Item>,
    ) -> Result<(), Error> {
        self.sitemaps.fetch_add(1, Ordering::Relaxed);
        let (locs, index) = match listed {
            Sitemap::Pages(locs) => (locs, false),
            Sitemap::Index(locs) => (locs, true),
        };
        for loc in locs {
            let (target_url_id, fragment) = match Url::parse(loc) {
                Err(_) => (self.invalid(loc)?, None),
                Ok(url) => {
                    let mut bare = url.clone();
                    bare.set_fragment(None);
                    let (plan, target_form) = self.frontier.plan(&bare, Kind::Sitemap);
                    let id = target_form.id();
                    if !index && !matches!(plan, Plan::Page { .. }) {
                        self.dropped.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    if self.frontier.claim(id) {
                        let unchecked_as = match plan {
                            Plan::Unchecked("scheme") => Some("scheme"),
                            _ if index && depth >= MAX_SITEMAP_DEPTH => Some("too_deep"),
                            _ => None,
                        };
                        match unchecked_as {
                            Some(reason) => self.send_target(unchecked(
                                self.run,
                                id,
                                target_form.as_str(),
                                Outcome::Unchecked,
                                reason,
                            ))?,
                            None if index => found.push(self.sitemap_item(bare, depth + 1)),
                            None => found.push(Item {
                                url: bare,
                                kind: Kind::Sitemap,
                                via_links: false,
                                in_sitemap: true,
                                plan,
                                form: target_form,
                            }),
                        }
                    }
                    (id, url.fragment().map(str::to_owned))
                }
            };
            let kind = if index {
                Kind::SitemapFile
            } else {
                Kind::Sitemap
            };
            let row = self.link(form, target_url_id, fragment, loc, kind);
            self.rows.send(Row::Link(row))?;
        }
        Ok(())
    }

    /// The target of an href or entry that resolves to no URL, which the raw
    /// text alone names (D26), written by its first claimant.
    fn invalid(&self, raw: &str) -> Result<u64, Error> {
        let id = xxh3_64(raw.as_bytes());
        if self.frontier.claim(id) {
            self.send_target(unchecked(
                self.run,
                id,
                raw,
                Outcome::BrokenUrl,
                "invalid_url",
            ))?;
        }
        Ok(id)
    }

    /// Writes the `pages` row, the anchors and, for a page of a root, the
    /// facts and links of the page `form` names. An allowlisted page is one hop
    /// deep: its links were never followed, and recording them would have
    /// orphan and inbound reports count a site nobody crawls.
    fn write_page(
        &self,
        item: &Item,
        plan: Plan,
        form: &Canonical,
        content_type: Option<&str>,
        page: &Page,
        found: &mut Vec<Item>,
    ) -> Result<(), Error> {
        let e = &page.extracted;
        let url_id = form.id();
        let whole = matches!(plan, Plan::Page { .. });
        let edit = e.edit_link.as_ref().filter(|_| whole);
        let source = edit.and_then(|l| l.source.as_ref());
        self.pages.fetch_add(1, Ordering::Relaxed);
        self.rows.send(Row::Page(PageRow {
            run_id: self.run.0,
            url_id,
            title: e.title.clone(),
            content_type: content_type.map(str::to_owned),
            size: Some(page.size),
            content_hash: Some(e.hash.to_be_bytes().to_vec()),
            profile: e.profile.to_owned(),
            source_url: edit.map(|l| l.url.to_string()),
            source_repo: source.map(|s| s.repo.clone()),
            source_ref: source.map(|s| s.git_ref.clone()),
            source_path: source.map(|s| s.path.clone()),
            in_sitemap: item.in_sitemap,
            via_links: item.via_links,
        }))?;
        for a in &e.anchors {
            self.rows.send(Row::Anchor(AnchorRow {
                run_id: self.run.0,
                url_id,
                anchor: a.value.clone(),
                tag: a.tag.clone(),
                placement: a.placement.as_str(),
            }))?;
        }
        if !whole {
            return Ok(());
        }
        for (key, value) in e.facts.pairs() {
            self.rows.send(Row::Fact(FactRow {
                run_id: self.run.0,
                url_id,
                key: key.clone(),
                value: value.clone(),
            }))?;
        }
        for link in &e.links {
            let (target_url_id, fragment) = match &link.url {
                Some(target) => {
                    let mut bare = target.clone();
                    bare.set_fragment(None);
                    let (plan, target_form) = self.frontier.plan(&bare, link.kind);
                    if self.frontier.claim(target_form.id()) {
                        match plan {
                            Plan::Unchecked(reason) => self.send_target(unchecked(
                                self.run,
                                target_form.id(),
                                target_form.as_str(),
                                Outcome::Unchecked,
                                reason,
                            ))?,
                            plan => found.push(Item {
                                url: bare,
                                kind: link.kind,
                                via_links: true,
                                in_sitemap: false,
                                plan,
                                form: target_form.clone(),
                            }),
                        }
                    }
                    (target_form.id(), target.fragment().map(str::to_owned))
                }
                None => {
                    // Only a `<base href>` that cannot be a base makes this
                    // depend on the page, its relative hrefs then sharing a
                    // target.
                    (self.invalid(&link.raw)?, None)
                }
            };
            self.rows.send(Row::Link(LinkRow {
                run_id: self.run.0,
                src_url_id: url_id,
                target_url_id,
                fragment,
                raw_href: link.raw.clone(),
                kind: link.kind.as_str(),
                xref_refspec: link.tag.clone(),
                outcome: None,
                reason: None,
                // deficiency: KD3 The column is always false. A page's links
                // are written when it is read, before the targets they name
                // are asked about, and D2 lets only `runs.finished_at` change
                // once written; whether a link goes through a redirect is
                // `targets.redirect_chain` of its target, a join. Holding a
                // run's links back until their targets settle costs memory
                // for the whole link graph. The column is vestigial, and
                // dropping it is a schema change (D43).
                redirected: false,
            }))?;
        }
        Ok(())
    }

    fn send_target(&self, row: TargetRow) -> Result<(), Error> {
        self.targets.fetch_add(1, Ordering::Relaxed);
        self.rows.send(Row::Target(row))?;
        Ok(())
    }
}

/// A target no request was made for.
fn unchecked(run: RunId, url_id: u64, url: &str, outcome: Outcome, reason: &str) -> TargetRow {
    TargetRow {
        run_id: run.0,
        url_id,
        url: url.to_owned(),
        status: None,
        final_url: None,
        redirect_chain: Vec::new(),
        outcome,
        reason: Some(reason.to_owned()),
        checked_at: None,
        duration_ms: None,
    }
}

#[cfg(test)]
mod tests;
