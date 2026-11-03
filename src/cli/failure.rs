//! Why a subcommand could not do its work: a message for stderr and whether
//! the cause was a database or baseline of another schema version, which every
//! subcommand answers with exit status 3 (D102).

use std::fmt;
use std::path::Path;

use crate::report;
use crate::store;

/// The exit status of a database or baseline file of another schema version,
/// whichever subcommand met it (D102).
pub const SCHEMA: u8 = 3;

/// A subcommand's failure.
#[derive(Debug)]
pub struct Failure {
    message: String,
    schema: bool,
}

impl Failure {
    /// A store failure at `path`, the database or baseline file named in the message.
    pub fn at(path: impl fmt::Display, e: &store::Error) -> Failure {
        Failure {
            message: format!("{path}: {e}"),
            schema: matches!(e, store::Error::SchemaVersion(_)),
        }
    }

    /// A report's failure over the database at `db`, a baseline file it imported included.
    pub fn report(db: &Path, e: &report::Error) -> Failure {
        Failure {
            message: format!("{}: {e}", db.display()),
            schema: matches!(e, report::Error::Baseline(store::Error::SchemaVersion(_))),
        }
    }

    /// The exit status: [`SCHEMA`] for another schema version, else `otherwise`.
    pub fn status(&self, otherwise: u8) -> u8 {
        if self.schema { SCHEMA } else { otherwise }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for Failure {
    fn from(message: String) -> Failure {
        Failure {
            message,
            schema: false,
        }
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Failure {
        Failure::from(message.to_owned())
    }
}

#[cfg(test)]
mod tests;
