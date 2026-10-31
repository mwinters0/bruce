//! `[[profile]]`: a declarative profile as the file defines it (D33), and
//! the rows a run records it as (D78).
//!
//! The loader refuses what the profile built from it could not run: a name
//! that is no namespace, a regex that does not compile or a `pattern` without
//! exactly one capture, a selector lol_html cannot stream, a `region` entry
//! that is not one compound selector. A name a compiled profile holds is
//! `cli`'s to refuse, this module naming none (D39).

use std::collections::BTreeSet;

use regex::Regex;
use serde::Deserialize;

use super::selector::selectors;

/// The profile value that picks one per page by its generator meta (D35).
pub const AUTO_PROFILE: &str = "auto";

/// A declarative profile (D33): what a site generator the compiled profiles do
/// not know writes into a page, found by selectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    /// The name a root's `profile` gives and the namespace of the profile's
    /// facts (D37).
    pub name: String,
    /// A regex matched against a page's generator meta under `auto`, which
    /// then picks this profile (D35); none leaves it to a root naming it.
    pub generator: Option<String>,
    /// The content region (D34), each entry one compound selector; empty
    /// takes `generic`'s.
    pub region: Vec<String>,
    /// A selector for the link that is the page's edit link, where the core's
    /// ranking earns none (D36).
    pub edit_link: Option<String>,
    /// The facts the profile reads, in file order.
    pub facts: Vec<Fact>,
}

/// One fact rule: the first element matching `selector` that gives a value
/// gives the fact `key` (D37).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// The fact's key without the namespace.
    pub key: String,
    /// The streaming CSS selector (D11) of the elements read.
    pub selector: String,
    /// The attribute read, kept as written; without one, the element's
    /// trimmed text.
    pub attr: Option<String>,
    /// A regex over the value whose one capture is the fact.
    pub pattern: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::config) struct RawProfile {
    name: String,
    generator: Option<String>,
    #[serde(default)]
    region: Vec<String>,
    edit_link: Option<String>,
    #[serde(default)]
    fact: Vec<RawFact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::config) struct RawFact {
    key: String,
    selector: String,
    attr: Option<String>,
    pattern: Option<String>,
}

/// The `[[profile]]` array validated, in the order the file writes it (D35),
/// `profile.<name>` leading each refusal and a name given twice refused.
pub(super) fn validate(raw: Vec<RawProfile>) -> Result<Vec<Definition>, String> {
    let mut defined: Vec<Definition> = Vec::with_capacity(raw.len());
    for profile in raw {
        let name = profile.name.clone();
        let definition =
            Definition::from_raw(profile).map_err(|e| format!("profile.{name}: {e}"))?;
        if defined.iter().any(|d| d.name == name) {
            return Err(format!("profile.{name}: the name of an earlier profile"));
        }
        defined.push(definition);
    }
    Ok(defined)
}

impl Definition {
    fn from_raw(raw: RawProfile) -> Result<Definition, String> {
        let name = raw.name.as_str();
        if !is_name(name) {
            return Err(format!(
                "{name:?} is not a profile name (a letter, then letters, digits, - _), which \
                 also begins its facts' keys"
            ));
        }
        if name == AUTO_PROFILE {
            return Err("`auto` picks a profile per page; it cannot be one".to_owned());
        }
        if let Some(generator) = &raw.generator {
            Regex::new(generator).map_err(|e| format!("generator: {e}"))?;
        }
        let region = selectors(&raw.region).map_err(|e| format!("region: {e}"))?;
        for entry in &region {
            compound(entry).map_err(|e| format!("region: {e}"))?;
        }
        if let Some(selector) = &raw.edit_link {
            streamed(selector).map_err(|e| format!("edit_link: {e}"))?;
        }
        let mut seen = BTreeSet::new();
        let mut facts = Vec::with_capacity(raw.fact.len());
        for (i, fact) in raw.fact.into_iter().enumerate() {
            let at = |e: String| format!("fact[{i}]: {e}");
            if !is_key(&fact.key) {
                return Err(at(format!(
                    "key: {:?} is not a fact key (letters, digits, - _)",
                    fact.key
                )));
            }
            if !seen.insert(fact.key.clone()) {
                return Err(at(format!(
                    "key: {:?} is the key of another fact",
                    fact.key
                )));
            }
            streamed(&fact.selector).map_err(|e| at(format!("selector: {e}")))?;
            if fact.attr.as_deref().is_some_and(|a| !is_attr(a)) {
                return Err(at(format!(
                    "attr: {:?} is not an attribute name",
                    fact.attr.unwrap_or_default()
                )));
            }
            if let Some(pattern) = &fact.pattern {
                let re = Regex::new(pattern).map_err(|e| at(format!("pattern: {e}")))?;
                if re.captures_len() != 2 {
                    return Err(at(format!(
                        "pattern: {pattern:?} holds {} capture groups; the value is the one",
                        re.captures_len() - 1
                    )));
                }
            }
            facts.push(Fact {
                key: fact.key,
                selector: fact.selector,
                attr: fact.attr,
                pattern: fact.pattern,
            });
        }
        Ok(Definition {
            name: raw.name,
            generator: raw.generator,
            region,
            edit_link: raw.edit_link,
            facts,
        })
    }

    /// The definition as the rows a run records (D78), keys under
    /// `profile.<name>.`: `generator` always, empty for none, `region` a row
    /// to an entry, `edit_link`, and each fact's `selector`, `attr` and
    /// `pattern` under `fact.<key>.`.
    pub fn rows(&self) -> Vec<(String, String)> {
        let row =
            |key: String, value: &str| (format!("profile.{}.{key}", self.name), value.to_owned());
        let mut rows = vec![row(
            "generator".to_owned(),
            self.generator.as_deref().unwrap_or(""),
        )];
        rows.extend(self.region.iter().map(|r| row("region".to_owned(), r)));
        if let Some(selector) = &self.edit_link {
            rows.push(row("edit_link".to_owned(), selector));
        }
        for fact in &self.facts {
            let key = &fact.key;
            rows.push(row(format!("fact.{key}.selector"), &fact.selector));
            if let Some(attr) = &fact.attr {
                rows.push(row(format!("fact.{key}.attr"), attr));
            }
            if let Some(pattern) = &fact.pattern {
                rows.push(row(format!("fact.{key}.pattern"), pattern));
            }
        }
        rows
    }
}

fn is_name(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && is_key(s)
}

fn is_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
}

fn is_attr(s: &str) -> bool {
    !s.is_empty()
        && !s
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "\"'>/=".contains(c))
}

/// `selector` as one selector `lol_html` streams (D11), a list of them
/// included.
fn streamed(selector: &str) -> Result<(), String> {
    if selector.trim().is_empty() {
        return Err("empty".to_owned());
    }
    selector
        .parse::<lol_html::Selector>()
        .map(drop)
        .map_err(|e| format!("{selector:?}: {e}"))
}

/// Whether `selector`, one of a list, is a compound selector: no combinator,
/// which the core extends with `[id]`, ` [id]` and ` a[name]` (D34).
fn compound(selector: &str) -> Result<(), String> {
    let (mut depth, mut quote, mut escaped) = (0u32, None, false);
    for c in selector.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match (c, quote) {
            ('\\', _) => escaped = true,
            (q, Some(open)) if q == open => quote = None,
            (_, Some(_)) => {}
            ('"' | '\'', None) => quote = Some(c),
            ('[' | '(', None) => depth += 1,
            (']' | ')', None) => depth = depth.saturating_sub(1),
            (c, None) if depth == 0 && (c.is_whitespace() || "> +~".contains(c)) => {
                return Err(format!(
                    "{selector:?} holds a combinator; a region entry is one compound selector, \
                     as `div.document`"
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
