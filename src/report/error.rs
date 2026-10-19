//! Why a report could not be run.

use std::fmt;

use crate::store::RunId;

#[derive(Debug)]
pub enum Error {
    Db(duckdb::Error),
    /// No `--run`, and no run in the database has finished.
    NoFinishedRun,
    NoSuchRun(RunId),
    /// A `diff` with no `--from`, and no finished run before the one compared.
    NoEarlierRun(RunId),
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
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Db(e) => Some(e),
            _ => None,
        }
    }
}

impl From<duckdb::Error> for Error {
    fn from(e: duckdb::Error) -> Error {
        Error::Db(e)
    }
}
