//! The `antora` profile: Antora's generator meta, nav container, content
//! region and xrefs (D19, D20). Detect and Views are built; Extract is the
//! extractor's (P2) and Resolve the resolver's (P5), and until then both do
//! nothing.

use std::cell::RefCell;

use url::Url;

use super::{BoxError, ElementHandlers, Facts, Lookup, Profile, View};

/// The profile for sites built by Antora.
pub struct Antora;

/// A page's Antora coordinates as columns (D19, D37). A page with no nav
/// container has no `antora.version`, which D19 reads as the component's only
/// or latest version, so the column is NULL there.
const PAGES: View = View {
    name: "antora_pages",
    select: "SELECT run_id, url_id, \
        max(value) FILTER (WHERE key = 'antora.component') AS component, \
        max(value) FILTER (WHERE key = 'antora.version') AS version, \
        max(value) FILTER (WHERE key = 'antora.module') AS module, \
        max(value) FILTER (WHERE key = 'antora.rel_path') AS rel_path \
        FROM page_facts WHERE key LIKE 'antora.%' GROUP BY run_id, url_id",
};

impl Profile for Antora {
    fn name(&self) -> &'static str {
        "antora"
    }

    /// Antora's generator meta reads `Antora 3.1.15` (I12): the first word.
    fn detect(&self, generator: &str) -> bool {
        generator
            .split_whitespace()
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("antora"))
    }

    fn extract<'h>(&self, _facts: &'h RefCell<Facts>) -> ElementHandlers<'h> {
        Vec::new()
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
        &[PAGES]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_claims_antora_generators_only() {
        assert!(Antora.detect("Antora 3.1.15"));
        assert!(Antora.detect("antora"));
        assert!(!Antora.detect("Docusaurus v4.0.0"));
        assert!(!Antora.detect("Antoraesque 1.0"));
        assert!(!Antora.detect(""));
    }

    #[test]
    fn view_names_carry_the_profile_name() {
        for view in Antora.views() {
            assert!(view.name.starts_with("antora_"), "{}", view.name);
        }
    }
}
