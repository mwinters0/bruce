//! Why a report could not be run.

use std::fmt;

use crate::store::RunId;

#[derive(Debug)]
pub enum Error {
    Db(duckdb::Error),
    /// No `--run`, and no run in the database has finished.
    NoFinishedRun,
    NoSuchRun(RunId),
    /// A two-run report (`diff`, `changed`, `anchors-removed`) with no `--from`,
    /// and no finished run before the one compared.
    NoEarlierRun(RunId),
    /// An `introduced` or `inverse` report over a run that names no baseline,
    /// which only a `check-dir` run does.
    NoBaseline(RunId),
    /// A digest of a run `check-dir` made, which only a crawl run has.
    NotCrawl(RunId),
    /// No `--run`, and the latest finished run is the baseline of a later run
    /// that never finished: a stopped `check-dir` (D59).
    BaselineOfLater {
        run: RunId,
        later: RunId,
    },
    /// The baseline file a digest is compared with could not be imported (D52).
    Baseline(crate::store::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Db(e) => write!(f, "{e}"),
            Error::NoFinishedRun => {
                f.write_str("the database holds no finished run; name one with --run")
            }
            Error::NoSuchRun(run) => write!(f, "the database holds no run {}", run.0),
            Error::NoEarlierRun(run) => write!(
                f,
                "the database holds no finished run before run {}; name one with --from",
                run.0
            ),
            Error::NoBaseline(run) => write!(
                f,
                "run {} was not checked against a baseline; name a `check-dir` run with --run",
                run.0
            ),
            Error::NotCrawl(run) => write!(
                f,
                "run {} is a `check-dir` run; a digest is of, and compares with, a crawl run, \
                 so name one with --run or --from",
                run.0
            ),
            Error::Baseline(e) => write!(f, "the baseline to compare with: {e}"),
            // deficiency: KD22 The advice is `--run`'s, which `cli::report::check`
            // refuses for `diff`, `changed` and `anchors-removed`: their default
            // `--to` raises this too (`run::choose`), and is answered by `--to`.
            Error::BaselineOfLater { run, later } => write!(
                f,
                "run {} is the baseline run {} was checked against, and run {} never finished; \
                 name a run with --run",
                run.0, later.0, later.0
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Db(e) => Some(e),
            Error::Baseline(e) => Some(e),
            _ => None,
        }
    }
}

impl From<duckdb::Error> for Error {
    fn from(e: duckdb::Error) -> Error {
        Error::Db(e)
    }
}
