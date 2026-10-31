//! The `policy` rows that differ between the two runs a digest compares (D91),
//! which the settings off their defaults cannot show: they say what one run
//! configured, not a row the other run held and this one lacks.
//!
//! Rows are read as the runs recorded them, whatever trial judges the digest
//! (D84). A declared profile's rows are compared among the profiles either
//! run's pages were extracted by, and its source rules by their place in each
//! run's list (D93), aligned so a rule inserted or removed is named alone
//! and not every rule after it.

use std::collections::{BTreeMap, BTreeSet};

use duckdb::Connection;

use crate::config::Entry;
use crate::report::error::Error;
use crate::store::RunId;

/// One row that differs: held by the compared run alone (`from`), by the
/// digest's run alone (`to`), or by both with another value, or, for a source
/// rule, as the same rule's field at another place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub from: Option<Entry>,
    pub to: Option<Entry>,
}

impl Change {
    /// Whether a comparison judges by the row (D81): neither an asking setting
    /// nor a profile's definition, so a difference in it changed no verdict
    /// the comparison shows, both runs being judged by one policy.
    pub fn judges(&self) -> bool {
        self.listed().is_judged()
    }

    /// The row the change is listed by: the digest's run's where it holds one.
    fn listed(&self) -> &Entry {
        self.to
            .as_ref()
            .or(self.from.as_ref())
            .expect("a change has a side")
    }
}

/// The rows of `from` and of `run` that differ, in the order they are listed.
pub(super) fn changes(conn: &Connection, from: RunId, run: RunId) -> Result<Vec<Change>, Error> {
    let used = used_profiles(conn, from, run)?;
    Ok(differences(
        &recorded(conn, from)?,
        &recorded(conn, run)?,
        &used,
    ))
}

/// The profiles the pages of either run were extracted by (`pages.profile`).
fn used_profiles(conn: &Connection, a: RunId, b: RunId) -> Result<BTreeSet<String>, Error> {
    let mut stmt = conn.prepare("SELECT DISTINCT profile FROM pages WHERE run_id IN (?, ?)")?;
    let used = stmt
        .query_map([a.0, b.0], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(used)
}

/// The rows `run` recorded (D78), not the policy in force.
fn recorded(conn: &Connection, run: RunId) -> Result<Vec<Entry>, Error> {
    let mut stmt = conn.prepare("SELECT host, key, value FROM policy WHERE run_id = ?")?;
    let rows = stmt.query_map([run.0], |row| {
        Ok(Entry {
            host: row.get(0)?,
            key: row.get(1)?,
            value: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// A source rule's row, split: the profile, the rule's place and its field.
fn source_row(e: &Entry) -> Option<(&str, usize, &str)> {
    if e.host.is_some() {
        return None;
    }
    let rest = e.key.strip_prefix("profile.")?;
    let (name, rest) = rest.split_once('.')?;
    let (place, field) = rest.strip_prefix("source.")?.split_once('.')?;
    Some((name, place.parse().ok()?, field))
}

/// The profile a row defines, if it is one of a declared profile's.
fn profile_of(e: &Entry) -> Option<&str> {
    if !e.is_profile() {
        return None;
    }
    e.key.strip_prefix("profile.")?.split_once('.').map(|p| p.0)
}

/// What differs between two runs' rows, `used` the profiles whose rows count.
fn differences(from: &[Entry], to: &[Entry], used: &BTreeSet<String>) -> Vec<Change> {
    let counts = |e: &&Entry| profile_of(e).is_none_or(|name| used.contains(name));
    let plain = |e: &&Entry| counts(e) && source_row(e).is_none();
    let (from_plain, to_plain): (Vec<&Entry>, Vec<&Entry>) = (
        from.iter().filter(plain).collect(),
        to.iter().filter(plain).collect(),
    );
    let mut out = plain_changes(&from_plain, &to_plain);
    let rules = |rows: &[Entry]| -> BTreeMap<String, Vec<Rule>> {
        let mut by_profile: BTreeMap<String, BTreeMap<usize, BTreeMap<String, String>>> =
            BTreeMap::new();
        for e in rows.iter().filter(counts) {
            if let Some((name, place, field)) = source_row(e) {
                by_profile
                    .entry(name.to_owned())
                    .or_default()
                    .entry(place)
                    .or_default()
                    .insert(field.to_owned(), e.value.clone());
            }
        }
        by_profile
            .into_iter()
            .map(|(name, rules)| {
                let list = rules
                    .into_iter()
                    .map(|(place, fields)| Rule { place, fields })
                    .collect();
                (name, list)
            })
            .collect()
    };
    let (before, after) = (rules(from), rules(to));
    let names: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    for name in names {
        let none = Vec::new();
        out.extend(rule_changes(
            name,
            before.get(name).unwrap_or(&none),
            after.get(name).unwrap_or(&none),
        ));
    }
    out.sort_by(|a, b| a.listed().listed_order(b.listed()));
    out
}

/// The rows other than source rules: those of a (host, key) held by one side
/// alone are removed or added, a key whose one value changed is a change, and
/// a key holding several values (`schemes`, `region`) lists the values it lost
/// and gained apart.
fn plain_changes(from: &[&Entry], to: &[&Entry]) -> Vec<Change> {
    let keys: BTreeSet<(&Option<String>, &String)> =
        from.iter().chain(to).map(|e| (&e.host, &e.key)).collect();
    let mut out = Vec::new();
    for (host, key) in keys {
        let held = |rows: &[&Entry]| -> Vec<Entry> {
            rows.iter()
                .filter(|e| &e.host == host && &e.key == key)
                .map(|e| (*e).clone())
                .collect()
        };
        let (before, after) = (held(from), held(to));
        let lost: Vec<&Entry> = before.iter().filter(|e| !after.contains(e)).collect();
        let gained: Vec<&Entry> = after.iter().filter(|e| !before.contains(e)).collect();
        if let ([lost], [gained]) = (lost.as_slice(), gained.as_slice()) {
            out.push(Change {
                from: Some((*lost).clone()),
                to: Some((*gained).clone()),
            });
            continue;
        }
        out.extend(lost.into_iter().map(|e| Change {
            from: Some(e.clone()),
            to: None,
        }));
        out.extend(gained.into_iter().map(|e| Change {
            from: None,
            to: Some(e.clone()),
        }));
    }
    out
}

/// A source rule: its place in its run's list, and its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    place: usize,
    fields: BTreeMap<String, String>,
}

impl Rule {
    fn entry(&self, profile: &str, field: &str, value: &str) -> Entry {
        Entry {
            host: None,
            key: format!("profile.{profile}.source.{}.{field}", self.place),
            value: value.to_owned(),
        }
    }
}

/// The changes between two lists of one profile's source rules. The rules
/// equal in both, longest such run in order, align the lists; between two
/// aligned rules the rules removed and those added are paired in order, a pair
/// naming the fields that changed, and the rest are removed or added alone.
fn rule_changes(profile: &str, from: &[Rule], to: &[Rule]) -> Vec<Change> {
    let same = |i: usize, j: usize| from[i].fields == to[j].fields;
    // The longest common subsequence of equal rules, by dynamic programming.
    let mut lcs = vec![vec![0usize; to.len() + 1]; from.len() + 1];
    for i in (0..from.len()).rev() {
        for j in (0..to.len()).rev() {
            lcs[i][j] = if same(i, j) {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    let (mut removed, mut added): (Vec<&Rule>, Vec<&Rule>) = (Vec::new(), Vec::new());
    loop {
        let matched = i < from.len() && j < to.len() && same(i, j);
        if matched || (i == from.len() && j == to.len()) {
            out.extend(gap(profile, &removed, &added));
            (removed, added) = (Vec::new(), Vec::new());
            if !matched {
                break;
            }
            (i, j) = (i + 1, j + 1);
        } else if i < from.len() && (j == to.len() || lcs[i + 1][j] >= lcs[i][j + 1]) {
            removed.push(&from[i]);
            i += 1;
        } else {
            added.push(&to[j]);
            j += 1;
        }
    }
    out
}

/// The rules removed and added between two aligned ones: paired in order, the
/// fields that differ named, then the unpaired ones alone.
fn gap(profile: &str, removed: &[&Rule], added: &[&Rule]) -> Vec<Change> {
    let mut out = Vec::new();
    for k in 0..removed.len().max(added.len()) {
        match (removed.get(k), added.get(k)) {
            (Some(was), Some(now)) => {
                let fields: BTreeSet<&String> =
                    was.fields.keys().chain(now.fields.keys()).collect();
                for field in fields {
                    let (a, b) = (was.fields.get(field), now.fields.get(field));
                    if a != b {
                        out.push(Change {
                            from: a.map(|v| was.entry(profile, field, v)),
                            to: b.map(|v| now.entry(profile, field, v)),
                        });
                    }
                }
            }
            (Some(was), None) => out.extend(was.fields.iter().map(|(f, v)| Change {
                from: Some(was.entry(profile, f, v)),
                to: None,
            })),
            (None, Some(now)) => out.extend(now.fields.iter().map(|(f, v)| Change {
                from: None,
                to: Some(now.entry(profile, f, v)),
            })),
            (None, None) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests;
