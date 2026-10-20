//! The baseline file (D21): one finished run's rows in a fresh database of the
//! current schema, so every view and report reads it unchanged (D28, D43).
//!
//! The copy is `INSERT ... SELECT` across an `ATTACH`, table by table in one
//! transaction. `bodies` stay behind, a baseline answering what a crawl found
//! and not what it fetched (D15), and `runs.baseline_run` is cleared, naming a
//! run the file does not hold.

use std::fs;
use std::path::{Path, PathBuf};

use super::{Error, RunId, Store};
use crate::profile::Registry;

/// The tables a baseline carries, `runs` first, each filtered by its `run_id`.
const TABLES: [&str; 6] = ["runs", "targets", "pages", "page_facts", "anchors", "links"];

/// What [`Store::export_baseline`] copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exported {
    /// The run, which keeps its number in the file.
    pub run: RunId,
    pub targets: u64,
    pub pages: u64,
    pub facts: u64,
    pub anchors: u64,
    pub links: u64,
}

impl Store {
    /// Writes `run`, or the latest finished run, into a new database at `out`
    /// and gives what it copied. **An unfinished run is refused**: a stopped
    /// run lacks targets (D48), and each would become a live request in every
    /// CI job that reads the file. `out` must not exist; the file is removed
    /// again if the copy fails.
    pub fn export_baseline(
        &self,
        run: Option<RunId>,
        out: &Path,
        profiles: &Registry,
    ) -> Result<Exported, Error> {
        let run = self.exportable(run)?;
        Store::open(out, profiles)?;
        match self.copy_into(run, out) {
            Ok(exported) => Ok(exported),
            Err(e) => {
                remove(out);
                Err(e)
            }
        }
    }

    /// `wanted`, or the latest finished run, if it is finished.
    fn exportable(&self, wanted: Option<RunId>) -> Result<RunId, Error> {
        let run = match wanted {
            Some(run) => run,
            None => {
                let latest: Option<u64> = self.conn.query_row(
                    "SELECT max(run_id) FROM runs WHERE finished_at IS NOT NULL",
                    [],
                    |row| row.get(0),
                )?;
                return latest.map(RunId).ok_or(Error::NoFinishedRun);
            }
        };
        let unfinished: Option<bool> = self
            .conn
            .query_row(
                "SELECT finished_at IS NULL FROM runs WHERE run_id = ?",
                [run.0],
                |row| row.get(0),
            )
            .map(Some)
            .or_else(|e| match e {
                duckdb::Error::QueryReturnedNoRows => Ok(None),
                e => Err(e),
            })?;
        match unfinished {
            None => Err(Error::NoSuchRun(run)),
            Some(true) => Err(Error::Unfinished(run)),
            Some(false) => Ok(run),
        }
    }

    fn copy_into(&self, run: RunId, out: &Path) -> Result<Exported, Error> {
        let path = out.to_string_lossy().replace('\'', "''");
        self.conn
            .execute_batch(&format!("ATTACH '{path}' AS baseline_out"))?;
        let copied = self.copy_tables(run);
        // Detaching checkpoints, so the file is whole on return; it is tried
        // after a failed copy too, to release the file.
        let detached = self.conn.execute_batch("DETACH baseline_out");
        let counts = copied?;
        detached?;
        let [_, targets, pages, facts, anchors, links] = counts;
        Ok(Exported {
            run,
            targets,
            pages,
            facts,
            anchors,
            links,
        })
    }

    fn copy_tables(&self, run: RunId) -> Result<[u64; 6], Error> {
        self.conn.execute_batch("BEGIN")?;
        let mut counts = [0; 6];
        let copied = TABLES.iter().enumerate().try_for_each(|(i, table)| {
            let select = if *table == "runs" {
                "run_id, started_at, finished_at, mode, tool_version, config_hash, \
                 NULL, roots"
            } else {
                "*"
            };
            counts[i] = self.conn.execute(
                &format!(
                    "INSERT INTO baseline_out.main.{table} \
                     SELECT {select} FROM main.{table} WHERE run_id = ?"
                ),
                [run.0],
            )? as u64;
            Ok::<(), duckdb::Error>(())
        });
        match copied {
            Ok(()) => self.conn.execute_batch("COMMIT")?,
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(e.into());
            }
        }
        Ok(counts)
    }
}

/// Removes a database file and its write-ahead log, if there.
fn remove(db: &Path) {
    let mut wal = db.as_os_str().to_owned();
    wal.push(".wal");
    for file in [db.to_path_buf(), PathBuf::from(wal)] {
        let _ = fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests;
