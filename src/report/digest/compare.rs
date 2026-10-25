//! What a digest is compared with (D73): the run before it or the one named,
//! its figures, and the anchors removed since, which the JSON alone holds.

use duckdb::Connection;

use super::document::Compared;
use super::sections;
use crate::extract::Kind;
use crate::report::anchors_removed;
use crate::report::error::Error;
use crate::report::request::{Report, Request};
use crate::report::table::Table;
use crate::store::{Mode, RunId};

/// The finished crawl run before `run`, the one a digest compares with where
/// it is told none. A `check-dir` run is none: it holds the build's pages
/// alone.
pub(super) fn previous(conn: &Connection, run: RunId) -> Result<Option<RunId>, Error> {
    let before: Option<u64> = conn.query_row(
        "SELECT max(run_id) FROM runs WHERE finished_at IS NOT NULL AND mode = ? AND run_id < ?",
        duckdb::params![Mode::Crawl.as_str(), run.0],
        |row| row.get(0),
    )?;
    Ok(before.map(RunId))
}

/// `run`, read as the run a digest was compared with; `file` says it came from
/// a baseline file.
pub(super) fn compared(conn: &Connection, run: RunId, file: bool) -> Result<Compared, Error> {
    Ok(Compared {
        run: sections::run(conn, run)?,
        totals: sections::totals(conn, run)?,
        file,
        shares_day: !file && shares_day(conn, run)?,
    })
}

/// Whether another crawl run started on `run`'s UTC day.
fn shares_day(conn: &Connection, run: RunId) -> Result<bool, Error> {
    let others: i64 = conn.query_row(
        "SELECT count(*) FROM runs r, runs this WHERE this.run_id = ? AND r.run_id <> this.run_id \
         AND r.mode = ? AND epoch(r.started_at)::BIGINT // 86400 = epoch(this.started_at)::BIGINT // 86400",
        duckdb::params![run.0, Mode::Crawl.as_str()],
        |row| row.get(0),
    )?;
    Ok(others > 0)
}

/// The anchors `to`'s pages lost since `from` that links still name, over
/// every kind of link (D13), the linking pages listed.
pub(super) fn anchors_removed(conn: &Connection, from: RunId, to: RunId) -> Result<Table, Error> {
    let mut request = Request::new(Report::AnchorsRemoved);
    request.kinds = Kind::ALL.to_vec();
    request.linking_pages = true;
    Ok(anchors_removed::anchors_removed(conn, from, to, &request)?.0)
}

#[cfg(test)]
mod tests;
