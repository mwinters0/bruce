//! The settings that decide how a crawl asks (D85): the timeouts, the retries,
//! the longest `Retry-After` waited out, whether D72's probe is sent, and the
//! cap on a body read. A run records them with the judging settings (D78) but
//! no report judges by them: a run cannot be asked again, so a trial policy
//! leaves them as the run recorded them (D84).

use std::collections::BTreeMap;
use std::time::Duration;

use super::Entry;

/// Seconds to connect to a host before giving up (D16).
pub const DEFAULT_CONNECT_TIMEOUT: u64 = 10;

/// Seconds for a whole response, body included (D16).
pub const DEFAULT_TIMEOUT: u64 = 30;

/// Retries of a request that failed transiently, the waits being 1, 2, 4 …
/// seconds (D16).
pub const DEFAULT_RETRIES: u32 = 3;

/// The most retries a file may ask for: the waits double, so ten is a wait of
/// over eight minutes before the last.
pub const MAX_RETRIES: u32 = 10;

/// Seconds of the longest `Retry-After` waited out; a longer one settles the
/// target `error` at once (D16).
pub const DEFAULT_MAX_RETRY_AFTER: u64 = 60;

/// MiB a page's body may come to; a larger one is `ok`, reason `too_large`,
/// and is not extracted (D16).
pub const DEFAULT_MAX_BODY_MIB: u64 = 16;

/// The keys of the asking settings in a run's policy rows: the five a host may
/// set and, run-wide only, the body cap.
pub const KEYS: [&str; 6] = [
    "connect_timeout",
    "timeout",
    "retries",
    "max_retry_after",
    "slash_probe",
    "max_body_mib",
];

/// How a run asks, each setting at its default until config sets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asking {
    /// Seconds to connect to a host, where it sets none.
    pub connect_timeout: u64,
    /// Seconds for a whole response, where a host sets none.
    pub timeout: u64,
    /// Retries of a request that failed transiently, where a host sets none.
    pub retries: u32,
    /// Seconds of the longest `Retry-After` waited out, where a host sets none.
    pub max_retry_after: u64,
    /// Whether D72's probe is sent, where a host sets none.
    pub slash_probe: bool,
    /// MiB a page's body may come to, whichever host it comes from.
    pub max_body_mib: u64,
    /// The hosts' own settings, by host as `Host` prints it, only those a file
    /// gives an asking key of their own.
    pub hosts: BTreeMap<String, HostAsking>,
}

/// The settings one host sets for itself, each over [`Asking`]'s.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostAsking {
    pub connect_timeout: Option<u64>,
    pub timeout: Option<u64>,
    pub retries: Option<u32>,
    pub max_retry_after: Option<u64>,
    pub slash_probe: Option<bool>,
}

impl Default for Asking {
    fn default() -> Asking {
        Asking {
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            timeout: DEFAULT_TIMEOUT,
            retries: DEFAULT_RETRIES,
            max_retry_after: DEFAULT_MAX_RETRY_AFTER,
            slash_probe: true,
            max_body_mib: DEFAULT_MAX_BODY_MIB,
            hosts: BTreeMap::new(),
        }
    }
}

/// The waits before each of `retries` retries: a second, then double the one
/// before.
pub fn backoff(retries: u32) -> Vec<Duration> {
    (0..retries).map(|n| Duration::from_secs(1 << n)).collect()
}

impl Entry {
    /// Whether the row is one of the asking settings, which no report judges
    /// by.
    pub fn is_asking(&self) -> bool {
        KEYS.contains(&self.key.as_str())
    }
}

impl Asking {
    /// The most a page's body may come to, in bytes.
    pub fn max_body(&self) -> usize {
        usize::try_from(self.max_body_mib.saturating_mul(1024 * 1024)).unwrap_or(usize::MAX)
    }

    /// Whether D72's probe is sent to `host`, as `Host` prints it.
    pub fn probes(&self, host: &str) -> bool {
        self.hosts
            .get(host)
            .and_then(|h| h.slash_probe)
            .unwrap_or(self.slash_probe)
    }

    /// The settings as the rows the store records (D78): every run-wide one,
    /// the defaults included, then each host's own.
    pub(super) fn entries(&self) -> Vec<Entry> {
        let row = |host: Option<&String>, key: &str, value: String| Entry {
            host: host.cloned(),
            key: key.to_owned(),
            value,
        };
        let mut rows = vec![
            row(None, "connect_timeout", self.connect_timeout.to_string()),
            row(None, "timeout", self.timeout.to_string()),
            row(None, "retries", self.retries.to_string()),
            row(None, "max_retry_after", self.max_retry_after.to_string()),
            row(None, "slash_probe", self.slash_probe.to_string()),
            row(None, "max_body_mib", self.max_body_mib.to_string()),
        ];
        for (host, h) in &self.hosts {
            let host = Some(host);
            let numbers = [
                ("connect_timeout", h.connect_timeout),
                ("timeout", h.timeout),
                ("retries", h.retries.map(u64::from)),
                ("max_retry_after", h.max_retry_after),
            ];
            for (key, value) in numbers {
                rows.extend(value.map(|n| row(host, key, n.to_string())));
            }
            rows.extend(
                h.slash_probe
                    .map(|on| row(host, "slash_probe", on.to_string())),
            );
        }
        rows
    }

    /// The settings a file's `[default]` table and top-level `max_body_mib`
    /// say, over the defaults, with the hosts' own.
    pub(in crate::config) fn from_raw(
        max_body_mib: Option<u64>,
        default: &RawAsking,
        hosts: BTreeMap<String, HostAsking>,
    ) -> Result<Asking, String> {
        let set = HostAsking::from_raw("default", default)?;
        let fixed = Asking::default();
        let mut asking = Asking {
            connect_timeout: set.connect_timeout.unwrap_or(fixed.connect_timeout),
            timeout: set.timeout.unwrap_or(fixed.timeout),
            retries: set.retries.unwrap_or(fixed.retries),
            max_retry_after: set.max_retry_after.unwrap_or(fixed.max_retry_after),
            slash_probe: set.slash_probe.unwrap_or(fixed.slash_probe),
            ..fixed
        };
        if let Some(mib) = max_body_mib {
            if mib == 0 {
                return Err("max_body_mib: 0 is not at least 1".to_owned());
            }
            asking.max_body_mib = mib;
        }
        asking.hosts = hosts;
        Ok(asking)
    }
}

impl HostAsking {
    /// The settings the table named `table` (`default`, or `host."<name>"`)
    /// says, each checked.
    pub(in crate::config) fn from_raw(table: &str, raw: &RawAsking) -> Result<HostAsking, String> {
        let at_least_one = |key: &str, n: Option<u64>| match n {
            Some(0) => Err(format!("{table}.{key}: 0 is not at least 1")),
            n => Ok(n),
        };
        if let Some(n) = raw.retries.filter(|n| *n > MAX_RETRIES) {
            return Err(format!(
                "{table}.retries: {n} is more than {MAX_RETRIES}, the waits doubling from a second"
            ));
        }
        Ok(HostAsking {
            connect_timeout: at_least_one("connect_timeout", raw.connect_timeout)?,
            timeout: at_least_one("timeout", raw.timeout)?,
            retries: raw.retries,
            max_retry_after: raw.max_retry_after,
            slash_probe: raw.slash_probe,
        })
    }
}

/// The asking keys of a `[default]` or `[host]` table, which the table's own
/// shape reads, as the judging ones are (`RawJudging`).
#[derive(Debug, Clone, Copy, Default)]
pub(in crate::config) struct RawAsking {
    pub connect_timeout: Option<u64>,
    pub timeout: Option<u64>,
    pub retries: Option<u32>,
    pub max_retry_after: Option<u64>,
    pub slash_probe: Option<bool>,
}
