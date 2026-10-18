//! Per-host politeness: how fast, how many at once, and what robots.txt
//! allows (D8, D18). `http` asks for a permit before each request and
//! supplies robots.txt when asked; nothing here makes a request.
//!
//! A host's pace is a next-allowed instant, not a limiter library (D11): each
//! request reserves the slot after the last one's, so a retry after a
//! backoff waits for whichever is later, the backoff or the host's own
//! interval. The interval is the configured rate until the host's robots.txt
//! is in, then [`interval`] combines the two as D18 says. A host's state is
//! keyed by the name [`url::Host`] prints, ports and schemes sharing it.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use texting_robots::Robot;
use tokio::sync::{OnceCell, OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, sleep_until};
use url::Url;

use crate::config::{Config, HostSettings, Rate};

/// The longest wait between two requests to a host. A `Crawl-delay` past it
/// is read as this, so an absurd one cannot stall the run for good.
const MAX_INTERVAL: Duration = Duration::from_secs(3600);

/// What fetching a host's robots.txt came to.
#[derive(Debug)]
pub enum Listing {
    /// A 2xx answer: the file, as the bytes were read.
    Found(Vec<u8>),
    /// A 4xx answer: no file, so everything is allowed (RFC 9309).
    Absent,
    /// A 5xx, no answer, or anything else: the host is disallowed for the
    /// run (RFC 9309), its targets `unchecked` with `robots_unavailable`.
    Unreachable,
}

/// Whether a URL may be requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Allowed,
    /// A rule of the host's robots.txt forbids it: `unchecked`, `robots`.
    Disallowed,
    /// The host's robots.txt could not be read: `unchecked`,
    /// `robots_unavailable`.
    Unavailable,
}

/// A host's robots.txt, settled once per run.
#[derive(Debug)]
enum Robots {
    /// Not read (the host has `robots = false`) or not there: all allowed.
    Open,
    Rules(Robot),
    Unreachable,
}

impl Robots {
    fn delay(&self) -> Option<f32> {
        match self {
            Robots::Rules(robot) => robot.delay,
            _ => None,
        }
    }
}

/// Every host's state, shared by clones.
#[derive(Debug, Clone)]
pub struct Hosts(Arc<Inner>);

#[derive(Debug)]
struct Inner {
    rate: Rate,
    settings: BTreeMap<String, HostSettings>,
    root_hosts: HashSet<String>,
    token: String,
    hosts: Mutex<HashMap<String, Arc<Host>>>,
}

/// One host's pace, permits and robots.txt.
#[derive(Debug)]
pub struct Host {
    token: String,
    settings: HostSettings,
    default_per_second: f64,
    is_root: bool,
    permits: Arc<Semaphore>,
    pace: Mutex<Pace>,
    robots: OnceCell<Robots>,
}

#[derive(Debug)]
struct Pace {
    next: Instant,
    interval: Duration,
}

impl Hosts {
    /// Politeness as `config` sets it, robots.txt read under its
    /// [`Config::robots_token`].
    pub fn new(config: &Config) -> Hosts {
        let root_hosts = config
            .roots
            .iter()
            .filter_map(|r| r.url.host().map(|h| h.to_string()))
            .collect();
        Hosts(Arc::new(Inner {
            rate: config.rate.clone(),
            settings: config.hosts.clone(),
            root_hosts,
            token: config.robots_token().to_owned(),
            hosts: Mutex::new(HashMap::new()),
        }))
    }

    /// No pacing, for tests that are not about it.
    #[cfg(test)]
    pub fn unlimited() -> Hosts {
        let mut config = Config::for_url("http://unlimited.invalid/").expect("a valid URL");
        config.rate = Rate {
            per_second: 1e9,
            concurrency: 1024,
        };
        Hosts::new(&config)
    }

    /// The state of `url`'s host, created on first sight.
    pub fn host(&self, url: &Url) -> Arc<Host> {
        let name = url.host().map(|h| h.to_string()).unwrap_or_default();
        let mut hosts = self.0.hosts.lock().expect("no panic holds the lock");
        let host = hosts.entry(name.clone()).or_insert_with(|| {
            let settings = self.0.settings.get(&name).copied().unwrap_or_default();
            let concurrency = settings.concurrency.unwrap_or(self.0.rate.concurrency);
            let interval = interval(
                &settings,
                self.0.rate.per_second,
                self.0.root_hosts.contains(&name),
                None,
            );
            Arc::new(Host {
                is_root: self.0.root_hosts.contains(&name),
                token: self.0.token.clone(),
                settings,
                default_per_second: self.0.rate.per_second,
                permits: Arc::new(Semaphore::new(concurrency as usize)),
                pace: Mutex::new(Pace {
                    next: Instant::now(),
                    interval,
                }),
                robots: OnceCell::new(),
            })
        });
        Arc::clone(host)
    }
}

impl Host {
    /// Waits for a place among the host's in-flight requests and then for
    /// its next slot. The permit is held until dropped, so a request holds
    /// its place while its body is read.
    pub async fn admit(&self) -> OwnedSemaphorePermit {
        let permit = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .expect("the semaphore is never closed");
        let slot = {
            let mut pace = self.pace.lock().expect("no panic holds the lock");
            let at = pace.next.max(Instant::now());
            pace.next = at + pace.interval;
            at
        };
        sleep_until(slot).await;
        permit
    }

    /// Whether `url` may be requested, loading the host's robots.txt with
    /// `load` the first time anyone asks. `load` is not polled for a host
    /// with `robots = false`. Once the file is in, its `Crawl-delay` sets
    /// the host's interval (D18).
    pub async fn access(&self, url: &Url, load: impl Future<Output = Listing>) -> Access {
        let robots = self
            .robots
            .get_or_init(|| async {
                let robots = if self.settings.robots {
                    match load.await {
                        Listing::Found(bytes) => match Robot::new(&self.token, &bytes) {
                            Ok(robot) => Robots::Rules(robot),
                            // The parser is forgiving; a file it refuses has
                            // no rules we could obey.
                            Err(_) => Robots::Open,
                        },
                        Listing::Absent => Robots::Open,
                        Listing::Unreachable => Robots::Unreachable,
                    }
                } else {
                    Robots::Open
                };
                let mut pace = self.pace.lock().expect("no panic holds the lock");
                pace.interval = interval(
                    &self.settings,
                    self.default_per_second,
                    self.is_root,
                    robots.delay(),
                );
                robots
            })
            .await;
        match robots {
            Robots::Open => Access::Allowed,
            Robots::Unreachable => Access::Unavailable,
            Robots::Rules(robot) if robot.allowed(url.as_str()) => Access::Allowed,
            Robots::Rules(_) => Access::Disallowed,
        }
    }

    /// The host's `Sitemap:` lines, once [`Host::access`] has loaded its
    /// robots.txt; empty before then, and for a file not read or not there.
    pub fn sitemaps(&self) -> Vec<String> {
        match self.robots.get() {
            Some(Robots::Rules(robot)) => robot.sitemaps.clone(),
            _ => Vec::new(),
        }
    }
}

/// The time between two requests to a host (D18). `crawl_delay` is the
/// host's, once its robots.txt has been read; a negative or non-finite one
/// is ignored.
///
/// With no `per_second` written for the host, a `Crawl-delay` governs and
/// the default rate never overrides it; with none of either, the default
/// rate. Where `per_second` is written, it beats a `Crawl-delay` on a host a
/// root starts on and loses to a slower one anywhere else.
fn interval(
    settings: &HostSettings,
    default_per_second: f64,
    is_root_host: bool,
    crawl_delay: Option<f32>,
) -> Duration {
    let written = settings.per_second.map(|n| 1.0 / n);
    let delay = crawl_delay
        .filter(|d| d.is_finite() && *d >= 0.0)
        .map(f64::from);
    let seconds = match (written, delay) {
        (Some(w), Some(_)) if is_root_host => w,
        (Some(w), Some(d)) => w.max(d),
        (Some(w), None) => w,
        (None, Some(d)) => d,
        (None, None) => 1.0 / default_per_second,
    };
    Duration::from_secs_f64(seconds.min(MAX_INTERVAL.as_secs_f64()))
}

#[cfg(test)]
mod tests;
