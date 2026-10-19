//! The database a subcommand reads or writes, shared by `crawl` and `report`.

use std::path::Path;

use crate::profile::Registry;
use crate::store::{self, Store};

/// The database a subcommand uses when `--db` names none.
pub const DEFAULT_DB: &str = "bruce.duckdb";

/// `--db`, or [`DEFAULT_DB`].
pub fn path(db: Option<&Path>) -> &Path {
    db.unwrap_or(Path::new(DEFAULT_DB))
}

/// Opens the database at `path` with every view installed: the store's, the
/// profiles' and the reports' (D6), so a `duckdb` user finds them after a
/// crawl alone.
pub fn open(path: &Path, registry: &Registry) -> Result<Store, store::Error> {
    let store = Store::open(path, registry)?;
    store.install(crate::report::VIEWS)?;
    Ok(store)
}
