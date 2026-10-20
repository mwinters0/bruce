//! What a baseline run holds, read once before a CI run starts (D52).
//!
//! The crawl decides a URL's route as it plans it, and the writer thread owns
//! the connection by then, so the question "does the baseline hold a target
//! for this URL" is answered from a set loaded up front.

use std::collections::HashSet;

use super::super::{Error, RunId, Store};

/// The `url_id` of every target a baseline run holds.
#[derive(Debug, Clone, Default)]
pub struct BaselineIndex {
    targets: HashSet<u64>,
}

impl BaselineIndex {
    /// An index of `targets`.
    pub fn new(targets: impl IntoIterator<Item = u64>) -> BaselineIndex {
        BaselineIndex {
            targets: targets.into_iter().collect(),
        }
    }

    /// Whether the baseline holds a target for `url_id`, so a copy of its
    /// rows answers it ([`Store::copy_target`]).
    pub fn holds(&self, url_id: u64) -> bool {
        self.targets.contains(&url_id)
    }
}

impl Store {
    /// The targets `run` holds.
    pub fn baseline_index(&self, run: RunId) -> Result<BaselineIndex, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT url_id FROM targets WHERE run_id = ?")?;
        let ids = stmt
            .query_map([run.0], |row| row.get::<_, u64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(BaselineIndex::new(ids))
    }
}
