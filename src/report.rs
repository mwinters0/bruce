//! Reports: SQL views over a run, read by `bruce report <kind>` (D6).
//!
//! A view takes no parameter, so each `report_*` view carries every run and
//! the query binds the run, the filters and the limit (`query`). The views
//! (`views`) are the same rows an ad-hoc `duckdb` user reads; this module
//! chooses the run (`run`), runs one report and renders it (`table`).
//!
//! The filters are the same for every report: the links of kinds `kinds`
//! (`anchor` by default, D13), from pages in the repo `repo` (D17) and
//! holding every `facts` pair (D32, D38).

mod error;
mod query;
mod request;
mod run;
mod table;
mod views;

use crate::store::Store;

pub use error::Error;
pub use request::{Output, Report, Request};
pub use table::{Cell, Format, Table};
pub use views::VIEWS;

/// Runs one report against `store`, whose views are installed.
pub fn run(store: &Store, request: &Request) -> Result<Output, Error> {
    let conn = store.connection();
    let (run, unfinished) = run::choose(conn, request.run)?;
    let (table, notes) = query::run(conn, run, request)?;
    Ok(Output {
        run,
        unfinished,
        table,
        notes,
    })
}

#[cfg(test)]
mod tests;
