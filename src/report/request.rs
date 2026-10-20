//! What a report is asked: which report, over which run and which links, and
//! what it answers.

use super::table::Table;
use crate::extract::Kind;
use crate::store::RunId;

/// A report `bruce report` runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    /// The links whose outcome is `broken_url` or `broken_anchor`.
    Broken,
    /// The links whose outcome is `error` (D16).
    Errors,
    /// Each crawled page's counts, and the run's totals.
    Stats,
    /// Crawled pages ranked by the distinct pages linking to them.
    Inbound,
    /// Every crawled page, and every sitemap entry that was not fetched.
    Sitemap,
    /// Sitemap pages no other page links to.
    Orphans,
    /// Crawled pages absent from the sitemap of a root that read one.
    Unlisted,
    /// The links broken in one run and not in another, and the reverse.
    Diff,
    /// The redirected links, grouped by where they landed.
    Redirects,
    /// A CI run's broken links that its baseline did not have broken, the
    /// gate's failures (D9), and the sections `check-dir` prints beside them.
    Introduced,
}

impl Report {
    /// Every report built.
    pub const ALL: [Report; 10] = [
        Report::Broken,
        Report::Errors,
        Report::Stats,
        Report::Inbound,
        Report::Sitemap,
        Report::Orphans,
        Report::Unlisted,
        Report::Redirects,
        Report::Diff,
        Report::Introduced,
    ];

    /// The name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Report::Broken => "broken",
            Report::Errors => "errors",
            Report::Stats => "stats",
            Report::Inbound => "inbound",
            Report::Sitemap => "sitemap",
            Report::Orphans => "orphans",
            Report::Unlisted => "unlisted",
            Report::Redirects => "redirects",
            Report::Diff => "diff",
            Report::Introduced => "introduced",
        }
    }
}

/// What to report, and over which links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub report: Report,
    /// The run; the latest finished one where `None`.
    pub run: Option<RunId>,
    /// `diff` only: the run compared against, by default the finished run
    /// before `to`.
    pub from: Option<RunId>,
    /// `diff` only: the run compared, by default the latest finished one.
    pub to: Option<RunId>,
    /// `pages.source_repo` of the page a row is about: the linking page for a
    /// link report, else the page listed.
    pub repo: Option<String>,
    /// `page_facts` pairs that page holds, all of them.
    pub facts: Vec<(String, String)>,
    /// The link kinds counted. A sitemap report counts no link.
    pub kinds: Vec<Kind>,
    /// Keep this many rows, in the report's order.
    pub top: Option<usize>,
    /// Name the linking pages in a `redirects` group, a list too long for a
    /// text table.
    pub linking_pages: bool,
}

impl Request {
    /// `report` in the latest finished run, unfiltered, over anchor links;
    /// the gate judges every kind (D13).
    pub fn new(report: Report) -> Request {
        Request {
            report,
            run: None,
            from: None,
            to: None,
            repo: None,
            facts: Vec::new(),
            kinds: match report {
                Report::Introduced => Kind::ALL.to_vec(),
                _ => vec![Kind::Anchor],
            },
            top: None,
            linking_pages: false,
        }
    }
}

/// A report's rows, and what to say beside them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The run read, or for `diff` the one compared (`to`).
    pub run: RunId,
    /// The run never finished, so the report is incomplete.
    pub unfinished: bool,
    /// `diff` only: the run compared against, and whether it never finished.
    pub from: Option<(RunId, bool)>,
    pub table: Table,
    /// Lines for a text rendering to print under the table; JSON omits them.
    pub notes: Vec<String>,
}
