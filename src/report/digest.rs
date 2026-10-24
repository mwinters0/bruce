//! The digest: one crawl run's report, a document built once and written as
//! dated JSON (`json`), for a database to load and graph over a retention the
//! crawl's own never has, and as HTML (`html`) for a person to read (D64).
//!
//! It composes what the reports hold into the sections of the P8 spec: the
//! figures, the broken targets ranked by the links naming them, the pages
//! ranked by their broken links, the repos and components, the errors apart
//! from breakage (D16) and the housekeeping. Every section is a [`Table`] whose
//! rows are already in the order a reader wants them, so a rendering walks
//! them and ranks nothing. Everything counts every kind of link (D13), and
//! reads the pages a root covers alone.

mod document;
mod html;
mod json;
mod sections;

use duckdb::Connection;

use super::error::Error;
use super::run;
use super::table::Table;
use crate::store::{RunId, Store};

pub use document::Digest;
pub use json::FORMAT_VERSION;

/// The digest of `run`, else of the latest finished one (D59). A run
/// `check-dir` made has none.
pub fn digest(store: &Store, run: Option<RunId>) -> Result<Digest, Error> {
    let conn = store.connection();
    let (id, unfinished, later_unfinished) = run::one(conn, run)?;
    build(conn, id, unfinished, later_unfinished)
}

fn build(
    conn: &Connection,
    id: RunId,
    unfinished: bool,
    later_unfinished: Option<RunId>,
) -> Result<Digest, Error> {
    let run = sections::run(conn, id)?;
    let counts = sections::page_counts(conn, id)?;
    let pages = sections::pages(conn, id, counts)?;
    let mut broken = sections::broken(conn, id)?;
    in_order_of(&mut broken, &pages);
    Ok(Digest {
        run,
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
        roots_without_sitemap: sections::roots_without_sitemap(conn, id)?,
        unfinished,
        later_unfinished,
    })
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
