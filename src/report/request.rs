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
}

impl Report {
    /// Every report built.
    pub const ALL: [Report; 3] = [Report::Broken, Report::Errors, Report::Stats];

    /// The name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Report::Broken => "broken",
            Report::Errors => "errors",
            Report::Stats => "stats",
        }
    }
}

/// What to report, and over which links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub report: Report,
    /// The run; the latest finished one where `None`.
    pub run: Option<RunId>,
    /// `pages.source_repo` of the linking page.
    pub repo: Option<String>,
    /// `page_facts` pairs the linking page holds, all of them.
    pub facts: Vec<(String, String)>,
    /// The link kinds counted.
    pub kinds: Vec<Kind>,
    /// Keep this many rows, the worst first.
    pub top: Option<usize>,
}

impl Request {
    /// `report` over anchor links, in the latest finished run, unfiltered.
    pub fn new(report: Report) -> Request {
        Request {
            report,
            run: None,
            repo: None,
            facts: Vec::new(),
            kinds: vec![Kind::Anchor],
            top: None,
        }
    }
}

/// A report's rows, and what to say beside them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub run: RunId,
    /// The run never finished, so the report is incomplete.
    pub unfinished: bool,
    pub table: Table,
    /// Lines for a text rendering to print under the table; JSON omits them.
    pub notes: Vec<String>,
}
