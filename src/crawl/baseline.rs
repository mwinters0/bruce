//! A run checked against a baseline: what it holds of the build, and what it
//! leaves to the baseline's answer (D22, D52; `roadmap-P5-ci.md`, "A baseline
//! answer is copied into the run" and "What a build holds").
//!
//! The frontier sorts a URL the build does not hold into two classes. One
//! outside the build's prefix, or on an allowlisted host, is answered by
//! copying the baseline's rows for it when it holds a target, else asked
//! live (`--externals=skip` leaves an allowlisted one `unchecked` instead,
//! D54). One under the prefix and not built waits in [`Pending`] until every
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
    /// Versions whose URLs carry a version segment.
    named: HashSet<String>,
    /// A version whose URLs carry none: a placeholder, or no version.
    bare: bool,
}

impl Held {
    /// Records a built page's component and `version`, as the nav container
    /// gives them (D19). The placeholders Antora writes for a version its
    /// URLs omit, `""` and `master`, and a page with no version, are alike.
    pub(super) fn record(&mut self, component: &str, version: Option<&str>) {
        let versions = self.components.entry(component.to_owned()).or_default();
        match version {
            None | Some("" | "master") => versions.bare = true,
            Some(version) => {
                versions.named.insert(version.to_owned());
            }
        }
    }

    /// Whether `url`, below the build's `prefix`, is in a component-version
    /// held: its first segment is a held component, and its second is a held
    /// version of it or the component holds a version whose URLs name none
    /// (`site-invariants.md`, I16). A URL whose second segment names a
    /// version the build lacks is the baseline's, unless the component's held
    /// version is a bare one.
    pub(super) fn holds(&self, prefix: &Url, url: &Url) -> bool {
        // `report::inverse::prefix` reads the same layout back from the stored run (D56).
        // deficiency: KD5 A component-version is held whole, so a component
        // split across repos' start paths and partly built reads the pages
        // of the other start paths as `not_built`. Telling the pages a
        // change owns from the rest takes a flag naming their globs, which
        // every repo's CI would keep current by hand.
        let Some(below) = url.path().strip_prefix(prefix.path()) else {
            return false;
        };
        let mut segments = below.split('/').map(decode);
        let Some(versions) = segments.next().and_then(|c| self.components.get(&c)) else {
            return false;
        };
        segments.next().is_some_and(|s| versions.named.contains(&s)) || versions.bare
    }
}

impl Held {
    /// Whether the build holds the component `component` at `version`, as a
    /// refspec names them: a refspec with no version asks for the component's
    /// latest, which the build holds whenever it holds the component, and one
    /// with a version is held as a URL under that version segment is
    /// ([`Held::holds`]).
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
    pub(super) fn hold(&self, component: &str, version: Option<&str>) {
        locked(&self.held).record(component, version);
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
