//! The crawl: the frontier, hops, outcomes and the rows a run writes (D2,
//! D3, D7, D13, D16, D28; `roadmap-P3-crawler.md`, "What gets requested",
//! "Redirects" and "Assets and allowlisted pages").
//!
//! A run seeds the roots' URLs and asks about each target once, at most
//! [`WORKERS`] at a time, a host's own pace held by the fetcher (D8). One
//! task follows one target's hops itself, so each passes robots.txt and its
//! host's rate (D7), reads the answer, and then, on the blocking pool,
//! extracts the page and sends its rows to the writer from there, so the
//! writer's bounded channel pushes back on the pool and not on the
//! runtime (D28). A URL is asked about by whoever claims its `url_id` first
//! (`frontier`); a page a redirect lands on is recorded by whichever of the
//! redirecting target and the landing URL's own discovery claims it first.
//! The run writes every target's HTTP-level outcome and leaves
//! `links.outcome` NULL, the verdicts being SQL's after the crawl (D28).

mod frontier;
mod hops;
mod outcome;

use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime};

use tokio::task::JoinSet;
use url::Url;
use xxhash_rust::xxh3::xxh3_64;

use crate::config::{Config, HashScope, ProfileChoice};
use crate::extract::{self, Extracted, Kind};
use crate::fetch::{Access, Body, Failure, Fetcher, Mode};
use crate::profile::Registry;
use crate::store::{
    AnchorRow, FactRow, Hop, LinkRow, Outcome, PageRow, Row, RunId, TargetRow, WriteHandle,
    WriterGone,
};
use crate::url::Canonical;
use frontier::{Frontier, Plan};
use hops::{Chain, Stop};

/// Targets in flight at once, across hosts. A host's own cap and pace are
/// the fetcher's (D8); this only bounds the tasks a run holds.
const WORKERS: usize = 64;

/// What a finished run wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub targets: u64,
    pub pages: u64,
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
}

/// A target claimed and waiting to be asked about.
struct Item {
    url: Url,
    /// The kind of the link that found it; a seed counts as an anchor.
    kind: Kind,
    /// Found by a link, not seeded.
    via_links: bool,
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
        Crawl {
            shared: Arc::new(Shared {
                fetcher,
                frontier: Frontier::new(config),
                registry,
                run,
                rows,
                targets: AtomicU64::new(0),
                pages: AtomicU64::new(0),
            }),
        }
    }

    /// Crawls from the roots' URLs until nothing is left to ask about.
    pub async fn run(self) -> Result<Summary, Error> {
        let shared = self.shared;
        let mut queue = VecDeque::new();
        for root in &shared.frontier.config().roots {
            let (plan, form) = shared.frontier.plan(&root.url, Kind::Anchor);
            if shared.frontier.claim(form.id()) {
                queue.push_back(Item {
                    url: root.url.clone(),
                    kind: Kind::Anchor,
                    via_links: false,
                    plan,
                    form,
                });
            }
        }
        let mut tasks = JoinSet::new();
        loop {
            while tasks.len() < WORKERS
                && let Some(item) = queue.pop_front()
            {
                tasks.spawn(Arc::clone(&shared).process(item));
            }
            let Some(joined) = tasks.join_next().await else {
                break;
            };
            queue.extend(joined.map_err(|_| Error::Panicked)??);
        }
        Ok(Summary {
            targets: shared.targets.load(Ordering::Relaxed),
            pages: shared.pages.load(Ordering::Relaxed),
        })
    }
}

impl<F> Shared<F>
where
    F: Fetcher + Send + Sync + 'static,
{
    /// Follows one target's hops and writes what they came to. Returns the
    /// targets its page named that this run has not met.
    async fn process(self: Arc<Self>, item: Item) -> Result<Vec<Item>, Error> {
        let started = SystemTime::now();
        let clock = Instant::now();
        let mut chain = Chain::new(item.form.as_str());
        let (mut url, mut plan, mut form) = (item.url.clone(), item.plan, item.form.clone());
        let mut asked = false;
        let end = loop {
            match self.fetcher.access(&url).await {
                Access::Allowed => {}
                Access::Disallowed => break End::Blocked("robots"),
                Access::Unavailable => break End::Blocked("robots_unavailable"),
            }
            asked = true;
            let mode = if plan == Plan::Status {
                Mode::Asset
            } else {
                Mode::Page
            };
            let mut response = match self.fetcher.fetch(&url, mode).await {
                Ok(response) => response,
                Err(failure) => break End::Failed(failure),
            };
            if let Some(next) = hops::location(&url, &response) {
                let (next_plan, next_form) = self.frontier.hop_plan(&next, item.kind);
                if let Err(stop) = chain.follow(response.status, false, next_form.as_str()) {
                    break End::Stopped(stop);
                }
                (url, plan, form) = (next, next_plan, next_form);
                continue;
            }
            let mut page = None;
            match std::mem::replace(&mut response.body, Body::Unread) {
                Body::Html { text, size } => {
                    let Ok(extracted) = self.read(&url, plan, text).await? else {
                        break End::Unreadable;
                    };
                    if let Some(next) = hops::refresh_target(&extracted.links) {
                        let (next_plan, next_form) = self.frontier.hop_plan(&next, item.kind);
                        if let Err(stop) = chain.follow(response.status, true, next_form.as_str()) {
                            break End::Stopped(stop);
                        }
                        (url, plan, form) = (next, next_plan, next_form);
                        continue;
                    }
                    page = Some(Page { extracted, size });
                }
                unread => response.body = unread,
            }
            break End::Answered(Box::new(Answer {
                status: response.status,
                content_type: response.content_type,
                body: response.body,
                page,
            }));
        };
        let done = Done {
            item,
            chain: chain.into_hops(),
            plan,
            form,
            end,
            started,
            asked,
            elapsed: clock.elapsed(),
        };
        tokio::task::spawn_blocking(move || self.write(done))
            .await
            .map_err(|_| Error::Panicked)?
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
        if owns
            && let End::Answered(answer) = &end
            && let Some(page) = &answer.page
        {
            let content_type = answer.content_type.as_deref();
            self.write_page(&item, plan, &form, content_type, page, &mut found)?;
        }
        Ok(found)
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
            in_sitemap: false,
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
                                plan,
                                form: target_form.clone(),
                            }),
                        }
                    }
                    (target_form.id(), target.fragment().map(str::to_owned))
                }
                None => {
                    // An href that resolves to no URL is a broken link, and
                    // the raw text is all that names it (D26). Only a
                    // `<base href>` that cannot be a base makes that depend
                    // on the page, its relative hrefs then sharing a target.
                    let id = xxh3_64(link.raw.as_bytes());
                    if self.frontier.claim(id) {
                        self.send_target(unchecked(
                            self.run,
                            id,
                            &link.raw,
                            Outcome::BrokenUrl,
                            "invalid_url",
                        ))?;
                    }
                    (id, None)
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
