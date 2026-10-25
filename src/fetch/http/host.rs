//! Per-host politeness: how fast, how many at once, and what robots.txt
//! allows (D8, D18, D69). `http` asks for a permit before each request and
//! supplies robots.txt when asked; nothing here makes a request.
//!
//! A host's pace is a next-allowed instant, not a limiter library (D11): each
//! request reserves the slot after the last one's, so a retry after a
//! backoff waits for whichever is later, the backoff or the host's own
//! interval. The interval is the configured rate until the host's robots.txt
//! is in, then [`interval`] combines the two as D18 says, the slowest
//! `Crawl-delay` among the name's origins read. A host's pace, permits and
//! switch are keyed by the name [`url::Host`] prints, ports and schemes
//! sharing it; a robots.txt is an origin's, its scheme, host and port (D69).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use texting_robots::Robot;
use tokio::sync::{OnceCell, OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, sleep_until};
use url::Url;

use crate::config::{Config, HostSettings, Rate};
use crate::fetch::{Access, Failure};

/// The longest wait between two requests to a host. A `Crawl-delay` past it
/// is read as this, so an absurd one cannot stall the run for good.
const MAX_INTERVAL: Duration = Duration::from_secs(3600);

/// What fetching an origin's robots.txt came to.
#[derive(Debug, Clone)]
pub enum Listing {
    /// A 2xx answer: the file, as the bytes were read.
    Found(Vec<u8>),
    /// A 4xx answer but 429: no file, so everything is allowed (RFC 9309).
    Absent,
    /// The host answered, but with no file to read: a 5xx or a 429 that
    /// outlasts the retries, or redirects that led nowhere, onto another host
    /// that gave no response among them; or a stop. The origin is disallowed
    /// for the run (RFC 9309), its targets `unchecked`, `robots_unavailable`.
    Unreachable,
    /// A request on `name`, the first or one a redirect kept there, produced
    /// no response after its retries: the name does not exist, the host
    /// refused, it never answered. Each target of an origin on that name
    /// settles as that failure (D8, D16); one on another name that reached
    /// this listing by a redirect is `Unreachable`.
    Failed { failure: Failure, name: String },
}

/// An origin's robots.txt, settled once per run.
#[derive(Debug)]
enum Robots {
    /// Not there (a 4xx), or a file the parser refuses: all allowed. A host
    /// with `robots = false` never reads one.
    Open,
    Rules(Robot),
    Unreachable,
    Failed(Failure),
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
    reads: Mutex<Reads>,
}

/// The robots.txt of each origin asked, apart from what a host makes of it,
/// so an origin whose file redirects to another's shares that one's read.
#[derive(Debug, Default)]
struct Reads {
    cells: HashMap<String, Arc<OnceCell<Arc<Listing>>>>,
    /// The origin whose read is waiting on another's, keyed by the waiter.
    waiting: HashMap<String, String>,
}

/// One host's pace, permits and the robots.txt of each of its origins.
#[derive(Debug)]
pub struct Host {
    name: String,
    token: String,
    settings: HostSettings,
    default_per_second: f64,
    is_root: bool,
    permits: Arc<Semaphore>,
    pace: Mutex<Pace>,
    robots: Mutex<HashMap<String, Arc<OnceCell<Arc<Robots>>>>>,
}

#[derive(Debug)]
struct Pace {
    next: Instant,
    interval: Duration,
    /// The slowest `Crawl-delay` among the origins read so far.
    delay: Option<f32>,
}

/// The key of `url`'s origin: its scheme, host and port (RFC 9309, 2.3).
fn origin(url: &Url) -> String {
    url.origin().ascii_serialization()
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
            reads: Mutex::new(Reads::default()),
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
                name: name.clone(),
                is_root: self.0.root_hosts.contains(&name),
                token: self.0.token.clone(),
                settings,
                default_per_second: self.0.rate.per_second,
                permits: Arc::new(Semaphore::new(concurrency as usize)),
                pace: Mutex::new(Pace {
                    next: Instant::now(),
                    interval,
                    delay: None,
                }),
                robots: Mutex::new(HashMap::new()),
            })
        });
        Arc::clone(host)
    }

    /// The robots.txt of `url`'s origin, read with `load` the first time
    /// anyone asks. `via` is the origin whose own read is asking, when this
    /// read stands in for its file because its redirect ended at this
    /// origin's `/robots.txt` (D69); a pair of origins redirecting to each
    /// other would wait on each other for ever, so the read that would close
    /// the cycle is `Unreachable`, a redirect leading nowhere.
    pub async fn read(
        &self,
        url: &Url,
        via: Option<&Url>,
        load: impl Future<Output = Listing>,
    ) -> Arc<Listing> {
        let key = origin(url);
        let (cell, waiting) = {
            let mut reads = self.0.reads.lock().expect("no panic holds the lock");
            let cell = Arc::clone(reads.cells.entry(key.clone()).or_default());
            let mut waiting = None;
            if let Some(from) = via.map(origin).filter(|_| !cell.initialized()) {
                let mut at = Some(&key);
                while let Some(next) = at {
                    if *next == from {
                        return Arc::new(Listing::Unreachable);
                    }
                    at = reads.waiting.get(next);
                }
                reads.waiting.insert(from.clone(), key.clone());
                waiting = Some(from);
            }
            (cell, waiting)
        };
        let _wait = waiting.map(|from| Wait {
            inner: &self.0,
            from,
        });
        Arc::clone(cell.get_or_init(|| async { Arc::new(load.await) }).await)
    }
}

/// A read waiting on another's, forgotten when the wait ends or is dropped.
struct Wait<'a> {
    inner: &'a Inner,
    from: String,
}

impl Drop for Wait<'_> {
    fn drop(&mut self) {
        let mut reads = self.inner.reads.lock().expect("no panic holds the lock");
        reads.waiting.remove(&self.from);
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

    /// Whether `url` may be requested, loading its origin's robots.txt with
    /// `load` the first time anyone asks about that origin. `load` is not
    /// polled for a host with `robots = false`. Once a file is in, the host's
    /// interval is set from it and the settings, the slowest `Crawl-delay`
    /// among the origins read governing ([`interval`], D18, D69).
    pub async fn access(&self, url: &Url, load: impl Future<Output = Listing>) -> Access {
        if !self.settings.robots {
            return Access::Allowed;
        }
        let cell = {
            let mut robots = self.robots.lock().expect("no panic holds the lock");
            Arc::clone(robots.entry(origin(url)).or_default())
        };
        let robots = cell
            .get_or_init(|| async {
                let robots = match load.await {
                    Listing::Found(bytes) => match Robot::new(&self.token, &bytes) {
                        Ok(robot) => Robots::Rules(robot),
                        // The parser is forgiving; a file it refuses has
                        // no rules we could obey.
                        Err(_) => Robots::Open,
                    },
                    Listing::Absent => Robots::Open,
                    Listing::Unreachable => Robots::Unreachable,
                    // A silence on this name fails its origins; on another
                    // name it is a redirect that led nowhere.
                    Listing::Failed { failure, name } if name == self.name => {
                        Robots::Failed(failure)
                    }
                    Listing::Failed { .. } => Robots::Unreachable,
                };
                if let Some(delay) = robots.delay() {
                    let mut pace = self.pace.lock().expect("no panic holds the lock");
                    pace.delay = Some(pace.delay.map_or(delay, |slowest| slowest.max(delay)));
                    pace.interval = interval(
                        &self.settings,
                        self.default_per_second,
                        self.is_root,
                        pace.delay,
                    );
                }
                Arc::new(robots)
            })
            .await;
        match &**robots {
            Robots::Open => Access::Allowed,
            Robots::Unreachable => Access::Unavailable,
            Robots::Failed(failure) => Access::Failed(failure.clone()),
            Robots::Rules(robot) if robot.allowed(url.as_str()) => Access::Allowed,
            Robots::Rules(_) => Access::Disallowed,
        }
    }

    /// The `Sitemap:` lines of `url`'s origin, once [`Host::access`] has
    /// loaded its robots.txt; empty before then, and for a file not read or
    /// not there.
    pub fn sitemaps(&self, url: &Url) -> Vec<String> {
        let cell = {
            let robots = self.robots.lock().expect("no panic holds the lock");
            robots.get(&origin(url)).map(Arc::clone)
        };
        match cell.as_deref().and_then(OnceCell::get).map(|r| &**r) {
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
