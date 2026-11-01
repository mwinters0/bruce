//! What the run's severity map makes of the findings a rendering counts, lists
//! and ranks (D82). A page's and a repo's counts are of their links by mapped
//! level, and they rank by red then amber; a kind at *none* is no finding, so
//! it is in no count, no ranking and no list, and a page whose only findings
//! are at *none* has no row. A page's findings are its broken links and its
//! errors, red first. The JSON counts and orders by outcome, the levels being
//! derivable from its `policy`, so these are the model's own reading of the
//! same tables.

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap};

use super::super::document::Digest;
use super::row::{Row, rows};
use crate::config::Reason;
use crate::config::{Severity, SeverityMap};
use crate::report::table::Table;

/// The kinds of link a count words, in the order a summary lists them within
/// a level.
#[derive(Clone, Copy)]
pub(in crate::report::digest) enum Kind {
    Broken,
    /// Broken through a link to a URL that doubles a slash (D88).
    Through,
    Unverifiable,
    Fragment,
}

impl Kind {
    /// The kind as a view names it.
    pub(in crate::report::digest) fn name(self) -> &'static str {
        match self {
            Kind::Broken => "broken",
            Kind::Through => "through",
            Kind::Unverifiable => "unverifiable",
            Kind::Fragment => "fragment",
        }
    }
}

pub(in crate::report::digest) const KINDS: [Kind; 4] = [
    Kind::Broken,
    Kind::Through,
    Kind::Unverifiable,
    Kind::Fragment,
];

/// The counted levels, the least first, indexing [`Counts`].
pub(in crate::report::digest) const LEVELS: [Severity; 3] =
    [Severity::Pedantic, Severity::Attention, Severity::Broken];

/// A page's or a repo's links, by kind and by the level the map gives them,
/// with the reasons that put a cell's links at a level other than their
/// outcome's own. A link at *none* is in no cell.
#[derive(Clone, Default)]
pub(in crate::report::digest) struct Counts {
    cells: [[i64; 3]; 4],
    moved: [[BTreeSet<String>; 3]; 4],
}

impl Counts {
    /// `n` links of `kind` at `level`, `moved_by` being the reason that put
    /// them off their outcome's level, if one did ([`SeverityMap::moved_by`]).
    pub(in crate::report::digest) fn add(
        &mut self,
        kind: Kind,
        level: Severity,
        n: i64,
        moved_by: Option<&str>,
    ) {
        if let Some(at) = LEVELS.iter().position(|l| *l == level) {
            self.cells[kind as usize][at] += n;
            if let Some(reason) = moved_by {
                self.moved[kind as usize][at].insert(reason.to_owned());
            }
        }
    }

    fn plus(&mut self, other: &Counts) {
        for (mine, theirs) in self.cells.iter_mut().zip(&other.cells) {
            for (mine, theirs) in mine.iter_mut().zip(theirs) {
                *mine += theirs;
            }
        }
        for (mine, theirs) in self.moved.iter_mut().zip(&other.moved) {
            for (mine, theirs) in mine.iter_mut().zip(theirs) {
                mine.extend(theirs.iter().cloned());
            }
        }
    }

    pub(in crate::report::digest) fn at(&self, kind: Kind, level: Severity) -> i64 {
        LEVELS
            .iter()
            .position(|l| *l == level)
            .map_or(0, |at| self.cells[kind as usize][at])
    }

    /// The reasons that put the links of `kind` at `level` off their
    /// outcome's level, in order; none where none did.
    pub(in crate::report::digest) fn reasons(&self, kind: Kind, level: Severity) -> Vec<&str> {
        let Some(at) = LEVELS.iter().position(|l| *l == level) else {
            return Vec::new();
        };
        self.moved[kind as usize][at]
            .iter()
            .map(String::as_str)
            .collect()
    }

    /// The links at `level`, of every kind.
    pub(in crate::report::digest) fn of_level(&self, level: Severity) -> i64 {
        KINDS.iter().map(|k| self.at(*k, level)).sum()
    }

    pub(in crate::report::digest) fn is_empty(&self) -> bool {
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

/// A link of a page that the map makes a finding.
pub(in crate::report::digest) struct Finding<'a> {
    pub(in crate::report::digest) row: Row<'a>,
    pub(in crate::report::digest) level: Severity,
    pub(in crate::report::digest) is_new: bool,
    /// The row is of `errors`, a link that could not be verified, not of
    /// `broken`.
    pub(in crate::report::digest) error: bool,
}

/// A crawled page holding a finding, with its row of `pages`.
pub(in crate::report::digest) struct Page<'a> {
    pub(in crate::report::digest) row: Row<'a>,
    pub(in crate::report::digest) counts: Counts,
    /// Its findings that are new since the compared run (D75).
    pub(in crate::report::digest) new_links: i64,
    /// Its findings: the red links together, an unverified link after the
    /// broken URLs, then the amber ones.
    pub(in crate::report::digest) findings: Vec<Finding<'a>>,
    /// Some broken link of the page, a finding or not, has a hint (D65).
    pub(in crate::report::digest) hinted: bool,
}

/// A repo with the findings of its pages, with its row of `by_repo`.
pub(in crate::report::digest) struct Repo<'a> {
    pub(in crate::report::digest) row: Row<'a>,
    pub(in crate::report::digest) counts: Counts,
    pub(in crate::report::digest) new_links: i64,
    /// Its pages that hold a finding.
    pub(in crate::report::digest) pages: usize,
}

/// `table`'s rows by their `page`.
fn by_page(table: &Table) -> HashMap<&str, Vec<Row<'_>>> {
    let mut out: HashMap<&str, Vec<Row>> = HashMap::new();
    for row in rows(table) {
        out.entry(row.text("page").unwrap_or(""))
            .or_default()
            .push(row);
    }
    out
}

/// The pages holding a finding, ranked by their red links then their amber
/// ones, a tie by URL. A page's links are `broken`'s and `errors`' rows at the
/// level of their outcome and reason; the links broken through it are the
/// `through` of its row of `pages`, at the level of a doubled slash.
pub(in crate::report::digest) fn pages(d: &Digest) -> Vec<Page<'_>> {
    let (broken, errors) = (by_page(&d.broken), by_page(&d.errors));
    let through = d
        .severity
        .of_link(Some("broken_url"), Some(Reason::DoubledSlash.as_str()));
    let mut out: Vec<Page> = rows(&d.pages)
        .filter_map(|row| {
            let url = row.text("page").unwrap_or("");
            let (links, failed) = (broken.get(url), errors.get(url));
            let mut counts = Counts::default();
            let mut new_links = 0;
            let mut findings = Vec::new();
            let broken_links = links.into_iter().flatten().map(|r| (*r, false));
            let errored = failed.into_iter().flatten().map(|r| (*r, true));
            for (r, error) in broken_links.chain(errored) {
                let outcome = if error {
                    Some("error")
                } else {
                    r.text("outcome")
                };
                let kind = match outcome {
                    _ if error => Kind::Unverifiable,
                    Some("broken_anchor") => Kind::Fragment,
                    _ => Kind::Broken,
                };
                let level = d.severity.of_link(outcome, r.text("reason"));
                let moved = d
                    .severity
                    .moved_by(outcome.unwrap_or("broken_url"), r.text("reason"));
                counts.add(kind, level, 1, moved);
                if level.is_finding() {
                    let is_new = r.is_new();
                    new_links += i64::from(is_new);
                    findings.push(Finding {
                        row: r,
                        level,
                        is_new,
                        error,
                    });
                }
            }
            counts.add(Kind::Through, through, row.int("through"), None);
            findings.sort_by_key(|f| Reverse(f.level));
            (!counts.is_empty()).then_some(Page {
                hinted: links
                    .into_iter()
                    .flatten()
                    .any(|r| r.text("hint").is_some()),
                row,
                counts,
                new_links,
                findings,
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
pub(in crate::report::digest) fn repos<'a>(d: &'a Digest, pages: &[Page<'_>]) -> Vec<Repo<'a>> {
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
                of_repo.get(&row.text("repo")).cloned().unwrap_or_default();
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

/// Whether the map makes the link a finding, a link at *none* being in no
/// list.
pub(in crate::report::digest) fn listed(
    sev: &SeverityMap,
    outcome: Option<&str>,
    reason: Option<&str>,
) -> bool {
    sev.of_link(outcome, reason).is_finding()
}

#[cfg(test)]
mod tests;
