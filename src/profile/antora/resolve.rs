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
/// module and path is taken ([`newest`]); where none has them, the URL the
/// refspec names ([`named`]).
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
    // deficiency: KD18 Antora resolves a versionless refspec in the
    // component's latest version alone (I21); this takes the latest version
    // among the pages holding the module and path, so a page the latest lacks
    // and an older version has resolves there, an `ok` where production's xref
    // is unresolved. Choosing the version from the component's pages first,
    // then the page in it or `named`'s URL, is the fix (D55).
    let page = match (spec.version, candidates.as_slice()) {
        (_, []) => named(&spec, &held, lookup)?,
        (Some(_), [page, ..]) | (None, [page]) => Some(page.clone()),
        (None, _) => Some(latest(candidates, spec.component, lookup)?),
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
/// page of the component-version carrying a version fact, or where the page
/// [`newest`] picks gives no prefix (KD19). A version served
/// under another name (the latest at a `latest_version_segment`, I5) is named
/// under that segment, whether or not the refspec names the version (D57).
fn named(
    spec: &Refspec<'_>,
    held: &[(&str, &str)],
    lookup: &dyn Lookup,
) -> Result<Option<Url>, BoxError> {
    let candidates = lookup.pages_with_facts(held)?;
    let Some(found) = newest(candidates, spec.component, lookup, true)? else {
        return Ok(None);
    };
    let Some(mut url) = prefix(&found.page, spec.component, &found.version, found.elsewhere) else {
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
/// With `elsewhere` (see [`Found`]) the segment after the component's is cut
/// whatever it is, the version being served under another name (I5).
fn prefix(page: &Url, component: &str, version: &str, elsewhere: bool) -> Option<Url> {
    let named = !matches!(version, "" | "master");
    // deficiency: KD19 With a bare version and no `elsewhere`, the component's
    // segment must be followed by a non-empty one, so a page at the
    // component's root (`/ops/`) gives no prefix and `named` none, though
    // `/ops/` is the prefix. Bare versions tie in `compare`, so the lookup's
    // first page decides, and an xref to a page the baseline lacks stays
    // `unresolved_xref` or not by row order.
    let raw: Vec<&str> = page.path_segments()?.collect();
    let at = segment_after_component(&raw, component, (!elsewhere && named).then_some(version))?;
    let mut url = page.clone();
    url.set_query(None);
    url.set_fragment(None);
    url.set_path(&format!("/{}/", raw[..=at + usize::from(named)].join("/")));
    Some(url)
}

/// The index of the first segment of `raw` that is `component` and, given
/// `version`, followed by it, else followed by a non-empty segment; decoded,
/// as Antora writes them escaped.
fn segment_after_component(raw: &[&str], component: &str, version: Option<&str>) -> Option<usize> {
    let decoded = |s: &str| percent_decode_str(s).decode_utf8_lossy().into_owned();
    (0..raw.len()).find(|&i| {
        decoded(raw[i]) == component
            && match version {
                Some(version) => raw.get(i + 1).is_some_and(|s| decoded(s) == version),
                None => raw.get(i + 1).is_some_and(|s| !s.is_empty()),
            }
    })
}

/// A page of a component with the version it carries.
struct Found {
    page: Url,
    version: String,
    /// The page sits under the component but not under its version's own
    /// segment: the version is served under another name, as docs.fp.o
    /// serves its latest at `latest` (I5, I16).
    elsewhere: bool,
}

/// The page of `candidates`, pages of `component`, whose version is its
/// latest: the one version served off its own segment, since a site serves
/// only its latest so (I5); with none or several, the version Antora orders
/// first (I20), the lookup's first where two tie.
fn latest(candidates: Vec<Url>, component: &str, lookup: &dyn Lookup) -> Result<Url, BoxError> {
    Ok(newest(candidates, component, lookup, false)?
        .expect("at least two candidates")
        .page)
}

/// [`latest`] with its version, `None` for no candidate; a page with no
/// version fact is passed over when `versioned`, else a bare one.
fn newest(
    candidates: Vec<Url>,
    component: &str,
    lookup: &dyn Lookup,
    versioned: bool,
) -> Result<Option<Found>, BoxError> {
    let mut all: Vec<Found> = Vec::new();
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
        let elsewhere = !matches!(version.as_str(), "" | "master") && {
            let raw: Vec<&str> = page.path_segments().into_iter().flatten().collect();
            segment_after_component(&raw, component, Some(&version)).is_none()
                && segment_after_component(&raw, component, None).is_some()
        };
        all.push(Found {
            page,
            version,
            elsewhere,
        });
    }
    let mut served = all.iter().filter(|f| f.elsewhere);
    let only = served
        .next()
        .filter(|first| served.all(|f| f.version == first.version))
        .map(|first| first.version.clone());
    if let Some(only) = only {
        return Ok(all.into_iter().find(|f| f.elsewhere && f.version == only));
    }
    let mut best: Option<Found> = None;
    for found in all {
        if best
            .as_ref()
            .is_none_or(|held| compare(&found.version, &held.version).is_gt())
        {
            best = Some(found);
        }
    }
    Ok(best)
}

/// Antora's version order, `Greater` for the version it lists first (I20): no
/// version (a bare one) before any, then a name that is no semantic version
/// in reverse alphabetical order ([`collate`]), then semantic versions newest
/// first. A prerelease is not told apart from a release by its descriptor,
/// which the baseline does not hold: only a `-` suffix is lower than its
/// release.
fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    // deficiency: KD14 Antora's latest is the first version not flagged
    // `prerelease` in its `antora.yml` (I20), which no page carries. This
    // takes the one version served off its own segment (I5), and with none
    // or several this order, so a site serving its latest at its own name
    // beside a prerelease (`2.0` and `3.0-beta`) resolves a versionless xref
    // to the prerelease where Antora's is the release.
    use std::cmp::Ordering;
    match (a.is_empty(), b.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        _ => {}
    }
    match (semver(a), semver(b)) {
        (Some(a), Some(b)) => a
            .numbers
            .cmp(&b.numbers)
            .then(a.release.cmp(&b.release))
            .then_with(|| collate(a.pre, b.pre)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => collate(a, b),
    }
}

/// `localeCompare(…, 'en', {numeric: true})` as far as version names need it:
/// digit runs by value, the rest case-folded, and a lowercase letter before
/// its capital where nothing else differs.
fn collate(a: &str, b: &str) -> std::cmp::Ordering {
    let mut chunks = (chunks(a), chunks(b));
    loop {
        match (chunks.0.next(), chunks.1.next()) {
            (None, None) => return b.cmp(a),
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) => {
                let digits = |s: &str| s.starts_with(|c: char| c.is_ascii_digit());
                let order = if digits(x) && digits(y) {
                    let (x, y) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
                    x.len().cmp(&y.len()).then_with(|| x.cmp(y))
                } else {
                    x.to_lowercase().cmp(&y.to_lowercase())
                };
                if order.is_ne() {
                    return order;
                }
            }
        }
    }
}

/// `s` cut into runs of ASCII digits and runs of anything else.
fn chunks(s: &str) -> impl Iterator<Item = &str> {
    let mut rest = s;
    std::iter::from_fn(move || {
        let digit = rest.chars().next()?.is_ascii_digit();
        let end = rest
            .find(|c: char| c.is_ascii_digit() != digit)
            .unwrap_or(rest.len());
        let (chunk, tail) = rest.split_at(end);
        rest = tail;
        Some(chunk)
    })
}

/// A version that begins with a number (or `v` and a number) and has a `.` or
/// is one number, which is what Antora calls semantic.
struct Semver<'a> {
    /// The three parts, a part that is no number lower than any (`None`).
    numbers: [Option<u64>; 3],
    /// No `-` suffix.
    release: bool,
    pre: &'a str,
}

fn semver(version: &str) -> Option<Semver<'_>> {
    let rest = version.strip_prefix('v').unwrap_or(version);
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    if !rest.contains('.') && rest.parse::<u64>().is_err() {
        return None;
    }
    let (numbers, pre) = rest.split_once('-').unwrap_or((rest, ""));
    let mut parts = numbers.split('.');
    let mut next = || parts.next().map_or(Some(0), |n| n.parse::<u64>().ok());
    Some(Semver {
        numbers: [next(), next(), next()],
        release: pre.is_empty(),
        pre,
    })
}

#[cfg(test)]
mod tests;
