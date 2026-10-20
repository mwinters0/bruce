//! What a baseline run holds, read once before a CI run starts (D52).
//!
//! The crawl decides a URL's route as it plans it, and the writer thread owns
//! the connection by then, so the questions "does the baseline hold a target
//! for this URL" and "which page has these coordinates" (a profile's Resolve,
//! D38) are answered from sets loaded up front.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use url::Url;

use super::super::{Error, RunId, Store};
use crate::profile::{BoxError, Lookup};

/// A page's facts, as full keys.
type Facts = Vec<(String, String)>;

/// The `url_id` of every target a baseline run holds, and the facts of every
/// page that has any.
#[derive(Debug, Clone, Default)]
pub struct BaselineIndex {
    targets: HashSet<u64>,
    pages: Arc<HashMap<Url, Facts>>,
}

impl BaselineIndex {
    /// An index of `targets`, holding no page's facts.
    pub fn new(targets: impl IntoIterator<Item = u64>) -> BaselineIndex {
        BaselineIndex {
            targets: targets.into_iter().collect(),
            pages: Arc::default(),
        }
    }

    /// This index, with the facts of the pages `pages` names.
    pub fn with_pages(mut self, pages: impl IntoIterator<Item = (Url, Facts)>) -> BaselineIndex {
        self.pages = Arc::new(pages.into_iter().collect());
        self
    }

    /// Whether the baseline holds a target for `url_id`, so a copy of its
    /// rows answers it ([`Store::copy_target`]).
    pub fn holds(&self, url_id: u64) -> bool {
        self.targets.contains(&url_id)
    }
}

/// The baseline's pages that carry a fact, as a profile's [`Lookup`], from
/// memory: for a non-empty filter, the pages a [`super::super::RunLookup`] of
/// its run gives, sorted.
impl Lookup for BaselineIndex {
    fn pages_with_facts(&self, facts: &[(&str, &str)]) -> Result<Vec<Url>, BoxError> {
        let mut urls: Vec<Url> = self
            .pages
            .iter()
            .filter(|(_, held)| {
                facts
                    .iter()
                    .all(|(k, v)| held.iter().any(|(hk, hv)| hk == k && hv == v))
            })
            .map(|(url, _)| url.clone())
            .collect();
        urls.sort();
        Ok(urls)
    }

    fn facts(&self, page: &Url) -> Result<Facts, BoxError> {
        Ok(self.pages.get(page).cloned().unwrap_or_default())
    }
}

impl Store {
    /// The targets `run` holds, and its pages' facts.
    pub fn baseline_index(&self, run: RunId) -> Result<BaselineIndex, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT url_id FROM targets WHERE run_id = ?")?;
        let ids = stmt
            .query_map([run.0], |row| row.get::<_, u64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let mut stmt = self.conn.prepare(
            "SELECT t.url, f.key, f.value FROM page_facts f \
             JOIN targets t ON t.run_id = f.run_id AND t.url_id = f.url_id \
             WHERE f.run_id = ?",
        )?;
        let rows = stmt
            .query_map([run.0], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut pages: HashMap<Url, Facts> = HashMap::new();
        for (url, key, value) in rows {
            // A target whose text is no URL is not a page a profile resolves to.
            if let Ok(url) = Url::parse(&url) {
                pages.entry(url).or_default().push((key, value));
            }
        }
        Ok(BaselineIndex::new(ids).with_pages(pages))
    }
}
