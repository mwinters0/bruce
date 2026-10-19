//! The statement every report builds: its text, its bound arguments and the
//! filters they share. Every value is bound, never written into the text.

use duckdb::types::Value;
use duckdb::{Connection, params_from_iter};

use super::error::Error;
use super::request::Request;
use super::table::{Cell, Table};
use crate::store::RunId;

/// A statement and its arguments, kept in the order the text names them.
pub(super) struct Sql {
    text: String,
    args: Vec<Value>,
}

impl Sql {
    pub(super) fn new(text: &str) -> Sql {
        Sql {
            text: text.to_owned(),
            args: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, text: &str) -> &mut Sql {
        self.text.push_str(text);
        self
    }

    /// Appends a placeholder for `value`.
    pub(super) fn bind(&mut self, value: impl Into<Value>) -> &mut Sql {
        self.args.push(value.into());
        self.text.push('?');
        self
    }

    /// Appends a placeholder for the text `s`.
    pub(super) fn bind_text(&mut self, s: &str) -> &mut Sql {
        self.bind(Value::Text(s.to_owned()))
    }

    /// `kind IN (…)` over `column`. A bound list is not supported (UF1), so
    /// the placeholders are written out.
    pub(super) fn kinds(&mut self, column: &str, request: &Request) -> &mut Sql {
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
    pub(super) fn scope(&mut self, run_id: RunId, request: &Request, cols: Cols) -> &mut Sql {
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

    pub(super) fn limit(&mut self, request: &Request) -> &mut Sql {
        if let Some(top) = request.top {
            self.push(" LIMIT ").push(&top.to_string());
        }
        self
    }

    pub(super) fn fetch(&self, conn: &Connection) -> Result<Table, Error> {
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
pub(super) struct Cols {
    pub(super) run: &'static str,
    pub(super) repo: &'static str,
    pub(super) page: &'static str,
}

/// The filter columns of a `report_pages` row aliased `pg`.
pub(super) const PAGE: Cols = Cols {
    run: "pg.run_id",
    repo: "pg.source_repo",
    page: "pg.url_id",
};
