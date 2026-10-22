//! A run checked against a baseline: what it holds of the build, and what it
//! leaves to the baseline's answer (D22, D52, D54).
//!
//! The frontier sorts a URL the build does not hold, and the config would
//! have asked about, into two classes. One outside the build's prefix, or on
//! an allowlisted host, is answered by copying the baseline's rows for it
//! when it holds a target, else asked live (`--externals=live` asks an
//! allowlisted one always, and `skip` leaves it `unchecked` instead, D54). One under the prefix and not built waits in [`Pending`] until every
//! built page is read, because whether it is `not_built` depends on the
//! component-versions those pages carry ([`Held`]): in a held one the build
//! is the truth and the URL is `broken_url`, in any other the baseline is
//! asked as for a URL outside. The copies are sent last, after every target
//! the build wrote, so a copied landing page never doubles a built one. The
//! run itself (`Shared::dispatch`, `settle_unbuilt` and `copy_from_baseline`)
//! is the parent's; this holds what it waits on.
//!
//! A tagged link of a built page, an xref Antora could not resolve (I6), waits
//! too: whether its refspec names a component-version the build holds is
//! known only then (`xref`, D20, D22).

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard};

use percent_encoding::percent_decode_str;
use url::Url;

use crate::extract::Kind;
use crate::store::LinkRow;
use crate::url::Canonical;

/// The reason of a URL the build lacks in a component-version it holds
/// (D22).
pub(super) const NOT_BUILT: &str = "not_built";

/// The component-versions the built pages carry, which the build holds
/// whole: Antora has that component's whole catalog, so a URL in one that is
/// not built is a page the change deleted (D22).
#[derive(Debug, Default)]
pub(super) struct Held {
    components: HashMap<String, Versions>,
}

#[derive(Debug, Default)]
struct Versions {
    /// Versions whose URLs carry a version segment, by name, as a refspec
    /// names them.
    named: HashSet<String>,
    /// The version segments the built pages of those versions sit at: a
    /// version's own name, or the one the build serves it under, as a
    /// `latest_version_segment` serves the latest (I16).
    served: HashSet<String>,
    /// A version whose URLs carry none: a placeholder, or no version.
    bare: bool,
}

impl Held {
    /// Records a built page at `page`, below the build's `prefix`, of its
    /// component and `version` as the nav container gives them (D19). The
    /// placeholders Antora writes for a version its URLs omit, `""` and
    /// `master`, and a page with no version, are alike. A named version is
    /// served at the segment the page sits at, the second below the prefix
    /// where the first is the component, else at its own name.
    pub(super) fn record(
        &mut self,
        prefix: &Url,
        page: &Url,
        component: &str,
        version: Option<&str>,
    ) {
        let versions = self.components.entry(component.to_owned()).or_default();
        match version {
            None | Some("" | "master") => versions.bare = true,
            Some(version) => {
                versions.named.insert(version.to_owned());
                let segment = page
                    .path()
                    .strip_prefix(prefix.path())
                    .and_then(|below| {
                        let mut segments = below.split('/').map(decode);
                        if segments.next()? == component {
                            segments.next()
                        } else {
                            None
                        }
                    })
                    .filter(|s| !s.is_empty());
                versions
                    .served
                    .insert(segment.unwrap_or_else(|| version.to_owned()));
            }
        }
    }

    /// Whether `url`, below the build's `prefix`, is in a component-version
    /// held: its first segment is a held component, and its second is a
    /// segment a held version of it is served at, or the component holds a
    /// version whose URLs name none (`site-invariants.md`, I16). A URL whose
    /// second segment is neither a built page's nor a held version's own name
    /// is the baseline's, unless the component's held version is a bare one.
    pub(super) fn holds(&self, prefix: &Url, url: &Url) -> bool {
        // `report::inverse::prefix` reads the same layout back from the stored run (D56).
        // deficiency: KD5 A component-version is held whole, so a component
        // split across repos' start paths and partly built reads the pages
        // of the other start paths as `not_built`, and the inverse check
        // skips their links, live though they are (D56). Telling the pages a
        // change owns from the rest takes a flag naming their globs, which
        // every repo's CI would keep current by hand.
        // deficiency: KD17 A build serving its latest at its own name (its
        // playbook sets no `urls.latest_version_segment`, as docs.fp.o's CI
        // playbooks do not) holds no `latest` segment, so a link written
        // under production's `latest` (113 absolute hrefs on 112 `fedora`
        // pages, `runs/p5.2-en-us-20261020/en-us.duckdb`) is the baseline's:
        // where the change deleted its page it passes on production's `ok`,
        // and where the change adds it, production lacks it and it fails the
        // gate as introduced though the build has it. Nothing in the build
        // says which version `latest` stands for. A config key naming it
        // would mirror each playbook by hand in the deployment's config
        // (D27), and a single-repo build may not hold production's latest.
        let Some(below) = url.path().strip_prefix(prefix.path()) else {
            return false;
        };
        let mut segments = below.split('/').map(decode);
        let Some(versions) = segments.next().and_then(|c| self.components.get(&c)) else {
            return false;
        };
        segments
            .next()
            .is_some_and(|s| versions.served.contains(&s))
            || versions.bare
    }
}

impl Held {
    /// Whether the build holds the component `component` at `version`, as a
    /// refspec names them: a refspec with no version asks for the component's
    /// latest, which the build holds whenever it holds the component, and one
    /// with a version is held where a built page carries that version, by
    /// name, whatever segment it is served at, or the component's held
    /// version is a bare one.
    pub(super) fn holds_coordinates(&self, component: &str, version: Option<&str>) -> bool {
        let Some(versions) = self.components.get(component) else {
            return false;
        };
        match version {
            None => true,
            Some("" | "master") => versions.bare,
            Some(version) => versions.bare || versions.named.contains(version),
        }
    }
}

fn decode(segment: &str) -> String {
    percent_decode_str(segment).decode_utf8_lossy().into_owned()
}

/// A URL under the prefix, in scope and not built, claimed and not yet judged.
pub(super) struct Unbuilt {
    pub(super) url: Url,
    /// The kind of the first link to it, as KD4 has it for an allowlisted URL.
    pub(super) kind: Kind,
    pub(super) form: Canonical,
}

/// A link a built page's profile tagged as a reference it could not resolve,
/// its row not yet written: pointing at the page itself, with the raw refspec
/// as its fragment, as a live crawl writes it (D20).
pub(super) struct Xref {
    /// The page the link is on.
    pub(super) from: Url,
    /// The profile that read the page.
    pub(super) profile: &'static str,
    pub(super) link: LinkRow,
}

/// What a run against a baseline has claimed and not yet answered.
#[derive(Default)]
pub(super) struct Pending {
    held: Mutex<Held>,
    unbuilt: Mutex<Vec<Unbuilt>>,
    copies: Mutex<Vec<u64>>,
    xrefs: Mutex<Vec<Xref>>,
}

fn locked<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().expect("no panic holds the lock")
}

impl Pending {
    /// Records a built page's component-version ([`Held::record`]).
    pub(super) fn hold(&self, prefix: &Url, page: &Url, component: &str, version: Option<&str>) {
        locked(&self.held).record(prefix, page, component, version);
    }

    /// The component-versions held so far.
    pub(super) fn held(&self) -> MutexGuard<'_, Held> {
        locked(&self.held)
    }

    pub(super) fn defer_xref(&self, xref: Xref) {
        locked(&self.xrefs).push(xref);
    }

    pub(super) fn take_xrefs(&self) -> Vec<Xref> {
        std::mem::take(&mut *locked(&self.xrefs))
    }

    pub(super) fn defer(&self, unbuilt: Unbuilt) {
        locked(&self.unbuilt).push(unbuilt);
    }

    /// `url_id`'s rows are to be copied from the baseline.
    pub(super) fn copy(&self, url_id: u64) {
        locked(&self.copies).push(url_id);
    }

    /// The URLs deferred, split by the held set as it now stands: those in a
    /// held component-version, and those to be answered from the baseline or
    /// live.
    pub(super) fn judge(&self, prefix: &Url) -> (Vec<Unbuilt>, Vec<Unbuilt>) {
        let held = locked(&self.held);
        std::mem::take(&mut *locked(&self.unbuilt))
            .into_iter()
            .partition(|u| held.holds(prefix, &u.url))
    }

    pub(super) fn take_copies(&self) -> Vec<u64> {
        std::mem::take(&mut *locked(&self.copies))
    }
}

#[cfg(test)]
mod tests;
