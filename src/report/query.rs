//! The query each report runs: its view, the run, the filters, the order and
//! the limit. Every value is bound, never written into the text.

use duckdb::types::Value;
use duckdb::{Connection, params_from_iter};

use super::error::Error;
use super::request::{Report, Request};
use super::table::{Cell, Table};
use crate::store::RunId;

/// A statement and its arguments, kept in the order the text names them.
struct Sql {
    text: String,
    args: Vec<Value>,
}

impl Sql {
    fn new(text: &str) -> Sql {
        Sql {
            text: text.to_owned(),
            args: Vec::new(),
        }
    }

    fn push(&mut self, text: &str) -> &mut Sql {
        self.text.push_str(text);
        self
    }

    /// Appends a placeholder for `value`.
    fn bind(&mut self, value: impl Into<Value>) -> &mut Sql {
        self.args.push(value.into());
        self.text.push('?');
        self
    }

    /// Appends a placeholder for the text `s`.
    fn bind_text(&mut self, s: &str) -> &mut Sql {
        self.bind(Value::Text(s.to_owned()))
    }

    /// `kind IN (…)` over `column`. A bound list is not supported (UF1), so
    /// the placeholders are written out.
    fn kinds(&mut self, column: &str, request: &Request) -> &mut Sql {
        self.push(column).push(" IN (");
        for (i, kind) in request.kinds.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.bind_text(kind.as_str());
        }
        self.push(")")
    }

    /// The run, the repo and the facts. `run`, `repo` and `page` name the
    /// columns of the relation being filtered; `page` is the linking page's
    /// `url_id`.
    fn scope(&mut self, run_id: RunId, request: &Request, cols: Cols) -> &mut Sql {
        self.push(cols.run).push(" = ").bind(run_id.0);
        if let Some(repo) = &request.repo {
            self.push(" AND ")
                .push(cols.repo)
                .push(" = ")
                .bind_text(repo);
        }
        for (key, value) in &request.facts {
            self.push(" AND EXISTS (SELECT 1 FROM page_facts f WHERE f.run_id = ")
                .push(cols.run)
                .push(" AND f.url_id = ")
                .push(cols.page)
                .push(" AND f.key = ")
                .bind_text(key)
                .push(" AND f.value = ")
                .bind_text(value)
                .push(")");
        }
        self
    }

    fn limit(&mut self, request: &Request) -> &mut Sql {
        if let Some(top) = request.top {
            self.push(" LIMIT ").push(&top.to_string());
        }
        self
    }

    fn fetch(&self, conn: &Connection) -> Result<Table, Error> {
        let mut stmt = conn.prepare(&self.text)?;
        let mut rows = stmt.query(params_from_iter(self.args.iter()))?;
        let width = rows.as_ref().map_or(0, |s| s.column_count());
        let mut cells = Vec::new();
        while let Some(row) = rows.next()? {
            let mut cols = Vec::with_capacity(width);
            for i in 0..width {
                cols.push(Cell::from(row.get::<_, Value>(i)?));
            }
            cells.push(cols);
        }
        let columns = rows
            .as_ref()
            .map(|s| s.column_names())
            .unwrap_or_default()
            .into_iter()
            .collect();
        Ok(Table {
            columns,
            rows: cells,
        })
    }
}

/// The columns of the relation a filter applies to.
#[derive(Clone, Copy)]
struct Cols {
    run: &'static str,
    repo: &'static str,
    page: &'static str,
}

/// Runs the report, returning its table and the notes a text rendering adds.
pub(super) fn run(
    conn: &Connection,
    run: RunId,
    request: &Request,
) -> Result<(Table, Vec<String>), Error> {
    match request.report {
        Report::Broken => Ok((links(conn, run, request, "report_broken")?, Vec::new())),
        Report::Errors => Ok((links(conn, run, request, "report_errors")?, Vec::new())),
        Report::Stats => stats(conn, run, request),
    }
}

/// A row per link of a `report_*` link view, the worst outcome first.
fn links(conn: &Connection, run: RunId, request: &Request, view: &str) -> Result<Table, Error> {
    let mut sql = Sql::new(
        "SELECT page, raw_href, target, fragment, kind, outcome, reason, status::INTEGER AS status, \
         source_repo, source_path FROM ",
    );
    sql.push(view)
        .push(" WHERE ")
        .kinds("kind", request)
        .push(" AND ");
    sql.scope(
        run,
        request,
        Cols {
            run: "run_id",
            repo: "source_repo",
            page: "src_url_id",
        },
    );
    sql.push(" ORDER BY outcome DESC, page, raw_href, fragment")
        .limit(request);
    sql.fetch(conn)
}

/// A row per crawled page with its counts, the most broken first, and the run's
/// totals in the notes.
fn stats(conn: &Connection, run: RunId, request: &Request) -> Result<(Table, Vec<String>), Error> {
    let mut sql = Sql::new(
        "SELECT * FROM (SELECT pg.url AS page, coalesce(sum(s.links), 0)::BIGINT AS links, \
         coalesce(sum(s.broken_url), 0)::BIGINT AS broken_url, \
         coalesce(sum(s.broken_anchor), 0)::BIGINT AS broken_anchor, \
         coalesce(sum(s.error), 0)::BIGINT AS error, \
         coalesce(sum(s.unchecked), 0)::BIGINT AS unchecked \
         FROM report_pages pg LEFT JOIN report_stats s \
         ON s.run_id = pg.run_id AND s.url_id = pg.url_id AND ",
    );
    sql.kinds("s.kind", request).push(" WHERE pg.crawled AND ");
    sql.scope(run, request, PAGE).push(
        " GROUP BY pg.url_id, pg.url) \
         ORDER BY broken_url + broken_anchor DESC, error DESC, links DESC, page",
    );
    let mut table = sql.fetch(conn)?;
    let notes = totals(conn, run, request, &table)?;
    if let Some(top) = request.top {
        table.rows.truncate(top);
    }
    Ok((table, notes))
}

const PAGE: Cols = Cols {
    run: "pg.run_id",
    repo: "pg.source_repo",
    page: "pg.url_id",
};

/// The run's totals over every page the filters keep, `unchecked` links
/// counted by reason, which no row carries.
fn totals(
    conn: &Connection,
    run: RunId,
    request: &Request,
    pages: &Table,
) -> Result<Vec<String>, Error> {
    let sum = |col: usize| -> i64 {
        pages
            .rows
            .iter()
            .map(|r| match r[col] {
                Cell::Int(n) => n,
                _ => 0,
            })
            .sum()
    };
    let mut line = format!(
        "total: {} pages, {} links, {} broken_url, {} broken_anchor, {} error, {} unchecked",
        pages.rows.len(),
        sum(1),
        sum(2),
        sum(3),
        sum(4),
        sum(5)
    );
    let mut sql = Sql::new(
        "SELECT coalesce(v.reason, '(none)') AS reason, count(*)::BIGINT AS links \
         FROM link_verdicts v JOIN report_pages pg \
         ON pg.run_id = v.run_id AND pg.url_id = v.src_url_id \
         WHERE pg.crawled AND v.outcome = 'unchecked' AND ",
    );
    sql.kinds("v.kind", request).push(" AND ");
    sql.scope(run, request, PAGE)
        .push(" GROUP BY 1 ORDER BY 2 DESC, 1");
    let reasons = sql.fetch(conn)?;
    if reasons.rows.is_empty() {
        return Ok(vec![line]);
    }
    line.push_str("\nunchecked links by reason:");
    for row in &reasons.rows {
        if let [Cell::Text(reason), Cell::Int(n)] = &row[..] {
            line.push_str(&format!("\n  {reason}: {n}"));
        }
    }
    Ok(vec![line])
}
