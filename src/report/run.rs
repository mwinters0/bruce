//! Choosing the run a report reads, or the two a two-run report compares.

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use crate::store::RunId;

/// The runs a request reads: `run`, and for a two-run report the run it is
/// compared against. Each carries whether it never finished.
pub(super) struct Chosen {
    pub(super) run: RunId,
    pub(super) unfinished: bool,
    /// The latest run after a defaulted `run` that never finished.
    pub(super) later_unfinished: Option<RunId>,
    pub(super) from: Option<(RunId, bool)>,
}

/// The runs `request` names, else the latest finished one. A two-run report
/// reads `to` and `from`, which default to the latest finished run and the finished
/// run before `to`; every other report reads `run`.
pub(super) fn choose(conn: &Connection, request: &Request) -> Result<Chosen, Error> {
    if !request.report.compares() {
        let (run, unfinished, later_unfinished) = one(conn, request.run)?;
        return Ok(Chosen {
            run,
            unfinished,
            later_unfinished,
            from: None,
        });
    }
    let (to, unfinished, later_unfinished) = one(conn, request.to)?;
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
        later_unfinished,
        from: Some(from),
    })
}

/// `wanted`, or the latest finished run, whether the run is unfinished, and
/// for the default the latest run after it that never finished. A named run
/// may be unfinished, which the caller says.
///
/// **The default refuses a run a later unfinished run was checked against**
/// (D59): that is a `check-dir` that stopped, and the report would read the
/// baseline it imported as if it were the build.
pub(super) fn one(
    conn: &Connection,
    wanted: Option<RunId>,
) -> Result<(RunId, bool, Option<RunId>), Error> {
    if let Some(run) = wanted {
        let (run, unfinished) = named(conn, run)?;
        return Ok((run, unfinished, None));
    }
    let latest: Option<u64> = conn.query_row(
        "SELECT max(run_id) FROM runs WHERE finished_at IS NOT NULL",
        [],
        |row| row.get(0),
    )?;
    let run = RunId(latest.ok_or(Error::NoFinishedRun)?);
    let mut stmt = conn.prepare(
        "SELECT run_id, baseline_run FROM runs \
         WHERE finished_at IS NULL AND run_id > ? ORDER BY run_id DESC",
    )?;
    let later = stmt
        .query_map([run.0], |row| {
            Ok((RunId(row.get(0)?), row.get::<_, Option<u64>>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if let Some((later, _)) = later.iter().find(|(_, base)| *base == Some(run.0)) {
        return Err(Error::BaselineOfLater { run, later: *later });
    }
    Ok((run, false, later.first().map(|(later, _)| *later)))
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
