//! The outcomes that fail the gate (D83): `[gate]` `fail`, the site's, read
//! for the CI run.

use serde::Deserialize;

/// An outcome the gate may fail on. A redirect is no outcome and `unchecked`
/// is a link not asked about, so neither is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FailOn {
    BrokenUrl,
    BrokenAnchor,
    Error,
}

/// What the gate fails on where a site sets none: D9's, an `error` never.
pub const DEFAULT_FAIL: [FailOn; 2] = [FailOn::BrokenUrl, FailOn::BrokenAnchor];

impl FailOn {
    const ALL: [FailOn; 3] = [FailOn::BrokenUrl, FailOn::BrokenAnchor, FailOn::Error];

    /// The outcome as the store spells it and a file writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            FailOn::BrokenUrl => "broken_url",
            FailOn::BrokenAnchor => "broken_anchor",
            FailOn::Error => "error",
        }
    }

    fn parse(s: &str) -> Result<FailOn, String> {
        FailOn::ALL
            .into_iter()
            .find(|o| o.as_str() == s)
            .ok_or_else(|| {
                format!(
                    "{s:?} is not an outcome the gate fails on: broken_url, broken_anchor or error"
                )
            })
    }

    /// The list a file writes, each outcome once, in the order of
    /// [`FailOn::ALL`] whatever order it was written in. Empty is "report
    /// only": everything listed, nothing failing.
    pub(super) fn from_raw(written: &[String]) -> Result<Vec<FailOn>, String> {
        let mut list = Vec::new();
        for outcome in written {
            list.push(FailOn::parse(outcome)?);
        }
        list.sort();
        list.dedup();
        Ok(list)
    }

    /// The list as the one row of a run's policy, comma-joined (D83).
    pub fn row(list: &[FailOn]) -> String {
        list.iter()
            .map(|o| o.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// The list a run's `gate.fail` row says, the default where there is none
    /// and an outcome this version does not know skipped.
    pub fn from_row(value: Option<&str>) -> Vec<FailOn> {
        let Some(value) = value else {
            return DEFAULT_FAIL.to_vec();
        };
        let mut list: Vec<FailOn> = value
            .split(',')
            .filter_map(|s| FailOn::parse(s).ok())
            .collect();
        list.sort();
        list.dedup();
        list
    }
}

/// A file's `[gate]` table (D83).
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(in crate::config) struct RawGate {
    pub fail: Option<Vec<String>>,
}
