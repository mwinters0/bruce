//! Reports: SQL views over a run, read by `bruce report <kind>` (D6).
//!
//! A view takes no parameter, so each `report_*` view carries every run and
//! the query binds the run and the filters (`query`). The views (`views`) are
//! the same rows an ad-hoc `duckdb` user reads; this module chooses the run
//! (`run`), runs one report and renders it (`table`).
//!
//! The filters are the same for every report: the links of kinds `kinds`
//! (`anchor` by default, every kind for the gate, D13; `sitemap` and `unlisted` count no link), and the
//! pages in the repo `repo` (D17) holding every `facts` pair (D32, D38). A link
//! report filters on the page a link is on, a page report on the page a row
//! lists; `links` has `broken` and `errors`, `query` `stats`, `reach` how a page is
//! reached, `redirects` the moved links, `diff` what broke or was fixed
//! between two runs and `gate` what a CI run broke that its baseline had not.

mod diff;
mod error;
mod gate;
mod links;
mod query;
mod reach;
mod redirects;
mod request;
mod run;
mod sql;
mod table;
mod views;

use crate::store::Store;

pub use error::Error;
pub use gate::Gate;
pub use request::{Output, Report, Request};
pub use table::{Cell, Format, Table};
pub use views::VIEWS;

/// Runs one report against `store`, whose views are installed.
pub fn run(store: &Store, request: &Request) -> Result<Output, Error> {
    let conn = store.connection();
    let chosen = run::choose(conn, request)?;
    let (table, notes) = query::run(conn, &chosen, request)?;
    Ok(Output {
        run: chosen.run,
        unfinished: chosen.unfinished,
        from: chosen.from,
        table,
        notes,
    })
}

/// The gate over the run `request` names, whose baseline its row records.
pub fn gate(store: &Store, request: &Request) -> Result<Gate, Error> {
    let conn = store.connection();
    let chosen = run::choose(conn, request)?;
    gate::gate(conn, chosen.run, request)
}

#[cfg(test)]
mod tests;
