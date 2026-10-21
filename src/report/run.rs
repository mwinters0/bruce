//! Choosing the run a report reads, or the two a `diff` compares.

use duckdb::Connection;

use super::error::Error;
use super::request::{Report, Request};
use crate::store::RunId;

/// The runs a request reads: `run`, and for `diff` the run it is compared
/// against. Each carries whether it never finished.
pub(super) struct Chosen {
    pub(super) run: RunId,
    pub(super) unfinished: bool,
    pub(super) from: Option<(RunId, bool)>,
}

/// The runs `request` names, else the latest finished one. A `diff` reads
/// `to` and `from`, which default to the latest finished run and the finished
/// run before `to`; every other report reads `run`.
pub(super) fn choose(conn: &Connection, request: &Request) -> Result<Chosen, Error> {
    if request.report != Report::Diff {
        let (run, unfinished) = one(conn, request.run)?;
        return Ok(Chosen {
            run,
            unfinished,
            from: None,
        });
    }
    let (to, unfinished) = one(conn, request.to)?;
    let from = match request.from {
        Some(from) => named(conn, from)?,
        None => {
            let before: Option<u64> = conn.query_row(
                "SELECT max(run_id) FROM runs WHERE finished_at IS NOT NULL AND run_id < ?",
                [to.0],
                |row| row.get(0),
            )?;
            before
                .map(|id| (RunId(id), false))
                .ok_or(Error::NoEarlierRun(to))?
        }
    };
    Ok(Chosen {
        run: to,
        unfinished,
        from: Some(from),
    })
}

/// `wanted`, or the latest finished run, and whether the run is unfinished.
/// A named run may be unfinished, which the caller says; the default never is.
// deficiency: KD13 The default is the latest finished run whatever came after
// it. A stopped `check-dir` leaves its run unfinished (D2, D58), so this reads
// the baseline it imported and says nothing of the later run.
fn one(conn: &Connection, wanted: Option<RunId>) -> Result<(RunId, bool), Error> {
    match wanted {
        Some(run) => named(conn, run),
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

/// The run `run` and whether it is unfinished.
fn named(conn: &Connection, run: RunId) -> Result<(RunId, bool), Error> {
    let mut stmt = conn.prepare("SELECT finished_at IS NULL FROM runs WHERE run_id = ?")?;
    let mut rows = stmt.query([run.0])?;
    match rows.next()? {
        Some(row) => Ok((run, row.get(0)?)),
        None => Err(Error::NoSuchRun(run)),
    }
}
