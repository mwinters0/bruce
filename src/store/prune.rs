//! Deleting old runs (D2): the one place rows leave the database.
//!
//! A run is a number, and every table but `meta` carries it, so a run goes by
//! deleting its number from each. The rule picks which runs go; three things
//! it never deletes are named in [`Store::prune`].

use std::time::Duration;

use super::{Error, RunId, Store};

/// Every table with a `run_id`, `runs` last.
const TABLES: [&str; 7] = [
    "links",
    "anchors",
    "page_facts",
    "pages",
    "bodies",
    "targets",
    "runs",
];

/// Which runs [`Store::prune`] deletes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Every run older than the `n` latest finished runs, finished or not;
    /// `Keep(0)` deletes nothing, the caller taking `n` of at least 1. With
    /// fewer than `n` finished runs it deletes only what precedes the first
    /// of them, and with none it deletes nothing.
    Keep(usize),
    /// Every run started longer ago than this.
    OlderThan(Duration),
}

impl Store {
    /// Deletes the runs `rule` picks, with every row of theirs, and
    /// checkpoints so the file can give the freed blocks back. Gives the runs
    /// deleted, oldest first.
    ///
    /// **An unfinished run newer than the latest finished one is kept**, so
    /// the last attempt stays inspectable until a finished run passes it; one
    /// older than a finished run is dead and goes by the rule like any other.
    /// **A run a kept run names as its `baseline_run` is kept too**, however
    /// the rule picks it, so the gate over a kept `check-dir` run still finds
    /// its baseline; that run's own baseline is kept in turn. A run kept so
    /// does not count toward `Keep(n)`'s `n`.
    /// DuckDB lets one process write a file (D28), so the caller holds the
    /// database alone and no crawl is writing one of these runs.
    pub fn prune(&self, rule: Rule) -> Result<Vec<RunId>, Error> {
        let doomed = self.doomed(rule)?;
        if doomed.is_empty() {
            return Ok(doomed);
        }
        let ids: Vec<String> = doomed.iter().map(|r| r.0.to_string()).collect();
        let ids = ids.join(", ");
        self.conn.execute_batch("BEGIN")?;
        let deleted = TABLES.iter().try_for_each(|table| {
            self.conn
                .execute_batch(&format!("DELETE FROM {table} WHERE run_id IN ({ids})"))
        });
        match deleted {
            Ok(()) => self.conn.execute_batch("COMMIT")?,
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(e.into());
            }
        }
        self.conn.execute_batch("CHECKPOINT")?;
        Ok(doomed)
    }

    /// The runs `rule` deletes, oldest first.
    fn doomed(&self, rule: Rule) -> Result<Vec<RunId>, Error> {
        let mut doomed = self.picked(rule)?;
        let mut stmt = self.conn.prepare(
            "SELECT run_id, baseline_run FROM runs WHERE baseline_run IS NOT NULL \
             ORDER BY run_id",
        )?;
        let named = stmt
            .query_map([], |row| Ok((row.get::<_, u64>(0)?, row.get::<_, u64>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        // A baseline is numbered before the run that names it (D52), so one
        // pass from the newest run down spares a chain.
        for (kept, baseline) in named.into_iter().rev() {
            if !doomed.contains(&RunId(kept)) {
                doomed.retain(|r| r.0 != baseline);
            }
        }
        Ok(doomed)
    }

    /// The runs `rule` picks before any is spared, oldest first.
    // Times compare as microseconds from the epoch: the bundled DuckDB has no
    // ICU, so a TIMESTAMPTZ takes no interval arithmetic.
    fn picked(&self, rule: Rule) -> Result<Vec<RunId>, Error> {
        let (sql, arg) = match rule {
            Rule::Keep(n) => (
                "SELECT run_id FROM runs WHERE run_id < (SELECT min(run_id) FROM \
                 (SELECT run_id FROM runs WHERE finished_at IS NOT NULL \
                  ORDER BY run_id DESC LIMIT ?)) \
                 ORDER BY run_id",
                n as i64,
            ),
            Rule::OlderThan(age) => (
                "SELECT run_id FROM runs WHERE epoch_us(started_at) < epoch_us(now()) - ? \
                 AND NOT (finished_at IS NULL AND run_id > \
                    coalesce((SELECT max(run_id) FROM runs \
                              WHERE finished_at IS NOT NULL), 0)) \
                 ORDER BY run_id",
                i64::try_from(age.as_micros()).unwrap_or(i64::MAX),
            ),
        };
        let mut stmt = self.conn.prepare(sql)?;
        let ids = stmt
            .query_map([arg], |row| row.get::<_, u64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids.into_iter().map(RunId).collect())
    }
}

#[cfg(test)]
mod tests;
