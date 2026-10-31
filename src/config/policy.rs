//! The policy a run judges its links by, and records (D78).
//!
//! A site's judging settings live in config, and a run stores the policy it
//! was judged by, so a report reads the run and never the file. Every setting
//! has a default, today's value (D77); [`Policy::default`] is the policy of a
//! run given no file. A setting is a field and a row of [`Policy::entries`],
//! so a new one changes no schema. A setting that is a host's own is a field
//! of [`HostJudging`], its rows carrying the host. The settings that decide how
//! a crawl asks are recorded beside them and judged by nothing ([`Asking`],
//! D85).

use std::collections::BTreeMap;

use serde::Deserialize;

pub(in crate::config) use self::asking::RawAsking;
pub use self::asking::{Asking, HostAsking, KEYS as ASKING_KEYS, backoff};
pub use self::entry::Entry;
pub(in crate::config) use self::gate::RawGate;
pub use self::gate::{DEFAULT_FAIL, FailOn};
pub use self::severity::{REASONS as SEVERITY_REASONS, Severity, SeverityMap};

mod asking;
mod entry;
mod gate;
mod severity;

/// The schemes besides `http(s)` that work on a public web page (D66), the
/// first of [`Policy::schemes`].
pub const WORKING_SCHEMES: [&str; 12] = [
    "mailto",
    "tel",
    "irc",
    "ircs",
    "xmpp",
    "appstream",
    "ftp",
    "news",
    "magnet",
    "sms",
    "data",
    "javascript",
];

/// What a status code is judged to be (D16): the outcomes a response can
/// carry, `unchecked` being a link not asked about and `broken_anchor` a
/// fragment's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Ok,
    BrokenUrl,
    Error,
}

impl Answer {
    /// The outcome as the store spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Answer::Ok => "ok",
            Answer::BrokenUrl => "broken_url",
            Answer::Error => "error",
        }
    }

    fn parse(s: &str) -> Result<Answer, String> {
        match s {
            "ok" => Ok(Answer::Ok),
            "broken_url" => Ok(Answer::BrokenUrl),
            "error" => Ok(Answer::Error),
            other => Err(format!(
                "{other:?} is not an outcome a status may be judged to: ok, broken_url or error"
            )),
        }
    }
}

/// How far the links broken by a doubled slash are counted on the page that
/// wrote it (D71, D80): the broken relative links on the doubled-slash pages
/// its link reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Every doubled-slash page the landing page's relative links reach.
    Full,
    /// The landing page alone.
    Immediate,
    /// None: those links are counted nowhere.
    Off,
}

impl Reach {
    /// The reach as the store spells it and a file writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Reach::Full => "full",
            Reach::Immediate => "immediate",
            Reach::Off => "off",
        }
    }

    /// The reach a policy row spells, `full` where it is missing or unknown.
    pub fn from_row(value: Option<&str>) -> Reach {
        [Reach::Immediate, Reach::Off]
            .into_iter()
            .find(|r| Some(r.as_str()) == value)
            .unwrap_or(Reach::Full)
    }

    fn parse(s: &str) -> Result<Reach, String> {
        [Reach::Full, Reach::Immediate, Reach::Off]
            .into_iter()
            .find(|r| r.as_str() == s)
            .ok_or_else(|| format!("{s:?} is not a reach: full, immediate or off"))
    }
}

/// The judging settings of a run, each at its default until config sets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Whether a redirect that only adds or drops a final `/` is `ok`, reason
    /// `pedantic_trailing_slash`, rather than a redirect to correct (D70).
    pub pedantic_trailing_slash: bool,
    /// The schemes besides `http(s)` counted `unchecked`, reason `scheme`,
    /// rather than `broken_url`, reason `bad_scheme` (D66): the built-in
    /// [`WORKING_SCHEMES`] first, then the file's.
    pub schemes: Vec<String>,
    /// Whether a link to a URL whose path doubles a slash is `broken_url`,
    /// reason `doubled_slash`, where a host sets none (D71).
    pub doubled_slash: bool,
    /// How far the breakage a doubled slash causes is counted on the page that
    /// wrote the link (D71, D80).
    pub doubled_slash_reach: Reach,
    /// What a status code is judged to be where a host sets none, a code
    /// not here being judged as the crawl classed it (D16).
    pub status: BTreeMap<u16, Answer>,
    /// How loudly the digest shows each kind of finding (D82).
    pub severity: SeverityMap,
    /// The outcomes that fail the gate, in [`FailOn`]'s order (D83).
    pub gate_fail: Vec<FailOn>,
    /// The hosts' own settings, by host as `Host` prints it, only those a
    /// file names.
    pub hosts: BTreeMap<String, HostJudging>,
    /// How the run asks, which no report judges by (D85).
    pub asking: Asking,
}

/// The settings one host sets for itself, beside its politeness (D8).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostJudging {
    /// A link to a URL on this host, in place of [`Policy::doubled_slash`].
    pub doubled_slash: Option<bool>,
    /// What a code this host answered is judged to be, over [`Policy::status`].
    pub status: BTreeMap<u16, Answer>,
}

impl Default for Policy {
    fn default() -> Policy {
        Policy {
            pedantic_trailing_slash: true,
            schemes: WORKING_SCHEMES.iter().map(|s| (*s).to_owned()).collect(),
            doubled_slash: true,
            doubled_slash_reach: Reach::Full,
            // D16's class for these three, which the crawl records the same.
            status: [401, 403, 429]
                .into_iter()
                .map(|code| (code, Answer::Error))
                .collect(),
            severity: SeverityMap::default(),
            gate_fail: DEFAULT_FAIL.to_vec(),
            hosts: BTreeMap::new(),
            asking: Asking::default(),
        }
    }
}

impl Policy {
    /// The settings as the rows the store records: every setting, the
    /// defaults included, so a run's policy is whole in its own rows and a
    /// later default changes none (D78). A status is the key `status.<code>`,
    /// a list one row to an element.
    pub fn entries(&self) -> Vec<Entry> {
        let run = |key: &str, value: String| Entry {
            host: None,
            key: key.to_owned(),
            value,
        };
        let mut rows = vec![run(
            "pedantic_trailing_slash",
            self.pedantic_trailing_slash.to_string(),
        )];
        rows.extend(self.schemes.iter().map(|s| run("schemes", s.clone())));
        rows.push(run("doubled_slash", self.doubled_slash.to_string()));
        rows.push(run(
            "doubled_slash_reach",
            self.doubled_slash_reach.as_str().to_owned(),
        ));
        rows.extend(statuses(None, &self.status));
        rows.extend(self.severity.entries());
        rows.push(run("gate.fail", FailOn::row(&self.gate_fail)));
        rows.extend(self.asking.entries());
        for (host, h) in &self.hosts {
            if let Some(on) = h.doubled_slash {
                rows.push(Entry {
                    host: Some(host.clone()),
                    key: "doubled_slash".to_owned(),
                    value: on.to_string(),
                });
            }
            rows.extend(statuses(Some(host), &h.status));
        }
        rows
    }

    /// The policy a file's `[judge]`, `[default]`, `[severity]`, `[gate]` and `[host]` tables say,
    /// each at its default where a key is left out.
    pub(super) fn from_raw(
        judge: RawJudge,
        default: &RawJudging,
        severity: &BTreeMap<String, String>,
        gate: &RawGate,
        hosts: BTreeMap<String, HostJudging>,
        asking: Asking,
    ) -> Result<Policy, String> {
        let mut policy = Policy::default();
        if let Some(on) = judge.pedantic_trailing_slash {
            policy.pedantic_trailing_slash = on;
        }
        if let Some(reach) = &judge.doubled_slash_reach {
            policy.doubled_slash_reach =
                Reach::parse(reach).map_err(|e| format!("judge.doubled_slash_reach: {e}"))?;
        }
        for written in &judge.schemes {
            let scheme = scheme(written).map_err(|e| format!("judge.schemes: {e}"))?;
            if !policy.schemes.contains(&scheme) {
                policy.schemes.push(scheme);
            }
        }
        if let Some(on) = default.doubled_slash {
            policy.doubled_slash = on;
        }
        policy
            .status
            .extend(statuses_of(&default.status).map_err(|e| format!("default.status: {e}"))?);
        policy.severity = SeverityMap::from_raw(severity).map_err(|e| format!("severity: {e}"))?;
        if let Some(fail) = &gate.fail {
            policy.gate_fail = FailOn::from_raw(fail).map_err(|e| format!("gate.fail: {e}"))?;
        }
        policy.hosts = hosts;
        policy.asking = asking;
        Ok(policy)
    }
}

/// The rows of one map of statuses, `host` being theirs.
fn statuses(host: Option<&String>, map: &BTreeMap<u16, Answer>) -> Vec<Entry> {
    map.iter()
        .map(|(code, answer)| Entry {
            host: host.cloned(),
            key: format!("status.{code}"),
            value: answer.as_str().to_owned(),
        })
        .collect()
}

impl HostJudging {
    /// The settings `[host."<name>"]` says, `name` as it is written there.
    pub(super) fn from_raw(name: &str, raw: &RawJudging) -> Result<HostJudging, String> {
        Ok(HostJudging {
            doubled_slash: raw.doubled_slash,
            status: statuses_of(&raw.status).map_err(|e| format!("host.\"{name}\".status: {e}"))?,
        })
    }
}

/// A scheme as a file writes it, in the form a URL has it: lower case, the
/// characters RFC 3986 allows, and not one the crawl asks about.
fn scheme(written: &str) -> Result<String, String> {
    let scheme = written.to_ascii_lowercase();
    let mut chars = scheme.chars();
    let well_formed = chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "+-.".contains(c));
    if !well_formed {
        return Err(format!(
            "{written:?} is not a scheme (a letter, then letters, digits, + - .)"
        ));
    }
    if matches!(scheme.as_str(), "http" | "https") {
        return Err(format!("{scheme} is asked about, not unchecked"));
    }
    Ok(scheme)
}

/// A table of `"<code>" = "<outcome>"` as a file writes it. A code is one
/// response's, and not a 3xx, which is a hop the crawl follows (D7).
fn statuses_of(written: &BTreeMap<String, String>) -> Result<BTreeMap<u16, Answer>, String> {
    let mut map = BTreeMap::new();
    for (code, outcome) in written {
        let n: u16 = code
            .parse()
            .ok()
            .filter(|n| (100..=599).contains(n))
            .ok_or_else(|| format!("{code:?} is not a status code, 100 to 599"))?;
        if (300..=399).contains(&n) {
            return Err(format!(
                "{n} is a redirect, which the crawl follows, so its judging is not a setting"
            ));
        }
        map.insert(
            n,
            Answer::parse(outcome).map_err(|e| format!("{code}: {e}"))?,
        );
    }
    Ok(map)
}

/// A file's `[judge]` table (D77).
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct RawJudge {
    pedantic_trailing_slash: Option<bool>,
    doubled_slash_reach: Option<String>,
    #[serde(default)]
    schemes: Vec<String>,
}

/// The judging keys of a `[default]` or `[host]` table, which the table's own
/// shape reads (it refuses unknown keys, which a flattened one cannot).
#[derive(Default)]
pub(super) struct RawJudging {
    pub doubled_slash: Option<bool>,
    pub status: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests;
