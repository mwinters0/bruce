//! The single writer (D28): one thread owns the [`Store`] and feeds DuckDB
//! appenders from a bounded channel, so a slow disk pushes back on the
//! crawl and DuckDB sees one writing process. It handles the rows of the two
//! tables the extractor's first output fills, `pages` and `page_facts`; the
//! other tables' rows join [`Row`] with the phase that produces them.

use std::fmt;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;

use duckdb::params;

use super::{Error, Store};

/// Rows an appender takes before it is flushed to the file.
const FLUSH_EVERY: usize = 10_000;

/// A row of `pages` (D26, D34–D36).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRow {
    pub run_id: u64,
    pub url_id: u64,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub content_hash: Option<Vec<u8>>,
    /// The name of the profile that extracted the page (D35).
    pub profile: String,
    pub source_url: Option<String>,
    pub source_repo: Option<String>,
    pub source_ref: Option<String>,
    pub source_path: Option<String>,
    pub in_sitemap: bool,
    pub via_links: bool,
}

/// A row of `page_facts` (D37): `key` is the full `<profile>.<key>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactRow {
    pub run_id: u64,
    pub url_id: u64,
    pub key: String,
    pub value: String,
}

/// One row for the writer to append.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    Page(PageRow),
    Fact(FactRow),
}

enum Msg {
    Row(Box<Row>),
    Finish,
}

/// The writer thread, started with [`Writer::start`] and ended with
/// [`Writer::finish`], which hands the store back.
pub struct Writer {
    tx: SyncSender<Msg>,
    thread: JoinHandle<Result<Store, Error>>,
}

/// A clonable way to send rows to the writer.
#[derive(Clone)]
pub struct WriteHandle {
    tx: SyncSender<Msg>,
}

/// The writer has stopped, after an error or a finish. The error is
/// [`Writer::finish`]'s to return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriterGone;

impl fmt::Display for WriterGone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the database writer has stopped")
    }
}

impl std::error::Error for WriterGone {}

impl Writer {
    /// Moves `store` onto a writer thread behind a channel holding at most
    /// `capacity` rows (D28).
    pub fn start(store: Store, capacity: usize) -> Writer {
        let (tx, rx) = sync_channel(capacity);
        let thread = std::thread::spawn(move || write_all(store, rx));
        Writer { tx, thread }
    }

    /// A handle that sends rows, blocking while the channel is full.
    pub fn handle(&self) -> WriteHandle {
        WriteHandle {
            tx: self.tx.clone(),
        }
    }

    /// Flushes what was sent before this call, stops the thread and returns
    /// the store, or the error that stopped it.
    pub fn finish(self) -> Result<Store, Error> {
        // A send fails only when the thread already stopped; its result is
        // the join's.
        let _ = self.tx.send(Msg::Finish);
        self.thread.join().map_err(|_| Error::WriterPanicked)?
    }
}

impl WriteHandle {
    /// Sends `row`, waiting while the channel is full.
    pub fn send(&self, row: Row) -> Result<(), WriterGone> {
        self.tx
            .send(Msg::Row(Box::new(row)))
            .map_err(|_| WriterGone)
    }
}

fn write_all(store: Store, rx: Receiver<Msg>) -> Result<Store, Error> {
    {
        let mut pages = store.conn.appender("pages")?;
        let mut facts = store.conn.appender("page_facts")?;
        let mut pending = 0;
        for msg in rx {
            let Msg::Row(row) = msg else { break };
            match *row {
                Row::Page(p) => pages.append_row(params![
                    p.run_id,
                    p.url_id,
                    p.title,
                    p.content_type,
                    p.size,
                    p.content_hash,
                    p.profile,
                    p.source_url,
                    p.source_repo,
                    p.source_ref,
                    p.source_path,
                    p.in_sitemap,
                    p.via_links
                ])?,
                Row::Fact(f) => facts.append_row(params![f.run_id, f.url_id, f.key, f.value])?,
            }
            pending += 1;
            if pending == FLUSH_EVERY {
                pages.flush()?;
                facts.flush()?;
                pending = 0;
            }
        }
        pages.flush()?;
        facts.flush()?;
    }
    Ok(store)
}
