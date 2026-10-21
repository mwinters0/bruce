//! The frontier: which URLs a run asks about, and how (D3, D7, D26).
//!
//! [`Frontier::plan`] sorts a URL into a [`Plan`] by the first test that
//! applies: a scheme other than `http(s)`, a `skip` glob (D50), a root's
//! scope, an allowlisted host (D3), a root's own host outside every scope
//! (`out_of_scope`, D10), any other host (`host_not_allowlisted`); robots.txt
//! is asked later, at each hop. It returns the URL's canonical form, the
//! string its `url_id` hashes. A URL in a root's scope, unless skipped, has that
//! root's form; every other URL has the plain form, `canonical(url, false)`,
//! since no root's `strip_query` governs it. [`Frontier::claim`] is the run's one
//! deduplication: a target is written by whoever claims its `url_id` first
//! (D13), and a claim is never given back.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use url::Url;

use crate::config::Config;
use crate::extract::Kind;
use crate::fetch::local::Site;
use crate::store::{AnsweredBy, BaselineIndex, RunId};
use crate::url::{Canonical, canonical};

/// The reason of a URL `skip` matched (D50).
pub const SKIPPED: &str = "skipped";

/// The reason of an allowlisted URL `--externals=skip` neither copies nor asks
/// (D22, D54).
pub const EXTERNALS_SKIPPED: &str = "externals_skipped";

/// What a run does about a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// In root `root`'s scope: GET, an HTML response extracted and its links
    /// followed.
    Page { root: usize },
    /// On an allowlisted host, or in scope where the build lacks it and the
    /// baseline holds no target, and named by an `anchor` link: GET,
    /// extracted for its anchors only (D3).
    Anchors,
    /// As [`Plan::Anchors`] but named by any other kind of link, or reached
    /// by a redirect where a link would leave it unchecked or to the
    /// baseline: asked for its status alone, HEAD with the GET fallback
    /// (D13).
    Status,
    /// Not requested; the target row says why.
    Unchecked(&'static str),
    /// Not requested: the baseline's rows for it are copied into the run
    /// (D52). Only a run checked against a baseline plans one.
    Baseline,
    /// In a root's scope, under the build's prefix and not built: judged
    /// once every built page is read, by the component-versions the build
    /// holds (D22). Only a run checked against a baseline plans one.
    Unbuilt,
    /// A sitemap, `depth` files below where discovery began (1 for the file
    /// discovered or set): GET, its body read for its entries (D14). Only
    /// the crawl plans one; [`Frontier::plan`] never does.
    Sitemap { depth: u8 },
}

/// The URLs a run has met, and how each is treated.
pub struct Frontier {
    config: Config,
    seen: Mutex<HashSet<u64>>,
    ci: Option<Ci>,
}

/// What `--externals` asks of a URL on an allowlisted host (D22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Externals {
    /// Nothing is requested: the baseline's rows are copied where it holds
    /// the target, else the URL is `unchecked` (D54).
    Skip,
    /// The baseline's verdict where it holds one, else a request.
    Baseline,
    /// A request, whatever the baseline holds.
    Live,
}

/// The baseline a run is checked against: its run in the database being
/// written, and the targets it holds.
#[derive(Debug, Clone)]
pub struct Baseline {
    pub run: RunId,
    pub index: BaselineIndex,
    pub externals: Externals,
}

/// What a run over a local build knows of the build and of its baseline.
struct Ci {
    site: Arc<Site>,
    baseline: Baseline,
}

impl Frontier {
    pub fn new(config: Config) -> Frontier {
        Frontier {
            config,
            seen: Mutex::new(HashSet::new()),
            ci: None,
        }
    }

    /// This frontier for a run over `site`, checked against `baseline`: a
    /// URL the build does not hold is no longer a page to read from it.
    pub fn against(&mut self, site: Arc<Site>, baseline: Baseline) {
        self.ci = Some(Ci { site, baseline });
    }

    /// The baseline this run is checked against, if any.
    pub fn baseline(&self) -> Option<&Baseline> {
        self.ci.as_ref().map(|ci| &ci.baseline)
    }

    /// Where the answer for the URL `form` names comes from: the build, if it
    /// holds the URL, else a request (D51).
    pub fn answered_by(&self, form: &Canonical) -> AnsweredBy {
        let built = self
            .ci
            .as_ref()
            .is_some_and(|ci| Url::parse(form.as_str()).is_ok_and(|url| ci.site.holds(&url)));
        if built {
            AnsweredBy::Local
        } else {
            AnsweredBy::Live
        }
    }

    /// The build's prefix, if the run is over a build.
    pub fn prefix(&self) -> Option<&Url> {
        self.ci.as_ref().map(|ci| ci.site.prefix())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Whether `url_id` is new to the run; true for the first caller only.
    pub fn claim(&self, url_id: u64) -> bool {
        self.seen
            .lock()
            .expect("no panic holds the lock")
            .insert(url_id)
    }

    /// How many URLs have been claimed.
    pub fn claimed(&self) -> u64 {
        self.seen.lock().expect("no panic holds the lock").len() as u64
    }

    /// Whether `url_id` has been claimed, claiming nothing.
    pub fn is_claimed(&self, url_id: u64) -> bool {
        self.seen
            .lock()
            .expect("no panic holds the lock")
            .contains(&url_id)
    }

    /// How `url` is treated when a link of `kind` names it, and its canonical
    /// form. The first root whose scope holds it is its root. Order is the
    /// spec's, after the scheme: `skip` (D50), matched on the plain form whatever the host,
    /// then a root's scope, then an allowlisted host, then the rest, an
    /// out-of-scope URL on a root's own host told apart from any other host
    /// by its reason.
    pub fn plan(&self, url: &Url, kind: Kind) -> (Plan, Canonical) {
        if !matches!(url.scheme(), "http" | "https") {
            return (Plan::Unchecked("scheme"), canonical(url, false));
        }
        let plain = canonical(url, false);
        if self.config.skip.is_match(plain.as_str()) {
            return (Plan::Unchecked(SKIPPED), plain);
        }
        for (root, r) in self.config.roots.iter().enumerate() {
            let form = r.canonical(url);
            if r.in_scope(&form) {
                let plan = match &self.ci {
                    Some(ci) if !ci.site.holds(url) && ci.site.under(url) => Plan::Unbuilt,
                    Some(ci) if !ci.site.holds(url) => self.beyond(&form, kind),
                    _ => Plan::Page { root },
                };
                return (plan, form);
            }
        }
        let form = plain;
        let host = url.host().map(|h| h.to_string()).unwrap_or_default();
        let plan = if self.config.allowlist.contains(&host) {
            self.external(&form, kind)
        } else if self.config.is_root_host(&host) {
            Plan::Unchecked("out_of_scope")
        } else {
            Plan::Unchecked("host_not_allowlisted")
        };
        (plan, form)
    }

    /// How a URL the build does not hold is answered: by copying the
    /// baseline's rows where it holds a target for it, else by asking, as an
    /// allowlisted URL is asked (D22, D52).
    pub fn beyond(&self, form: &Canonical, kind: Kind) -> Plan {
        self.copy_or(form, live(kind))
    }

    /// [`Plan::Baseline`] where the baseline holds a target for `form`, else
    /// `otherwise`.
    fn copy_or(&self, form: &Canonical, otherwise: Plan) -> Plan {
        match self.baseline() {
            Some(b) if b.index.holds(form.id()) => Plan::Baseline,
            _ => otherwise,
        }
    }

    /// How an allowlisted URL is answered: `--externals` decides in a run
    /// against a baseline, and in any other it is asked.
    fn external(&self, form: &Canonical, kind: Kind) -> Plan {
        match self.baseline().map(|b| b.externals) {
            Some(Externals::Skip) => self.copy_or(form, Plan::Unchecked(EXTERNALS_SKIPPED)),
            Some(Externals::Baseline) => self.beyond(form, kind),
            Some(Externals::Live) | None => live(kind),
        }
    }

    /// [`Frontier::plan`] for the URL a redirect leads to: a hop is followed
    /// wherever it goes, so what a link would leave unchecked is asked for its
    /// status alone. A skipped URL stays skipped, and ends the chain (D50);
    /// so does an allowlisted one under `--externals=skip` (D54). A hop onto
    /// a URL the build lacks is asked, never copied from the baseline nor
    /// judged `not_built`: an Antora alias stays in its component, so no hop
    /// out of a built page is known to land there.
    pub fn hop_plan(&self, url: &Url, kind: Kind) -> (Plan, Canonical) {
        match self.plan(url, kind) {
            (Plan::Unchecked(SKIPPED), form) => (Plan::Unchecked(SKIPPED), form),
            (Plan::Baseline | Plan::Unchecked(EXTERNALS_SKIPPED), form)
                if self.is_skipped_external(url) =>
            {
                (Plan::Unchecked(EXTERNALS_SKIPPED), form)
            }
            (Plan::Unchecked(_) | Plan::Baseline | Plan::Unbuilt, form) => (Plan::Status, form),
            planned => planned,
        }
    }

    /// Whether `url` is on an allowlisted host, in no root's scope, in a run
    /// whose `--externals` is `skip`.
    fn is_skipped_external(&self, url: &Url) -> bool {
        let host = url.host().map(|h| h.to_string()).unwrap_or_default();
        self.baseline()
            .is_some_and(|b| b.externals == Externals::Skip)
            && self.config.allowlist.contains(&host)
            && !self
                .config
                .roots
                .iter()
                .any(|r| r.in_scope(&r.canonical(url)))
    }
}

/// How a URL is asked about, by the kind of link that names it.
fn live(kind: Kind) -> Plan {
    // deficiency: KD4 The first link to an allowlisted URL decides how
    // it is asked: an anchor link gets a GET and its anchors are
    // stored, any other kind gets a HEAD. A URL named first by an
    // image and later by an anchor link with a fragment is never
    // read, so that fragment has no anchors to be checked against.
    // Reading it again would write the target twice (D2), and
    // deciding after the whole crawl needs the frontier to hold every
    // link's kind.
    if kind == Kind::Anchor {
        Plan::Anchors
    } else {
        Plan::Status
    }
}

#[cfg(test)]
mod tests;
