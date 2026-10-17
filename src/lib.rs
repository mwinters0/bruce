//! `bruce`, a link checker that validates anchors and keeps a link graph with
//! run history. Its concepts are top-level modules, layered as
//! `docs/design/decisions.md`, "D39" records: `cli` › `crawl`, `ci`, `report`
//! › `fetch`, `extract`, `store` › `profile` › `forge`, `url`, `config`, edges
//! pointing down only.

pub mod ci;
pub mod cli;
pub mod config;
pub mod crawl;
pub mod extract;
pub mod fetch;
pub mod forge;
pub mod profile;
pub mod report;
pub mod store;
pub mod url;
