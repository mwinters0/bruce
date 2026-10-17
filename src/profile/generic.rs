//! The `generic` profile: any site. Its content region is `<main>` or
//! `[role=main]` (D34), which its Extract declares, the core's heuristics
//! doing the rest (D36). It is `auto`'s fallback, so it claims no generator string, and it
//! records no facts, ships no views and resolves no reference (D38).

use std::cell::RefCell;

use url::Url;

use super::{BoxError, Extract, Lookup, Profile, Sink, View};

/// The profile for a site the core reads without help.
pub struct Generic;

impl Profile for Generic {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn detect(&self, _generator: &str) -> bool {
        false
    }

    fn extract<'h>(&self, _sink: &'h RefCell<Sink>) -> Extract<'h> {
        Extract::region(&["main", "[role=main]"])
    }

    fn resolve(
        &self,
        _reference: &str,
        _from: &Url,
        _lookup: &dyn Lookup,
    ) -> Result<Option<Url>, BoxError> {
        Ok(None)
    }

    fn views(&self) -> &'static [View] {
        &[]
    }
}
