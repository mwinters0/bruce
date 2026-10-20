//! Antora's Resolve: a refspec Antora left as `#<refspec>` (I6), mapped to
//! the URL of the page it names through the pages' coordinates (D19, D38, D55).
//!
//! A refspec is `[version@][component:][module:]page.adoc[#fragment]`, where
//! with a component and no module the module is `ROOT` (`component::page`). A
//! refspec naming no component is the page's own, whose component-version the
//! build holds, so only one naming a component is resolved here, and only
//! while the build does not hold that component-version ([`Lookup::built`]):
//! Antora had the held catalog whole, so an xref into it that stayed
//! unresolved is broken (D20, D22).

use url::Url;

use super::super::{BoxError, Lookup};

const COMPONENT: &str = "antora.component";
const VERSION: &str = "antora.version";
const MODULE: &str = "antora.module";
const REL_PATH: &str = "antora.rel_path";

/// A refspec's parts, for a page of another component.
#[derive(Debug, PartialEq, Eq)]
struct Refspec<'a> {
    version: Option<&'a str>,
    component: &'a str,
    module: &'a str,
    rel_path: &'a str,
    fragment: Option<&'a str>,
}

/// The parts of `reference`, or `None` when it names no component or is not a
/// page reference.
fn parse(reference: &str) -> Option<Refspec<'_>> {
    let (id, fragment) = match reference.split_once('#') {
        Some((id, fragment)) => (id, Some(fragment)),
        None => (reference, None),
    };
    let (version, id) = match id.split_once('@') {
        Some((version, id)) => (Some(version), id),
        None => (None, id),
    };
    let id = match id.split_once('$') {
        Some(("page", id)) => id,
        Some(_) => return None,
        None => id,
    };
    let mut parts = id.splitn(3, ':');
    let (component, module, rel_path) = (parts.next()?, parts.next()?, parts.next()?);
    if component.is_empty() || rel_path.is_empty() {
        return None;
    }
    Some(Refspec {
        version,
        component,
        module: if module.is_empty() { "ROOT" } else { module },
        rel_path,
        fragment: fragment.filter(|f| !f.is_empty()),
    })
}

/// The URL `reference` resolves to in the pages `lookup` knows, fragment
/// included. Without a version the latest of the component's pages with that
/// module and path is taken, as Antora takes its catalog's.
pub(super) fn resolve(reference: &str, lookup: &dyn Lookup) -> Result<Option<Url>, BoxError> {
    // deficiency: KD7 A baseline page with no forge edit link has no
    // `module` or `rel_path` fact, so it can be no refspec's target and an
    // xref to it stays unresolved. The pair cannot be read from the URL (D19).
    let Some(spec) = parse(reference) else {
        return Ok(None);
    };
    let mut coordinates = vec![(COMPONENT, spec.component)];
    coordinates.extend(spec.version.map(|v| (VERSION, v)));
    if lookup.built(&coordinates) {
        return Ok(None);
    }
    coordinates.extend([(MODULE, spec.module), (REL_PATH, spec.rel_path)]);
    let candidates = lookup.pages_with_facts(&coordinates)?;
    let mut page = match (spec.version, candidates.as_slice()) {
        (_, []) => return Ok(None),
        (Some(_), [page, ..]) | (None, [page]) => page.clone(),
        (None, _) => latest(candidates, lookup)?,
    };
    page.set_fragment(spec.fragment);
    Ok(Some(page))
}

/// The page of `candidates` whose version Antora orders first (a version it
/// would call the component's latest), the earliest URL where two tie.
fn latest(candidates: Vec<Url>, lookup: &dyn Lookup) -> Result<Url, BoxError> {
    let mut best: Option<(String, Url)> = None;
    for page in candidates {
        let facts = lookup.facts(&page)?;
        let version = facts
            .iter()
            .find(|(k, _)| k == VERSION)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let newer = best
            .as_ref()
            .is_none_or(|(held, _)| compare(&version, held).is_gt());
        if newer {
            best = Some((version, page));
        }
    }
    Ok(best.expect("at least two candidates").1)
}

/// Antora's version order, `Greater` for the version it lists first: no
/// version (a bare one) before any, then a name that is no semantic version
/// in reverse alphabetical order, then semantic versions newest first. A
/// prerelease is not told apart from a release by its descriptor, which the
/// baseline does not hold: only a `-` suffix is lower than its release.
fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.is_empty(), b.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        _ => {}
    }
    match (semver(a), semver(b)) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.cmp(b),
    }
}

/// `(major, minor, patch, is a release, prerelease)` of a version that begins
/// with a number (or `v` and a number) and has a `.` or is one number, which
/// is what Antora calls semantic.
fn semver(version: &str) -> Option<([u64; 3], bool, &str)> {
    let rest = version.strip_prefix('v').unwrap_or(version);
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    if !rest.contains('.') && rest.parse::<u64>().is_err() {
        return None;
    }
    let (numbers, pre) = rest.split_once('-').unwrap_or((rest, ""));
    let mut parts = numbers.split('.').map(|n| n.parse::<u64>().unwrap_or(0));
    let mut next = || parts.next().unwrap_or(0);
    Some(([next(), next(), next()], pre.is_empty(), pre))
}

#[cfg(test)]
mod tests;
