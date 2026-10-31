//! A trial policy in place of every run's recorded one (D84).

use super::{Error, Store};
use crate::config::Policy;

impl Store {
    /// Puts `policy` in the place of every run's recorded one for this
    /// connection (D84): a temporary table of the name of the `policy_in_force`
    /// view, which every reader of the policy asks for and which the
    /// connection finds first, holding the policy's rows once for each run the
    /// database holds, the judging rows alone: how a run asked is not
    /// judged and stays as it recorded it (D85). The stored `policy` is not
    /// written and the connection's end removes the table. A run imported afterwards keeps the rows its
    /// file carried.
    pub fn try_policy(&self, policy: &Policy) -> Result<(), Error> {
        self.conn.execute_batch(
            "CREATE OR REPLACE TEMP TABLE policy_in_force (run_id UBIGINT NOT NULL, host VARCHAR, \
             key VARCHAR NOT NULL, value VARCHAR NOT NULL)",
        )?;
        for row in policy.entries().into_iter().filter(|e| !e.is_asking()) {
            self.conn.execute(
                "INSERT INTO temp.policy_in_force SELECT run_id, ?, ?, ? FROM main.runs",
                duckdb::params![row.host, row.key, row.value],
            )?;
        }
        Ok(())
    }
}
