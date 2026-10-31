//! What is new since the compared run (D75): a `new` column on each section a
//! reader acts on, true for a row the compared run's same section lacked (a
//! repo for one whose pages held no broken link or error then), and
//! on the pages and repos a `new_links` count of the new links among theirs.
//! A broken link is new where section 3 lists it, so the two agree. The
//! sections the HTML lists through the severity map (`pages`, `by_repo`,
//! `errors`, `broken_targets`, `redirects`) also have `new_listed`, true for a
//! row that is a finding under the map and was none in the compared run's
//! section read through the same map, which the HTML marks from. With nothing
//! to compare with every such cell is null: nothing is known to be new.

use std::collections::{HashMap, HashSet};

use duckdb::Connection;

use super::document::Digest;
use super::sections;
use crate::config::{Reason, SeverityMap};
use crate::report::error::Error;
use crate::report::table::{Cell, Table};

/// A row's key: its cells in the key's columns, as text.
type Key = Vec<Option<String>>;

/// A link's key, as `diff` and section 3 key one (D74).
const LINK: &[&str] = &["page", "target", "fragment"];

/// The sections the HTML lists through the severity map, which have
/// `new_listed`.
fn listed_sections(d: &mut Digest) -> [&mut Table; 5] {
    [
        &mut d.pages,
        &mut d.by_repo,
        &mut d.errors,
        &mut d.broken_targets,
        &mut d.redirects,
    ]
}

/// Adds `new`, `new_listed` on the sections the map lists and `new_links` on
/// the pages and repos, to `d`'s sections.
pub(super) fn mark(conn: &Connection, d: &mut Digest) -> Result<(), Error> {
    let Some(from) = d.compared.as_ref().map(|c| c.run.id) else {
        for t in [
            &mut d.pages,
            &mut d.by_repo,
            &mut d.broken,
            &mut d.errors,
            &mut d.broken_targets,
            &mut d.redirects,
            &mut d.orphans,
            &mut d.unlisted,
            &mut d.sitemap_entries,
        ] {
            push(t, "new", vec![Cell::Null; t.rows.len()]);
        }
        for t in [&mut d.pages, &mut d.by_repo] {
            push(t, "new_links", vec![Cell::Null; t.rows.len()]);
        }
        for t in listed_sections(d) {
            push(t, "new_listed", vec![Cell::Null; t.rows.len()]);
        }
        return Ok(());
    };
    let new_breakage = keys(&d.new_breakage, LINK);
    let flags = column(&d.broken, LINK, |k| new_breakage.contains(k));
    push(&mut d.broken, "new", flags);
    let sev = d.severity.clone();
    let then_errors = sections::errors(conn, from)?;
    absent(&mut d.errors, &then_errors, LINK);
    absent_listed(&mut d.errors, &then_errors, LINK, |r| {
        sev.of_link(Some("error"), r.text("reason")).is_finding()
    });
    // The compared run's counts replace the digest's own, every section that
    // reads those being built by now.
    let counts = sections::page_counts(conn, from)?;
    let then_pages = sections::pages(conn, from, counts)?;
    absent(&mut d.pages, &then_pages, &["page"]);
    let then_broken = sections::broken(conn, from)?;
    let (now, then) = (
        listed_pages(&sev, &d.broken, &d.errors, &d.pages),
        listed_pages(&sev, &then_broken, &then_errors, &then_pages),
    );
    let flags = column(&d.pages, &["page"], |k| {
        let page = k[0].clone().unwrap_or_default();
        now.contains(&page) && !then.contains(&page)
    });
    push(&mut d.pages, "new_listed", flags);
    let then_repos = repos_of(&then_pages, &then);
    let now_repos = repos_of(&d.pages, &now);
    let by_repo = sections::by_repo(conn, from, counts)?;
    let then_targets = sections::broken_targets(conn, from)?;
    absent(
        &mut d.broken_targets,
        &then_targets,
        &["target", "fragment"],
    );
    absent_listed(
        &mut d.broken_targets,
        &then_targets,
        &["target", "fragment"],
        |r| {
            sev.of_link(r.text("outcome"), r.text("reason"))
                .is_finding()
        },
    );
    let then_redirects = sections::redirects(conn, from)?;
    absent(&mut d.redirects, &then_redirects, &["landing"]);
    absent_listed(&mut d.redirects, &then_redirects, &["landing"], |r| {
        sev.of_redirect(r.text("reason")).is_finding()
    });
    absent(&mut d.orphans, &sections::orphans(conn, from)?, &["page"]);
    absent(&mut d.unlisted, &sections::unlisted(conn, from)?, &["page"]);
    absent(
        &mut d.sitemap_entries,
        &sections::sitemap_entries(conn, from)?,
        &["page"],
    );
    repos(d, &by_repo);
    let flags = column(&d.by_repo, &["repo"], |k| {
        now_repos.contains(&k[0]) && !then_repos.contains(&k[0])
    });
    push(&mut d.by_repo, "new_listed", flags);
    new_links(d);
    Ok(())
}

/// Marks `now`'s rows new where `then` has no row of the same `key`, the
/// JSON's `new` (D75), every row counting whatever level the map gives it.
fn absent(now: &mut Table, then: &Table, key: &[&str]) {
    let then = keys(then, key);
    let flags = column(now, key, |k| !then.contains(k));
    push(now, "new", flags);
}

/// Marks `now`'s rows `new_listed` where the row is a finding (`finding`) and
/// `then` has no row of the same `key` that is one, `then` being read through
/// the same severity map.
fn absent_listed(now: &mut Table, then: &Table, key: &[&str], finding: impl Fn(&Row) -> bool) {
    let held: HashSet<Key> = rows(then)
        .filter(|r| finding(r))
        .map(|r| r.key(key))
        .collect();
    let flags = rows(now)
        .map(|r| Cell::Bool(finding(&r) && !held.contains(&r.key(key))))
        .collect();
    push(now, "new_listed", flags);
}

/// The pages of `pages` holding a finding under `sev`: a broken link of
/// `broken` or an error of `errors` that the map makes one, or links broken
/// through a doubled slash where that is one.
fn listed_pages(
    sev: &SeverityMap,
    broken: &Table,
    errors: &Table,
    pages: &Table,
) -> HashSet<String> {
    let mut out = HashSet::new();
    for r in rows(broken) {
        if sev
            .of_link(r.text("outcome"), r.text("reason"))
            .is_finding()
        {
            out.insert(r.text("page").unwrap_or_default().to_owned());
        }
    }
    for r in rows(errors) {
        if sev.of_link(Some("error"), r.text("reason")).is_finding() {
            out.insert(r.text("page").unwrap_or_default().to_owned());
        }
    }
    let through = sev
        .of_link(Some("broken_url"), Some(Reason::DoubledSlash.as_str()))
        .is_finding();
    for r in rows(pages) {
        if through && matches!(r.cell("through"), Cell::Int(n) if *n > 0) {
            out.insert(r.text("page").unwrap_or_default().to_owned());
        }
    }
    out
}

/// The source repos of the `pages` rows named in `listed`.
fn repos_of(pages: &Table, listed: &HashSet<String>) -> HashSet<Option<String>> {
    rows(pages)
        .filter(|r| listed.contains(r.text("page").unwrap_or_default()))
        .map(|r| text(r.cell("source_repo")))
        .collect()
}

fn rows(t: &Table) -> impl Iterator<Item = Row<'_>> {
    t.rows.iter().map(|cells| Row { table: t, cells })
}

/// A row of a table, its cells found by column name.
struct Row<'a> {
    table: &'a Table,
    cells: &'a [Cell],
}

impl<'a> Row<'a> {
    fn cell(&self, name: &str) -> &'a Cell {
        &self.cells[at(self.table, name)]
    }

    fn text(&self, name: &str) -> Option<&'a str> {
        self.cell(name).text().filter(|s| !s.is_empty())
    }

    fn key(&self, key: &[&str]) -> Key {
        key.iter().map(|k| text(self.cell(k))).collect()
    }
}

/// A repo is new where its pages hold a broken link or an error now and held
/// none then.
fn repos(d: &mut Digest, then: &Table) {
    let (repo, broken) = (at(then, "repo"), at(then, "broken_pages"));
    let held: HashSet<Option<String>> = then
        .rows
        .iter()
        .filter(|r| matches!(r[broken], Cell::Int(n) if n > 0))
        .map(|r| text(&r[repo]))
        .collect();
    let (repo, broken) = (at(&d.by_repo, "repo"), at(&d.by_repo, "broken_pages"));
    let flags = d
        .by_repo
        .rows
        .iter()
        .map(|r| {
            let now = matches!(r[broken], Cell::Int(n) if n > 0);
            Cell::Bool(now && !held.contains(&text(&r[repo])))
        })
        .collect();
    push(&mut d.by_repo, "new", flags);
}

/// Counts each page's new broken links and new errors, and each repo's over
/// its pages.
fn new_links(d: &mut Digest) {
    let mut per_page: HashMap<String, i64> = HashMap::new();
    for t in [&d.broken, &d.errors] {
        let (page, new) = (at(t, "page"), at(t, "new"));
        for r in &t.rows {
            if matches!(r[new], Cell::Bool(true)) {
                *per_page
                    .entry(text(&r[page]).unwrap_or_default())
                    .or_default() += 1;
            }
        }
    }
    let (page, repo) = (at(&d.pages, "page"), at(&d.pages, "source_repo"));
    let mut per_repo: HashMap<Option<String>, i64> = HashMap::new();
    let counts: Vec<Cell> = d
        .pages
        .rows
        .iter()
        .map(|r| {
            let n = per_page
                .get(&text(&r[page]).unwrap_or_default())
                .copied()
                .unwrap_or(0);
            *per_repo.entry(text(&r[repo])).or_default() += n;
            Cell::Int(n)
        })
        .collect();
    push(&mut d.pages, "new_links", counts);
    let repo = at(&d.by_repo, "repo");
    let counts = d
        .by_repo
        .rows
        .iter()
        .map(|r| Cell::Int(per_repo.get(&text(&r[repo])).copied().unwrap_or(0)))
        .collect();
    push(&mut d.by_repo, "new_links", counts);
}

fn at(t: &Table, name: &str) -> usize {
    t.columns
        .iter()
        .position(|c| c == name)
        .unwrap_or_else(|| panic!("the digest's table has no column `{name}`"))
}

fn text(cell: &Cell) -> Option<String> {
    cell.text().map(str::to_owned)
}

fn keys(t: &Table, key: &[&str]) -> HashSet<Key> {
    if t.columns.is_empty() {
        return HashSet::new();
    }
    let at: Vec<usize> = key.iter().map(|k| at(t, k)).collect();
    t.rows
        .iter()
        .map(|r| at.iter().map(|&i| text(&r[i])).collect())
        .collect()
}

/// A `Bool` for each of `t`'s rows, `is_new` of its key.
fn column(t: &Table, key: &[&str], is_new: impl Fn(&Key) -> bool) -> Vec<Cell> {
    let at: Vec<usize> = key.iter().map(|k| at(t, k)).collect();
    t.rows
        .iter()
        .map(|r| Cell::Bool(is_new(&at.iter().map(|&i| text(&r[i])).collect())))
        .collect()
}

fn push(t: &mut Table, name: &str, cells: Vec<Cell>) {
    t.columns.push(name.to_owned());
    for (row, cell) in t.rows.iter_mut().zip(cells) {
        row.push(cell);
    }
}

#[cfg(test)]
mod tests;
