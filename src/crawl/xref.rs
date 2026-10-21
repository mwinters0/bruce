//! The xrefs of a built page that Antora could not resolve, resolved against
//! the baseline once every built page is read (D20, D22, D38, D55, D57).
//!
//! A partial build lacks most components, so every xref into one reaches the
//! page as `#<refspec>` (I6, I10). The profile that read the page maps the
//! refspec to the baseline's page for it, or to the URL it names (D57),
//! unless the build holds the refspec's component-version: Antora had that
//! component's whole catalog, and the xref stays unresolved. The held set is
//! final only after the last built page, so the links wait in
//! [`Pending`](super::baseline::Pending).

use url::Url;

use super::baseline::{Held, Xref};
use crate::profile::{BoxError, Lookup, Registry};
use crate::store::{BaselineIndex, LinkRow};

/// The facts a built page's component-version is read from, as `crawl` notes
/// them (D19).
const COMPONENT: &str = "antora.component";
const VERSION: &str = "antora.version";

/// What a profile's Resolve sees of a run against a baseline: the baseline's
/// pages, and the component-versions the build holds.
struct Against<'a> {
    held: &'a Held,
    pages: &'a BaselineIndex,
}

impl Lookup for Against<'_> {
    fn pages_with_facts(&self, facts: &[(&str, &str)]) -> Result<Vec<Url>, BoxError> {
        self.pages.pages_with_facts(facts)
    }

    fn facts(&self, page: &Url) -> Result<Vec<(String, String)>, BoxError> {
        self.pages.facts(page)
    }

    fn built(&self, facts: &[(&str, &str)]) -> bool {
        let get = |key| facts.iter().find(|(k, _)| *k == key).map(|(_, v)| *v);
        get(COMPONENT).is_some_and(|c| self.held.holds_coordinates(c, get(VERSION)))
    }
}

/// Each xref's link row, with the URL its refspec resolves to, fragment
/// included: `None` where the profile cannot resolve it, or fails to, either
/// leaving it an unresolved xref (D20).
pub(super) fn resolve(
    xrefs: Vec<Xref>,
    held: &Held,
    pages: &BaselineIndex,
    registry: &Registry,
) -> Vec<(LinkRow, Option<Url>)> {
    let lookup = Against { held, pages };
    xrefs
        .into_iter()
        .map(|x| {
            let resolved = registry
                .get(x.profile)
                .zip(x.link.xref_refspec.as_deref())
                .and_then(|(profile, refspec)| profile.resolve(refspec, &x.from, &lookup).ok())
                .flatten();
            (x.link, resolved)
        })
        .collect()
}
