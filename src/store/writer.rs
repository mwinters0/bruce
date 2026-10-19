//! The single writer (D28): one thread owns the [`Store`] and feeds DuckDB
//! appenders from a bounded channel, so a slow disk pushes back on the
//! crawl and DuckDB sees one writing process. It writes the rows of
//! [`Row`], one table each but `bodies`, which waits for its option.

use std::fmt;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;

use duckdb::types::{Null, Value};
use duckdb::{Appender, params, params_from_iter};

use super::rows::{Hop, Row, TargetRow, timestamp};
use super::{Error, Store};

/// Rows the writer takes, counted across tables, before it flushes every
/// appender.
const FLUSH_EVERY: usize = 10_000;

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
        let mut tables = Tables::new(&store)?;
        let mut pending = 0;
        for msg in rx {
            let Msg::Row(row) = msg else { break };
            tables.append(*row)?;
            pending += 1;
            if pending == FLUSH_EVERY {
                tables.flush()?;
                pending = 0;
            }
        }
        tables.flush()?;
    }
    Ok(store)
}

/// An appender for each table the writer fills.
struct Tables<'a> {
    store: &'a Store,
    pages: Appender<'a>,
    facts: Appender<'a>,
    targets: Appender<'a>,
    anchors: Appender<'a>,
    links: Appender<'a>,
}

impl<'a> Tables<'a> {
    fn new(store: &'a Store) -> Result<Tables<'a>, Error> {
        Ok(Tables {
            store,
            pages: store.conn.appender("pages")?,
            facts: store.conn.appender("page_facts")?,
            targets: store.conn.appender("targets")?,
            anchors: store.conn.appender("anchors")?,
            links: store.conn.appender("links")?,
        })
    }

    fn append(&mut self, row: Row) -> Result<(), Error> {
        match row {
            Row::Page(p) => self.pages.append_row(params![
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
                p.via_links,
                p.root
            ])?,
            Row::Fact(f) => self
                .facts
                .append_row(params![f.run_id, f.url_id, f.key, f.value])?,
            Row::Target(t) if t.redirect_chain.is_empty() => {
                self.targets.append_row(params![
                    t.run_id,
                    t.url_id,
                    t.url,
                    t.status,
                    t.final_url,
                    Null,
                    t.outcome.as_str(),
                    t.reason,
                    t.checked_at.map(timestamp),
                    t.duration_ms
                ])?;
            }
            Row::Target(t) => self.insert_redirected(&t)?,
            Row::Anchor(a) => self.anchors.append_row(params![
                a.run_id,
                a.url_id,
                a.anchor,
                a.tag,
                a.placement
            ])?,
            Row::Link(l) => self.links.append_row(params![
                l.run_id,
                l.src_url_id,
                l.target_url_id,
                l.fragment,
                l.raw_href,
                l.kind,
                l.xref_refspec
            ])?,
        }
        Ok(())
    }

    /// Inserts a target with a redirect chain by statement, the chain's
    /// values bound inside a list literal. The statement is cached per chain
    /// length, at most D7's limit and one more.
    // upstream: UF1 -- the appender and bound parameters take no list.
    fn insert_redirected(&self, t: &TargetRow) -> Result<(), Error> {
        let hops = vec![
            "{'url': ?::VARCHAR, 'status': ?::USMALLINT, 'refresh': ?::BOOLEAN}";
            t.redirect_chain.len()
        ];
        let sql = format!(
            "INSERT INTO targets VALUES (?, ?, ?, ?, ?, [{}], ?, ?, ?, ?)",
            hops.join(", ")
        );
        let mut args: Vec<Value> = vec![
            t.run_id.into(),
            t.url_id.into(),
            t.url.clone().into(),
            t.status.into(),
            t.final_url.clone().into(),
        ];
        for Hop {
            url,
            status,
            refresh,
        } in &t.redirect_chain
        {
            args.extend([url.clone().into(), (*status).into(), (*refresh).into()]);
        }
        args.extend([
            t.outcome.as_str().to_string().into(),
            t.reason.clone().into(),
            t.checked_at.map_or(Value::Null, timestamp),
            t.duration_ms.into(),
        ]);
        self.store
            .conn
            .prepare_cached(&sql)?
            .execute(params_from_iter(args))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.pages.flush()?;
        self.facts.flush()?;
        self.targets.flush()?;
        self.anchors.flush()?;
        self.links.flush()?;
        Ok(())
    }
}
