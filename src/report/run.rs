//! Choosing the run a report reads.

use duckdb::Connection;

use super::error::Error;
use crate::store::RunId;

/// `wanted`, or the latest finished run, and whether the run is unfinished.
/// A named run may be unfinished, which the caller says; the default never is.
pub(super) fn choose(conn: &Connection, wanted: Option<RunId>) -> Result<(RunId, bool), Error> {
    match wanted {
        Some(run) => {
            let mut stmt = conn.prepare("SELECT finished_at IS NULL FROM runs WHERE run_id = ?")?;
            let mut rows = stmt.query([run.0])?;
            match rows.next()? {
                Some(row) => Ok((run, row.get(0)?)),
                None => Err(Error::NoSuchRun(run)),
            }
        }
        None => {
            let latest: Option<u64> = conn.query_row(
                "SELECT max(run_id) FROM runs WHERE finished_at IS NOT NULL",
                [],
                |row| row.get(0),
            )?;
            latest
                .map(|id| (RunId(id), false))
                .ok_or(Error::NoFinishedRun)
        }
    }
}
