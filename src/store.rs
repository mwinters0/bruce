//! Storage: the DuckDB schema, its version and the single writer (D26, D28,
//! D37, D43). A [`Store`] is one database, a file or in memory. Opening it
//! creates the schema in a new database, refuses a file of another version,
//! and creates its own views and every profile's (D39); a layer above hands it more with [`Store::install`].
//! [`Store::prune`] deletes old runs (D2), and [`Store::export_baseline`]
//! writes one run as a file of its own (D21), which [`Store::import_baseline`]
//! reads back as a run, and [`Store::copy_target`] copies a target between
//! runs (D52).

mod baseline;
mod prune;
mod rows;
mod schema;
mod views;
mod writer;

use std::fmt;
use std::path::Path;

use duckdb::{Connection, params, params_from_iter};
use url::Url;

use crate::profile::{BoxError, Lookup, Registry, View};

pub use baseline::{BaselineIndex, Exported, Imported};
pub use prune::Rule;
pub use rows::{
    AnchorRow, AnsweredBy, FactRow, Hop, LinkRow, Outcome, PageRow, Row, RunRoot, TargetRow,
};
pub use schema::SCHEMA_VERSION;
pub use writer::{WriteHandle, Writer, WriterGone};

/// A run's number: allocated in order from 1 (D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunId(pub u64);

/// How a run read its pages (D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The live site, over HTTP.
    Crawl,
    /// A local build directory.
    Local,
}

impl Mode {
    /// The word the `runs` table records.
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Crawl => "crawl",
            Mode::Local => "local",
        }
    }
}

/// Why a database could not be opened or written.
#[derive(Debug)]
pub enum Error {
    /// DuckDB refused.
    Db(duckdb::Error),
    /// The file holds a bruce database of another schema version (D43).
    SchemaVersion {
        /// The version the file records.
        found: String,
        /// The version this build reads.
        expected: u32,
    },
    /// The file holds tables but no `meta`, so it is not ours to write in.
    NotBruce,
    /// No run was named, and none in the database has finished.
    NoFinishedRun,
    /// The database holds no such run.
    NoSuchRun(RunId),
    /// The run never finished, so a baseline of it is refused (D48).
    Unfinished(RunId),
    /// A file given as a baseline is not one (D52): it holds other than one
    /// run, or its run never finished.
    NotBaseline(String),
    /// The writer thread panicked.
    WriterPanicked,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Db(e) => write!(f, "{e}"),
            Error::SchemaVersion { found, expected } => write!(
                f,
                "database has schema version {found}, but this bruce reads version {expected}"
            ),
            Error::NotBruce => f.write_str("not a bruce database: it has tables but no `meta`"),
            Error::NoFinishedRun => {
                f.write_str("the database holds no finished run; name one with --run")
            }
            Error::NoSuchRun(run) => write!(f, "the database holds no run {}", run.0),
            Error::Unfinished(run) => write!(
                f,
                "run {} never finished, so its targets are incomplete; name a finished run",
                run.0
            ),
            Error::NotBaseline(why) => write!(f, "not a baseline file: {why}"),
            Error::WriterPanicked => f.write_str("the database writer panicked"),
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

/// An open database.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens the database at `path`, creating it if the file is new.
    pub fn open(path: &Path, profiles: &Registry) -> Result<Store, Error> {
        Store::init(Connection::open(path)?, profiles)
    }

    /// A database that lives only as long as the value.
    pub fn open_in_memory(profiles: &Registry) -> Result<Store, Error> {
        Store::init(Connection::open_in_memory()?, profiles)
    }

    fn init(conn: Connection, profiles: &Registry) -> Result<Store, Error> {
        if table_exists(&conn, "meta")? {
            let found: String = conn.query_row(
                "SELECT value FROM meta WHERE key = ?",
                [schema::VERSION_KEY],
                |row| row.get(0),
            )?;
            if found != SCHEMA_VERSION.to_string() {
                return Err(Error::SchemaVersion {
                    found,
                    expected: SCHEMA_VERSION,
                });
            }
        } else {
            let tables: i64 = conn.query_row(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'main'",
                [],
                |row| row.get(0),
            )?;
            if tables > 0 {
                return Err(Error::NotBruce);
            }
            conn.execute_batch(&schema::create())?;
            conn.execute(
                "INSERT INTO meta VALUES (?, ?)",
                params![schema::VERSION_KEY, SCHEMA_VERSION.to_string()],
            )?;
        }
        let store = Store { conn };
        store.install(views::ALL)?;
        for profile in profiles.iter() {
            store.install(profile.views())?;
        }
        Ok(store)
    }

    /// Creates, or replaces, each of `views`. A layer above `store` hands its
    /// own views here after every open, so an ad-hoc `duckdb` user finds them
    /// after a crawl alone (D6, D39).
    pub fn install(&self, views: &[View]) -> Result<(), Error> {
        for view in views {
            self.conn.execute_batch(&format!(
                "CREATE OR REPLACE VIEW {} AS {}",
                view.name, view.select
            ))?;
        }
        Ok(())
    }

    /// Starts a run: the next number, recorded with the tool's version, a
    /// hash of the configuration it ran under and its roots (D2): their
    /// canonical URLs, so a root that crawled nothing still shows, and the
    /// scope each hashed its pages under (D15).
    pub fn begin_run(
        &self,
        mode: Mode,
        config_hash: &str,
        baseline: Option<RunId>,
        roots: &[RunRoot],
    ) -> Result<RunId, Error> {
        let id: u64 =
            self.conn
                .query_row("SELECT coalesce(max(run_id), 0) + 1 FROM runs", [], |row| {
                    row.get(0)
                })?;
        let mut args: Vec<duckdb::types::Value> = vec![
            id.into(),
            mode.as_str().to_string().into(),
            env!("CARGO_PKG_VERSION").to_string().into(),
            config_hash.to_string().into(),
            baseline.map(|b| b.0).into(),
        ];
        let urls = list(&mut args, roots.iter().map(|r| r.url.as_str()));
        let scopes = if roots.is_empty() {
            format!("[]::{}[]", schema::SCOPE)
        } else {
            let each: Vec<String> = roots
                .iter()
                .map(|r| {
                    args.push(r.url.clone().into());
                    let root = list(&mut args, r.hash_root.iter().map(String::as_str));
                    let exclude = list(&mut args, r.hash_exclude.iter().map(String::as_str));
                    format!("{{'root': ?, 'hash_root': {root}, 'hash_exclude': {exclude}}}")
                })
                .collect();
            format!("[{}]", each.join(", "))
        };
        self.conn.execute(
            &format!("INSERT INTO runs VALUES (?, now(), NULL, ?, ?, ?, ?, {urls}, {scopes})"),
            params_from_iter(args),
        )?;
        Ok(RunId(id))
    }

    /// Marks the run finished now.
    pub fn finish_run(&self, run: RunId) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE runs SET finished_at = now() WHERE run_id = ?",
            [run.0],
        )?;
        Ok(())
    }

    /// The pages of `run`, for a profile to resolve a reference through (D39).
    pub fn lookup(&self, run: RunId) -> RunLookup<'_> {
        RunLookup { store: self, run }
    }

    /// The connection, for the reports to query (D6).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// A list of `items` as a literal, their values pushed onto `args` in the
/// order its placeholders fall.
// upstream: UF1 -- bound parameters take no list, so a list is placeholders
// inside a list literal.
fn list<'a>(args: &mut Vec<duckdb::types::Value>, items: impl Iterator<Item = &'a str>) -> String {
    let items: Vec<&str> = items.collect();
    if items.is_empty() {
        return "[]::VARCHAR[]".to_string();
    }
    args.extend(
        items
            .iter()
            .map(|i| duckdb::types::Value::from((*i).to_string())),
    );
    format!("[{}]", vec!["?"; items.len()].join(", "))
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool, Error> {
    let n: i64 = conn.query_row(
        "SELECT count(*) FROM information_schema.tables \
         WHERE table_schema = 'main' AND table_name = ?",
        [name],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

/// One run's pages, as a profile's [`Lookup`].
pub struct RunLookup<'a> {
    store: &'a Store,
    run: RunId,
}

impl Lookup for RunLookup<'_> {
    fn pages_with_facts(&self, facts: &[(&str, &str)]) -> Result<Vec<Url>, BoxError> {
        let mut sql = String::from(
            "SELECT t.url FROM pages p \
             JOIN targets t ON t.run_id = p.run_id AND t.url_id = p.url_id \
             WHERE p.run_id = ?",
        );
        let mut args: Vec<duckdb::types::Value> = vec![self.run.0.into()];
        for (key, value) in facts {
            sql.push_str(
                " AND EXISTS (SELECT 1 FROM page_facts f WHERE f.run_id = p.run_id \
                 AND f.url_id = p.url_id AND f.key = ? AND f.value = ?)",
            );
            args.push((*key).to_string().into());
            args.push((*value).to_string().into());
        }
        let mut stmt = self.store.conn.prepare(&sql)?;
        let urls = stmt
            .query_map(params_from_iter(args), |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        urls.iter()
            .map(|u| Url::parse(u).map_err(BoxError::from))
            .collect()
    }

    fn facts(&self, page: &Url) -> Result<Vec<(String, String)>, BoxError> {
        let mut stmt = self.store.conn.prepare(
            "SELECT f.key, f.value FROM page_facts f \
             JOIN targets t ON t.run_id = f.run_id AND t.url_id = f.url_id \
             WHERE f.run_id = ? AND t.url = ?",
        )?;
        let facts = stmt
            .query_map(params![self.run.0, page.as_str()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<_, _>>()?;
        Ok(facts)
    }
}

#[cfg(test)]
mod tests;
