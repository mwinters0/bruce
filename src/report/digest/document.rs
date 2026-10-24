//! The digest's document: the run, its figures and the sections, each section
//! a [`Table`] already in the order a reader wants it.

use crate::report::table::Table;
use crate::store::RunId;

/// The run a digest is of, as the database records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub id: RunId,
    /// The `bruce` that crawled it, which may not be the one writing the digest.
    pub tool_version: String,
    /// Seconds since the Unix epoch.
    pub started: i64,
    /// `None` for a run that never finished.
    pub finished: Option<i64>,
    pub roots: Vec<String>,
}

/// The run's pages and links, and the links by outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Totals {
    pub pages: i64,
    pub links: i64,
    pub broken_url: i64,
    pub broken_anchor: i64,
    pub error: i64,
    pub unchecked: i64,
}

/// A crawl run's digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    pub run: Run,
    pub totals: Totals,
    /// The `unchecked` links by reason: `reason`, `links`, `targets`.
    pub unchecked: Table,
    /// The `unchecked` links by reason and by where they point: `reason`,
    /// `group` (a scheme, a host, or a host and part of a path), `links`,
    /// `targets`, a reason's groups the most links first.
    pub unchecked_groups: Table,
    /// The hosts no root starts on whose links' targets were asked about,
    /// the most links first: `host`, `links`, `targets`.
    pub checked_hosts: Table,
    /// The broken targets, the one the most links name first and then the
    /// most pages: `target`, `fragment`, `outcome`, `reason`, `status`,
    /// `links`, `pages`, and the
    /// target page's `source_repo`, `source_path` and `source_url`.
    pub broken_targets: Table,
    /// The pages holding a broken link, the most first: `page`, `title`,
    /// `source_repo`, `source_path`, `source_url`, `links`, `broken_url`,
    /// `broken_anchor` and `error`.
    pub pages: Table,
    /// Every broken link, a page's together and the pages in `pages`' order:
    /// `page`, `raw_href`, `target`, `fragment`, `kind`, `outcome`, `reason`
    /// and `status`.
    pub broken: Table,
    /// Each repo, the most broken first: `repo`, `pages`, `broken_pages`,
    /// `links`, `broken_url`, `broken_anchor` and `error`.
    pub by_repo: Table,
    /// `by_repo`'s columns for each Antora component, as `component`.
    pub by_component: Table,
    /// Every link whose outcome is `error`: `page`, `source_url`, `raw_href`,
    /// `target`, `fragment`, `kind`, `reason` and `status`.
    pub errors: Table,
    /// The redirected links by landing URL (`redirects`' columns and `pages`).
    pub redirects: Table,
    /// The sitemap's pages no page links to (`orphans`' columns).
    pub orphans: Table,
    /// The pages absent from their root's sitemap (`unlisted`' columns).
    pub unlisted: Table,
    /// The sitemap's entries no page was fetched for, those that did not
    /// answer `ok` first: `page`, `outcome`, `reason` and `status`.
    pub sitemap_entries: Table,
    /// The roots that read no sitemap, which have no unlisted pages.
    pub roots_without_sitemap: Vec<String>,
    /// The run never finished, so the digest is incomplete.
    pub unfinished: bool,
    /// Where no run was named, the latest run after the one read that never
    /// finished.
    pub later_unfinished: Option<RunId>,
}

impl Digest {
    /// The day the run started, in UTC, `YYYY-MM-DD`: the digest's name.
    pub fn date(&self) -> String {
        utc(self.run.started).0
    }
}

/// `seconds` since the epoch as a UTC date, `YYYY-MM-DD`, and as an RFC 3339
/// time, `YYYY-MM-DDTHH:MM:SSZ`.
pub(super) fn utc(seconds: i64) -> (String, String) {
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    // Civil from days, as Howard Hinnant's `civil_from_days` has it.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let date = format!("{year:04}-{month:02}-{day:02}");
    let time = format!(
        "{date}T{:02}:{:02}:{:02}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    );
    (date, time)
}
