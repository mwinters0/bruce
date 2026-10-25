//! What is new since the compared run (D75): a `new` column on each section a
//! reader acts on, true for a row the compared run's same section lacked, and
//! on the pages and repos a `new_links` count of the new links among theirs.
//! A broken link is new where section 3 lists it, so the two agree. With
//! nothing to compare with every such cell is null: nothing is known to be new.

use std::collections::{HashMap, HashSet};

use duckdb::Connection;

use super::document::Digest;
use super::sections;
use crate::report::error::Error;
use crate::report::table::{Cell, Table};

/// A row's key: its cells in the key's columns, as text.
type Key = Vec<Option<String>>;

/// A link's key, as `diff` and section 3 key one (D74).
const LINK: &[&str] = &["page", "target", "fragment"];

/// Adds `new`, and `new_links` on the pages and repos, to `d`'s sections.
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
        return Ok(());
    };
    let new_breakage = keys(&d.new_breakage, LINK);
    let flags = column(&d.broken, LINK, |k| new_breakage.contains(k));
    push(&mut d.broken, "new", flags);
    absent(&mut d.errors, &sections::errors(conn, from)?, LINK);
    // The compared run's counts replace the digest's own, every section that
    // reads those being built by now.
    let counts = sections::page_counts(conn, from)?;
    absent(
        &mut d.pages,
        &sections::pages(conn, from, counts)?,
        &["page"],
    );
    absent(
        &mut d.broken_targets,
        &sections::broken_targets(conn, from)?,
        &["target", "fragment"],
    );
    absent(
        &mut d.redirects,
        &sections::redirects(conn, from)?,
        &["landing"],
    );
    absent(&mut d.orphans, &sections::orphans(conn, from)?, &["page"]);
    absent(&mut d.unlisted, &sections::unlisted(conn, from)?, &["page"]);
    absent(
        &mut d.sitemap_entries,
        &sections::sitemap_entries(conn, from)?,
        &["page"],
    );
    repos(d, &sections::by_repo(conn, from, counts)?);
    new_links(d);
    Ok(())
}

/// Marks `now`'s rows new where `then` has no row of the same `key`.
fn absent(now: &mut Table, then: &Table, key: &[&str]) {
    let then = keys(then, key);
    let flags = column(now, key, |k| !then.contains(k));
    push(now, "new", flags);
}

/// A repo is new where its pages hold a broken link now and held none then.
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
