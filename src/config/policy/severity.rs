//! How loudly the digest shows each kind of finding (D82): a severity per
//! outcome, per reason, and for a redirect, the site's to set.
//!
//! D76's three levels are fixed, with *none* for a kind that is no finding;
//! what a site chooses is the level of each kind. A map is keyed by an
//! outcome (`broken_url`, `broken_anchor`, `error`), `redirected` or a
//! reason, and a reason's level beats its outcome's, so `doubled_slash =
//! "attention"` softens that kind of `broken_url` alone.

use std::collections::BTreeMap;

use super::entry::Entry;

/// A kind's severity, the least first. `None` is no dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    None,
    /// A redirect differing from its landing only by a final `/`, by default.
    Pedantic,
    /// `broken_anchor` and a redirect to update, by default.
    Attention,
    /// `broken_url` and `error`, by default.
    Broken,
}

impl Severity {
    /// The level as a file writes it and the store spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::None => "none",
            Severity::Pedantic => "pedantic",
            Severity::Attention => "attention",
            Severity::Broken => "broken",
        }
    }

    /// Whether a kind at this level is a finding: *none* is not (D82).
    pub fn is_finding(self) -> bool {
        self != Severity::None
    }

    fn parse(s: &str) -> Result<Severity, String> {
        [
            Severity::None,
            Severity::Pedantic,
            Severity::Attention,
            Severity::Broken,
        ]
        .into_iter()
        .find(|l| l.as_str() == s)
        .ok_or_else(|| format!("{s:?} is not a level: broken, attention, pedantic or none"))
    }
}

/// The outcomes a severity may be set for. `ok` is no finding and `unchecked`
/// keeps no level (D76).
pub const OUTCOMES: [&str; 3] = ["broken_url", "broken_anchor", "error"];

/// The key for a redirected link.
pub const REDIRECTED: &str = "redirected";

/// The reasons a link can carry, whatever its outcome (`too_large` and
/// `pedantic_trailing_slash` on an `ok` one), which a severity may be set for. The test
/// `crawl::outcome::tests::every_reason_the_crawl_writes_takes_a_severity`
/// holds it to what the crawl and the views write.
pub const REASONS: [&str; 16] = [
    "nxdomain",
    "timeout",
    "tls",
    "connect",
    "reset",
    "other",
    "redirect_loop",
    "too_many_redirects",
    "bad_redirect",
    "too_large",
    "unresolved_xref",
    "bad_scheme",
    "doubled_slash",
    "wrong_trailing_slash",
    "pedantic_trailing_slash",
    "invalid_url",
];

/// The severity of each kind a site has set or left at D76's default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeverityMap(BTreeMap<String, Severity>);

impl Default for SeverityMap {
    fn default() -> SeverityMap {
        SeverityMap(
            [
                ("broken_url", Severity::Broken),
                ("error", Severity::Broken),
                ("broken_anchor", Severity::Attention),
                (REDIRECTED, Severity::Attention),
                ("pedantic_trailing_slash", Severity::Pedantic),
            ]
            .into_iter()
            .map(|(key, level)| (key.to_owned(), level))
            .collect(),
        )
    }
}

impl SeverityMap {
    /// The default map with a file's `[severity]` laid over it.
    pub(super) fn from_raw(written: &BTreeMap<String, String>) -> Result<SeverityMap, String> {
        let mut map = SeverityMap::default();
        for (key, level) in written {
            if key == "unchecked" {
                return Err("unchecked keeps no level".to_owned());
            }
            if !OUTCOMES.contains(&key.as_str())
                && key != REDIRECTED
                && !REASONS.contains(&key.as_str())
            {
                return Err(format!(
                    "{key:?} is not an outcome (broken_url, broken_anchor, error), redirected or a reason"
                ));
            }
            let level = Severity::parse(level).map_err(|e| format!("{key}: {e}"))?;
            map.0.insert(key.clone(), level);
        }
        Ok(map)
    }

    /// The map a run's recorded rows say, the default for a kind with none.
    pub fn from_entries(rows: &[Entry]) -> SeverityMap {
        let mut map = SeverityMap::default();
        for row in rows.iter().filter(|r| r.host.is_none()) {
            if let Some(key) = row.key.strip_prefix("severity.")
                && let Ok(level) = Severity::parse(&row.value)
            {
                map.0.insert(key.to_owned(), level);
            }
        }
        map
    }

    /// The rows the store records, one to a kind.
    pub(super) fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.0.iter().map(|(key, level)| Entry {
            host: None,
            key: format!("severity.{key}"),
            value: level.as_str().to_owned(),
        })
    }

    /// The severity of a link's `outcome` and `reason`: the reason's where the
    /// map names it, else the outcome's, else none.
    pub fn of_link(&self, outcome: Option<&str>, reason: Option<&str>) -> Severity {
        reason
            .and_then(|r| self.0.get(r))
            .or_else(|| outcome.and_then(|o| self.0.get(o)))
            .copied()
            .unwrap_or(Severity::None)
    }

    /// The `reason` of a link of `outcome` where it puts the link off the
    /// level the map gives that outcome, else none: the reason a count part
    /// at a level other than its outcome's names (D82).
    pub fn moved_by<'a>(&self, outcome: &str, reason: Option<&'a str>) -> Option<&'a str> {
        reason.filter(|_| self.of_link(Some(outcome), reason) != self.of_link(Some(outcome), None))
    }

    /// The severity of a redirected link, whose row has a reason and no
    /// outcome.
    pub fn of_redirect(&self, reason: Option<&str>) -> Severity {
        self.of_link(Some(REDIRECTED), reason)
    }

    /// The kinds at `level`, in the order a reader meets them: the outcomes,
    /// `redirected`, then the reasons.
    pub fn kinds_at(&self, level: Severity) -> Vec<&str> {
        let order = OUTCOMES.iter().chain([&REDIRECTED]).chain(REASONS.iter());
        order
            .filter(|k| self.0.get(**k) == Some(&level))
            .copied()
            .collect()
    }
}
