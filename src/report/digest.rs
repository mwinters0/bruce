//! The digest: one crawl run's report, a document built once and written as
//! dated JSON (`json`), for a database to load and graph over a retention the
//! crawl's own never has, and as HTML (`html`) for a person to read (D64).
//!
//! It composes what the reports hold into the sections of the P8 spec: the
//! figures, the broken targets ranked by the links naming them, the pages
//! ranked by their broken links, the repos and components, the errors apart
//! from breakage (D16) and the housekeeping. Every section is a [`Table`] whose
//! rows are already in the order a reader wants them, so a rendering walks
//! them and ranks nothing but a page's drill-down, its errors joined to its
//! broken links by severity. Everything counts every kind of link (D13), and
//! reads the pages a root covers alone.

mod cause;
mod compare;
mod document;
mod html;
mod json;
mod novelty;
mod sections;

use std::path::Path;

use duckdb::Connection;

use super::error::Error;
use super::run;
use super::table::Table;
use crate::store::{RunId, Store};

pub use document::{Compared, Digest};
pub use json::FORMAT_VERSION;

/// What a digest is compared with (D73).
#[derive(Debug, Clone, Copy)]
pub enum Against<'a> {
    /// The finished crawl run before the digest's, if the database holds one.
    Previous,
    /// A crawl run of the database, finished or not.
    Run(RunId),
    /// A baseline file, imported for the digest and removed again.
    File(&'a Path),
}

/// The digest of `run`, else of the latest finished one (D59), compared
/// `against` another. A run `check-dir` made has none.
pub fn digest(store: &Store, run: Option<RunId>, against: Against<'_>) -> Result<Digest, Error> {
    let conn = store.connection();
    let (id, unfinished, later_unfinished) = run::one(conn, run)?;
    let read = Read {
        id,
        unfinished,
        later_unfinished,
    };
    match against {
        Against::Previous => build(conn, &read, compare::previous(conn, id)?, false),
        Against::Run(from) => build(conn, &read, Some(from), false),
        Against::File(path) => {
            let imported = store.import_baseline(path).map_err(Error::Baseline)?;
            let built = build(conn, &read, Some(imported.run), true);
            // The baseline was imported to be read, and as the latest finished
            // run it would be what a bare `report` read next.
            let discarded = store.discard_import(imported).map_err(Error::Baseline);
            let digest = built?;
            discarded?;
            Ok(digest)
        }
    }
}

/// The run a digest is of, as [`run::one`] chose it.
struct Read {
    id: RunId,
    unfinished: bool,
    later_unfinished: Option<RunId>,
}

fn build(conn: &Connection, read: &Read, from: Option<RunId>, file: bool) -> Result<Digest, Error> {
    let id = read.id;
    let run = sections::run(conn, id)?;
    let counts = sections::page_counts(conn, id)?;
    let pages = sections::pages(conn, id, counts)?;
    let mut broken = sections::broken(conn, id)?;
    in_order_of(&mut broken, &pages);
    let compared = from
        .map(|from| compare::compared(conn, from, file))
        .transpose()?;
    let anchors_removed = match &compared {
        Some(c) => compare::anchors_removed(conn, c.run.id, id)?,
        None => empty(),
    };
    let (new_breakage, fixed_breakage) = match &compared {
        Some(c) => (
            cause::new_breakage(conn, c.run.id, id)?,
            cause::fixed_breakage(conn, c.run.id, id)?,
        ),
        None => (empty(), empty()),
    };
    let mut digest = Digest {
        run,
        compared,
        totals: sections::totals(conn, id)?,
        unchecked: sections::unchecked(conn, id)?,
        unchecked_groups: sections::unchecked_groups(conn, id)?,
        checked_hosts: sections::checked_hosts(conn, id)?,
        broken_targets: sections::broken_targets(conn, id)?,
        pages,
        broken,
        by_repo: sections::by_repo(conn, id, counts)?,
        by_component: sections::by_component(conn, id, counts)?,
        errors: sections::errors(conn, id)?,
        redirects: sections::redirects(conn, id)?,
        orphans: sections::orphans(conn, id)?,
        unlisted: sections::unlisted(conn, id)?,
        sitemap_entries: sections::sitemap_entries(conn, id)?,
        anchors_removed,
        new_breakage,
        fixed_breakage,
        roots_without_sitemap: sections::roots_without_sitemap(conn, id)?,
        unfinished: read.unfinished,
        later_unfinished: read.later_unfinished,
    };
    novelty::mark(conn, &mut digest)?;
    Ok(digest)
}

/// The section of a digest with nothing to compare with.
fn empty() -> Table {
    Table {
        columns: Vec::new(),
        rows: Vec::new(),
    }
}

/// Puts `rows`' first column, a page, in the order `ranked` lists its own;
/// a page `ranked` lacks goes last. The sort is stable, so a page's links
/// keep their order.
fn in_order_of(rows: &mut Table, ranked: &Table) {
    let rank: std::collections::HashMap<&str, usize> = ranked
        .rows
        .iter()
        .enumerate()
        .filter_map(|(i, row)| Some((row.first()?.text()?, i)))
        .collect();
    rows.rows.sort_by_key(|row| {
        row.first()
            .and_then(|cell| cell.text())
            .and_then(|page| rank.get(page).copied())
            .unwrap_or(usize::MAX)
    });
}

#[cfg(test)]
mod tests;
