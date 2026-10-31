//! What the run's severity map makes of the findings the HTML counts, lists
//! and ranks (D82). A page's and a repo's counts are of their links by mapped
//! level, and they rank by red then amber; a kind at *none* is no finding, so
//! it is in no count, no ranking and no list, and a page whose only findings
//! are at *none* has no row. The JSON counts and orders by outcome, the levels
//! being derivable from its `policy`, so these are the HTML's own reading of
//! the same tables.

use std::cmp::Reverse;
use std::collections::HashMap;

use super::super::document::Digest;
use super::guide::Entry;
use super::markup::{Row, counted, num, rows};
use super::severity::mark_before;
use crate::config::{Severity, SeverityMap};
use crate::store::DOUBLED_SLASH;

/// The kinds of link a count words, in the order a summary lists them within
/// a level.
#[derive(Clone, Copy)]
enum Kind {
    Broken,
    /// Broken through a link to a URL that doubles a slash (D88).
    Through,
    Unverifiable,
    Fragment,
}

const KINDS: [Kind; 4] = [
    Kind::Broken,
    Kind::Through,
    Kind::Unverifiable,
    Kind::Fragment,
];

/// The counted levels, the least first, indexing [`Counts`].
const LEVELS: [Severity; 3] = [Severity::Pedantic, Severity::Attention, Severity::Broken];

/// A page's or a repo's links, by kind and by the level the map gives them.
/// A link at *none* is in no cell.
#[derive(Clone, Copy, Default)]
pub(super) struct Counts([[i64; 3]; 4]);

impl Counts {
    fn add(&mut self, kind: Kind, level: Severity, n: i64) {
        if let Some(at) = LEVELS.iter().position(|l| *l == level) {
            self.0[kind as usize][at] += n;
        }
    }

    fn plus(&mut self, other: &Counts) {
        for (mine, theirs) in self.0.iter_mut().zip(&other.0) {
            for (mine, theirs) in mine.iter_mut().zip(theirs) {
                *mine += theirs;
            }
        }
    }

    fn at(&self, kind: Kind, level: Severity) -> i64 {
        LEVELS
            .iter()
            .position(|l| *l == level)
            .map_or(0, |at| self.0[kind as usize][at])
    }

    /// The links at `level`, of every kind.
    fn of_level(&self, level: Severity) -> i64 {
        KINDS.iter().map(|k| self.at(*k, level)).sum()
    }

    fn is_empty(&self) -> bool {
        LEVELS.iter().all(|l| self.of_level(*l) == 0)
    }

    /// The rank: the red links, then the amber ones.
    fn rank(&self) -> (Reverse<i64>, Reverse<i64>) {
        (
            Reverse(self.of_level(Severity::Broken)),
            Reverse(self.of_level(Severity::Attention)),
        )
    }
}

/// A crawled page holding a finding, with its row of `pages`.
pub(super) struct Page<'a> {
    pub(super) row: Row<'a>,
    pub(super) counts: Counts,
    /// Its findings that are new since the compared run (D75).
    pub(super) new_links: i64,
}

/// A repo with the findings of its pages, with its row of `by_repo`.
pub(super) struct Repo<'a> {
    pub(super) row: Row<'a>,
    pub(super) counts: Counts,
    pub(super) new_links: i64,
    /// Its pages that hold a finding.
    pub(super) pages: usize,
}

/// The pages holding a finding, ranked by their red links then their amber
/// ones, a tie by URL. A page's links are `broken`'s and `errors`' rows at the
/// level of their outcome and reason; the links broken through it are the
/// `through` of its row of `pages`, at the level of a doubled slash.
pub(super) fn pages(d: &Digest) -> Vec<Page<'_>> {
    let mut found: HashMap<&str, (Counts, i64)> = HashMap::new();
    for r in rows(&d.broken) {
        let kind = if r.text("outcome") == Some("broken_anchor") {
            Kind::Fragment
        } else {
            Kind::Broken
        };
        let level = d.severity.of_link(r.text("outcome"), r.text("reason"));
        let (counts, new) = found.entry(r.text("page").unwrap_or("")).or_default();
        counts.add(kind, level, 1);
        *new += i64::from(level.is_finding() && r.is_new());
    }
    for r in rows(&d.errors) {
        let level = d.severity.of_link(Some("error"), r.text("reason"));
        let (counts, new) = found.entry(r.text("page").unwrap_or("")).or_default();
        counts.add(Kind::Unverifiable, level, 1);
        *new += i64::from(level.is_finding() && r.is_new());
    }
    let through = d.severity.of_link(Some("broken_url"), Some(DOUBLED_SLASH));
    let mut out: Vec<Page> = rows(&d.pages)
        .filter_map(|row| {
            let (mut counts, new_links) = found
                .get(row.text("page").unwrap_or(""))
                .copied()
                .unwrap_or_default();
            counts.add(Kind::Through, through, row.int("through"));
            (!counts.is_empty()).then_some(Page {
                row,
                counts,
                new_links,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a.counts
            .rank()
            .cmp(&b.counts.rank())
            .then_with(|| a.row.text("page").cmp(&b.row.text("page")))
    });
    out
}

/// The repos, those holding a finding first and ranked as the pages are, a
/// tie by name and the repo with none last; then the repos that hold none.
pub(super) fn repos<'a>(d: &'a Digest, pages: &[Page<'_>]) -> Vec<Repo<'a>> {
    let mut of_repo: HashMap<Option<&str>, (Counts, i64, usize)> = HashMap::new();
    for p in pages {
        let (counts, new, n) = of_repo.entry(p.row.text("source_repo")).or_default();
        counts.plus(&p.counts);
        *new += p.new_links;
        *n += 1;
    }
    let mut out: Vec<Repo> = rows(&d.by_repo)
        .map(|row| {
            let (counts, new_links, pages) =
                of_repo.get(&row.text("repo")).copied().unwrap_or_default();
            Repo {
                row,
                counts,
                new_links,
                pages,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        let key = |r: &Repo| (r.pages == 0, r.counts.rank(), r.row.text("repo").is_none());
        key(a)
            .cmp(&key(b))
            .then_with(|| a.row.text("repo").cmp(&b.row.text("repo")))
    });
    out
}

/// A page's or a repo's summary count: its links by level, red first, each
/// kind of a level behind the level's dot and a kind of none left out, and no
/// total (D82). Under D76's map: the broken links, those broken through a
/// link to a URL that doubles a slash (D88), the unverifiable ones, then the
/// missing fragments.
pub(super) fn tally(counts: &Counts) -> String {
    let mut parts = Vec::new();
    for level in LEVELS.into_iter().rev() {
        for kind in KINDS {
            let n = counts.at(kind, level);
            if n == 0 {
                continue;
            }
            let words = match kind {
                Kind::Broken => counted(n, "broken link"),
                Kind::Through => {
                    format!("{} broken through it{}", num(n), Entry::DoubledSlash.info())
                }
                Kind::Unverifiable => format!("{} unverifiable", num(n)),
                Kind::Fragment => counted(n, "missing fragment"),
            };
            parts.push(format!(
                "<span class=\"cn\">{}{words}</span>",
                mark_before(level)
            ));
        }
    }
    parts.join(", ")
}

/// Whether the map makes the link a finding, a link at *none* being in no
/// list.
pub(super) fn listed(sev: &SeverityMap, outcome: Option<&str>, reason: Option<&str>) -> bool {
    sev.of_link(outcome, reason).is_finding()
}

#[cfg(test)]
mod tests;
