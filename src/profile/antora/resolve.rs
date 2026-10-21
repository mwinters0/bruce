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
//!
//! Where the pages' coordinates find no page, the URL the refspec names is the
//! answer ([`named`], D57): Antora also resolves through a page-alias and to
//! the one source file of two sharing an output URL, which the baseline's
//! facts, held once per URL, cannot tell.

use percent_encoding::percent_decode_str;
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
    // deficiency: KD10 Antora's resource ID is
    // `version@component:module:family$relative` (I21), the family after the
    // module; this takes `page$` only before the component, so
    // `comp:mod:page$x.adoc` stays unresolved and `page$comp:mod:x.adoc`, which
    // Antora refuses, resolves. Refused too: a version with no component
    // (`2.0@page.adoc`, the page's own component at another version, which the
    // build need not hold), `_` as the empty version, and any family but
    // `page`, which an xref may name (`attachment$`). A third colon is kept in
    // `rel_path` where Antora refuses the spec, and `./` or `..` there is not
    // normalised, so a page the facts hold is missed.
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
/// module and path is taken, as Antora takes its catalog's; where none has
/// them, the URL the refspec names ([`named`]).
pub(super) fn resolve(reference: &str, lookup: &dyn Lookup) -> Result<Option<Url>, BoxError> {
    let Some(spec) = parse(reference) else {
        return Ok(None);
    };
    let mut coordinates = vec![(COMPONENT, spec.component)];
    coordinates.extend(spec.version.map(|v| (VERSION, v)));
    if lookup.built(&coordinates) {
        return Ok(None);
    }
    let held = coordinates.clone();
    coordinates.extend([(MODULE, spec.module), (REL_PATH, spec.rel_path)]);
    let candidates = lookup.pages_with_facts(&coordinates)?;
    let page = match (spec.version, candidates.as_slice()) {
        (_, []) => named(&spec, &held, lookup)?,
        (Some(_), [page, ..]) | (None, [page]) => Some(page.clone()),
        (None, _) => Some(latest(candidates, lookup)?),
    };
    Ok(page.map(|mut page| {
        page.set_fragment(spec.fragment);
        page
    }))
}

/// The URL `spec` names by the layout of I16, whether or not a page stands
/// there: the component-version's prefix, read off a baseline page carrying
/// its facts (`held` names them), then the module unless `ROOT` and the page
/// without `.adoc`, an `index` dropped. `None` where the baseline holds no
/// page of the component-version carrying a version fact, or the newest it
/// holds is not under its version's own segment (the latest at a
/// `latest_version_segment`, I16).
fn named(
    spec: &Refspec<'_>,
    held: &[(&str, &str)],
    lookup: &dyn Lookup,
) -> Result<Option<Url>, BoxError> {
    let candidates = lookup.pages_with_facts(held)?;
    let Some((page, version)) = newest(candidates, lookup, true)? else {
        return Ok(None);
    };
    let Some(mut url) = prefix(&page, spec.component, &version) else {
        return Ok(None);
    };
    let (dir, file) = spec
        .rel_path
        .rsplit_once('/')
        .unwrap_or(("", spec.rel_path));
    let name = file.strip_suffix(".adoc").unwrap_or(file);
    {
        let mut segments = url.path_segments_mut().expect("an http URL has a path");
        segments.pop_if_empty();
        if spec.module != "ROOT" {
            segments.push(spec.module);
        }
        segments.extend(dir.split('/').filter(|d| !d.is_empty()));
        if name != "index" {
            segments.push(name);
        }
        segments.push("");
    }
    Ok(Some(url))
}

/// The URL of `page`, a page of `component` at `version`, cut after the
/// component's segment and a named version's (I16); the placeholders
/// Antora writes for a version its URLs omit, `""` and `master`, name none.
fn prefix(page: &Url, component: &str, version: &str) -> Option<Url> {
    let named = !matches!(version, "" | "master");
    let raw: Vec<&str> = page.path_segments()?.collect();
    let decoded = |s: &str| percent_decode_str(s).decode_utf8_lossy().into_owned();
    let at = (0..raw.len()).find(|&i| {
        decoded(raw[i]) == component
            && (!named || raw.get(i + 1).is_some_and(|s| decoded(s) == version))
    })?;
    let mut url = page.clone();
    url.set_query(None);
    url.set_fragment(None);
    url.set_path(&format!("/{}/", raw[..=at + usize::from(named)].join("/")));
    Some(url)
}

/// The page of `candidates` whose version Antora orders first (a version it
/// would call the component's latest), the lookup's first where two tie.
fn latest(candidates: Vec<Url>, lookup: &dyn Lookup) -> Result<Url, BoxError> {
    Ok(newest(candidates, lookup, false)?
        .expect("at least two candidates")
        .0)
}

/// [`latest`] with its version, `None` for no candidate; a page with no
/// version fact is passed over when `versioned`, else a bare one.
fn newest(
    candidates: Vec<Url>,
    lookup: &dyn Lookup,
    versioned: bool,
) -> Result<Option<(Url, String)>, BoxError> {
    let mut best: Option<(String, Url)> = None;
    for page in candidates {
        let facts = lookup.facts(&page)?;
        let version = facts
            .iter()
            .find(|(k, _)| k == VERSION)
            .map(|(_, v)| v.clone());
        if versioned && version.is_none() {
            continue;
        }
        let version = version.unwrap_or_default();
        let newer = best
            .as_ref()
            .is_none_or(|(held, _)| compare(&version, held).is_gt());
        if newer {
            best = Some((version, page));
        }
    }
    Ok(best.map(|(version, page)| (page, version)))
}

/// Antora's version order, `Greater` for the version it lists first: no
/// version (a bare one) before any, then a name that is no semantic version
/// in reverse alphabetical order, then semantic versions newest first. A
/// prerelease is not told apart from a release by its descriptor, which the
/// baseline does not hold: only a `-` suffix is lower than its release.
fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    // deficiency: KD14 Antora's latest is the first version not flagged
    // `prerelease` in its `antora.yml` (I20), which no page carries, so this
    // takes docs.fp.o's `rawhide` over `f44`: a versionless `fedora::x.adoc`
    // xref out of a partial build resolves to `fedora/rawhide/x/` where
    // Antora's is `fedora/latest/x/`. Names compare bytewise where Antora
    // compares them numerically and case-folded (`f9` above `f10`), and a
    // non-numeric semantic part ties with 0 where Antora sorts it low. The
    // baseline recording which version a component serves at `latest` (I5)
    // would close the first.
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
