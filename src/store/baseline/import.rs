//! Importing a baseline file into a CI database as a run of its own (D52).
//!
//! The file's one run is copied in under the next free number, so the CI
//! run's `baseline_run` can name it and the gate is a two-run query in one
//! database. A run already there with the file's `started_at` and
//! `config_hash` is that baseline, imported before (or the very run the file
//! was exported from), and is reused.

use std::path::Path;

use super::super::{Error, RunId, Store, schema};

/// The alias the baseline file is attached under.
const ALIAS: &str = "baseline_in";

/// What [`Store::import_baseline`] found or made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imported {
    /// The baseline's run in this database.
    pub run: RunId,
    /// True where the database already held it, and nothing was copied.
    pub reused: bool,
}

impl Store {
    /// Makes the run `file` holds a run of this database, or finds it there.
    /// The file must be a baseline: a bruce database of this schema version
    /// (D43) with exactly one run, which finished. It is opened read-only and
    /// left as it was.
    pub fn import_baseline(&self, file: &Path) -> Result<Imported, Error> {
        self.with_attached(file, ALIAS, true, |store| store.import_attached())
    }

    fn import_attached(&self) -> Result<Imported, Error> {
        self.check_attached_version()?;
        let (run, finished): (u64, bool) = {
            let runs: u64 =
                self.conn
                    .query_row("SELECT count(*) FROM baseline_in.main.runs", [], |r| {
                        r.get(0)
                    })?;
            if runs != 1 {
                return Err(Error::NotBaseline(format!("it holds {runs} runs")));
            }
            self.conn.query_row(
                "SELECT run_id, finished_at IS NOT NULL FROM baseline_in.main.runs",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?
        };
        if !finished {
            return Err(Error::NotBaseline(format!("its run {run} never finished")));
        }
        let held: Option<u64> = self.conn.query_row(
            "SELECT min(r.run_id) FROM main.runs r, baseline_in.main.runs b \
                 WHERE r.started_at = b.started_at AND r.config_hash = b.config_hash",
            [],
            |row| row.get(0),
        )?;
        if let Some(held) = held {
            return Ok(Imported {
                run: RunId(held),
                reused: true,
            });
        }
        let next: u64 =
            self.conn
                .query_row("SELECT coalesce(max(run_id), 0) + 1 FROM runs", [], |r| {
                    r.get(0)
                })?;
        self.copy_run("baseline_in.main", "main", RunId(run), RunId(next))?;
        Ok(Imported {
            run: RunId(next),
            reused: false,
        })
    }

    /// Refuses the attached file unless it is a bruce database of this
    /// build's schema version.
    fn check_attached_version(&self) -> Result<(), Error> {
        let has_meta: i64 = self.conn.query_row(
            "SELECT count(*) FROM information_schema.tables \
             WHERE table_catalog = ? AND table_schema = 'main' AND table_name = 'meta'",
            [ALIAS],
            |r| r.get(0),
        )?;
        if has_meta == 0 {
            return Err(Error::NotBruce);
        }
        let found: String = self.conn.query_row(
            "SELECT value FROM baseline_in.main.meta WHERE key = ?",
            [schema::VERSION_KEY],
            |r| r.get(0),
        )?;
        if found != schema::SCHEMA_VERSION.to_string() {
            return Err(Error::SchemaVersion {
                found,
                expected: schema::SCHEMA_VERSION,
            });
        }
        Ok(())
    }
}
