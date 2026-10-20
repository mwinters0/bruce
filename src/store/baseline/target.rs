//! Copying one target's rows from a run to another of the same database
//! (D52): what a CI run answers a URL the build does not hold with.

use super::super::{Error, RunId, Store};

impl Store {
    /// Copies the rows `link_verdicts` joins for `url_id` from run `from`
    /// into run `to`: its `targets` row, the landing target where it was
    /// redirected, and for each of those the `pages` row and `anchors` of a
    /// page. Each copied target is `baseline` (D51). Gives whether `from`
    /// holds a target for `url_id`; false copies nothing, so the caller goes
    /// live.
    ///
    /// **A target `to` already holds is not touched**, nor its page, so a
    /// landing target that is itself built, or copied by an earlier call, is
    /// not doubled and a second call copies nothing. `page_facts` stay
    /// behind: a copied page is not one the build holds (D22).
    pub fn copy_target(&self, from: RunId, to: RunId, url_id: u64) -> Result<bool, Error> {
        let landing: Option<Option<u64>> = self
            .conn
            .query_row(
                "SELECT (SELECT l.url_id FROM targets l WHERE l.run_id = t.run_id \
                         AND l.url = t.final_url) \
                 FROM targets t WHERE t.run_id = ? AND t.url_id = ?",
                [from.0, url_id],
                |row| row.get(0),
            )
            .map(Some)
            .or_else(|e| match e {
                duckdb::Error::QueryReturnedNoRows => Ok(None),
                e => Err(e),
            })?;
        let Some(landing) = landing else {
            return Ok(false);
        };
        self.conn.execute_batch("BEGIN")?;
        let copied = [Some(url_id), landing]
            .into_iter()
            .flatten()
            .try_for_each(|id| self.copy_one(from, to, id));
        match copied {
            Ok(()) => self.conn.execute_batch("COMMIT")?,
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(e.into());
            }
        }
        Ok(true)
    }

    /// One target, and its page where it has one, unless `to` holds the target.
    fn copy_one(&self, from: RunId, to: RunId, id: u64) -> Result<(), duckdb::Error> {
        let copied = self.conn.execute(
            &format!(
                "INSERT INTO targets \
                 SELECT * REPLACE ({to} AS run_id, 'baseline' AS answered_by) \
                 FROM targets WHERE run_id = {from} AND url_id = {id} \
                 AND NOT EXISTS (SELECT 1 FROM targets x WHERE x.run_id = {to} AND x.url_id = {id})",
                to = to.0,
                from = from.0,
            ),
            [],
        )?;
        if copied == 0 {
            return Ok(());
        }
        for table in ["pages", "anchors"] {
            self.conn.execute(
                &format!(
                    "INSERT INTO {table} SELECT * REPLACE ({to} AS run_id) \
                     FROM {table} WHERE run_id = {from} AND url_id = {id}",
                    to = to.0,
                    from = from.0,
                ),
                [],
            )?;
        }
        Ok(())
    }
}
