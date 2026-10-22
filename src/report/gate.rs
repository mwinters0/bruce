//! The gate: what a CI run broke that its baseline had not (D9), and the
//! sections `check-dir` prints beside it.
//!
//! A link is keyed by its page's `url_id` (D26), its target URL and its
//! fragment, the same in every run, and a run's `baseline_run` names the run it was checked
//! against. A CI run holds the links of built pages alone, a copied page
//! having none (D52), so a baseline link on a page not built is absent from
//! it and is no one's to call fixed. **Introduced** is a link broken in the
//! run whose key the baseline does not have broken: absent, `ok`,
//! `unchecked` or `error`, which the row names. Several links sharing a key
//! are one row. `error` never fails (D16) and is listed, and breakage the
//! baseline had stays a count until asked for. The **inverse** section is
//! the baseline's links from pages outside the build to what the change
//! removed (`inverse`); it fails the gate while the request's mode is `Fail`.

use duckdb::Connection;

use super::error::Error;
use super::inverse::{counts, inverse};
use super::links::links;
use super::request::{InverseMode, Request};
use super::sql::{Cols, Sql};
use super::table::{Format, Table, quoted};
use crate::store::RunId;

/// The filter columns of a `report_broken` or `report_redirects` row aliased `b`.
const LINK: Cols = Cols {
    run: "b.run_id",
    repo: "b.source_repo",
    page: "b.src_url_id",
};

/// A baseline link `o` has the key of the CI link `b`.
const SAME_KEY: &str = "o.src_url_id = b.src_url_id AND o.target IS NOT DISTINCT FROM b.target \
                        AND o.frag = coalesce(b.fragment, '')";

/// The outcomes that are breakage.
const BROKEN: &str = "('broken_url', 'broken_anchor')";

/// The gate's sections over one CI run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub run: RunId,
    pub baseline: RunId,
    /// The baseline run's date, which its verdicts are as old as.
    pub baseline_date: String,
    /// Broken here and not in the baseline, each with its `baseline` state:
    /// `new` where the baseline has no such link. These fail the gate.
    pub introduced: Table,
    /// Baseline links from pages outside the build to what this run removed
    /// from it. These fail the gate unless `mode` is `Warn`.
    pub inverse: Table,
    /// Whether `inverse` fails the gate (`--inverse`).
    pub mode: InverseMode,
    /// Redirected here and not in the baseline. Never failing (D7).
    pub redirected: Table,
    /// Links whose outcome is `error`. Never failing (D16).
    pub errors: Table,
    /// Broken here and in the baseline.
    pub already: Table,
}

impl Gate {
    /// The gate fails: a link is introduced, or an inverse breakage is found
    /// while the mode is `Fail`.
    pub fn fails(&self) -> bool {
        !self.introduced.rows.is_empty()
            || (self.mode == InverseMode::Fail && !self.inverse.rows.is_empty())
    }

    /// The text `check-dir` prints: the baseline, every section, and with
    /// `all` the breakage already in the baseline in full.
    pub fn text(&self, all: bool) -> String {
        let mut out = self.baseline_line();
        out.push_str("\n\n");
        out.push_str(&section("introduced breakage", &self.introduced, true));
        out.push_str("\n\n");
        out.push_str(&self.rest(all));
        out
    }

    /// The baseline a run was checked against.
    pub(super) fn baseline_line(&self) -> String {
        baseline_line(self.baseline, &self.baseline_date)
    }

    /// The sections after the introduced breakage: the inverse breakage, which
    /// fails under `--inverse=fail`, and the rest, which never fail.
    pub(super) fn rest(&self, all: bool) -> String {
        [
            self.inverse_section(),
            section("introduced redirected links", &self.redirected, true),
            section("errors", &self.errors, true),
            section("breakage already in the baseline", &self.already, all)
                + if all || self.already.rows.is_empty() {
                    ""
                } else {
                    " (--all lists them)"
                },
        ]
        .join("\n\n")
    }

    /// The inverse breakage, and how many pages across how many components
    /// link to what was removed (D56).
    fn inverse_section(&self) -> String {
        let title = match self.mode {
            InverseMode::Fail => "inverse breakage",
            InverseMode::Warn => "inverse breakage (warning only)",
        };
        let links = self.inverse.rows.len();
        if links == 0 {
            return format!("{title}: none");
        }
        let (pages, groups) = counts(&self.inverse);
        format!(
            "{title}: {links} links, {pages} inbound pages across {groups} components\n{}",
            self.inverse.render(Format::Text)
        )
    }

    /// One JSON object holding every section; the breakage already in the
    /// baseline is a count, and its links only with `all`.
    pub fn json(&self, all: bool) -> String {
        let links = if all {
            self.already.render(Format::Json)
        } else {
            "null".to_owned()
        };
        format!(
            "{{\"run\":{},\"baseline_run\":{},\"baseline_date\":{},\"introduced\":{},\"inverse\":{},\
             \"redirected\":{},\"errors\":{},\"already_broken\":{{\"count\":{},\"links\":{}}}}}",
            self.run.0,
            self.baseline.0,
            quoted(&self.baseline_date),
            self.introduced.render(Format::Json),
            self.inverse.render(Format::Json),
            self.redirected.render(Format::Json),
            self.errors.render(Format::Json),
            self.already.rows.len(),
            links,
        )
    }
}

/// A title and the number of rows, and with `rows` the table.
fn section(title: &str, table: &Table, rows: bool) -> String {
    match table.rows.len() {
        0 => format!("{title}: none"),
        n if rows => format!("{title}: {n}\n{}", table.render(Format::Text)),
        n => format!("{title}: {n}"),
    }
}

/// The gate over `run`, the CI run, and the baseline run it names.
pub(super) fn gate(conn: &Connection, run: RunId, request: &Request) -> Result<Gate, Error> {
    // deficiency: KD20 Each section is fetched under `--top`'s limit and its
    // counts are taken from the rows fetched, so `report introduced --top N`
    // and `report inverse --top N` count the breakage already in the
    // baseline, the inverse links and their inbound pages and components over
    // N rows at most; `stats` counts before it cuts. `check-dir` sets no limit.
    let (baseline, baseline_date) = baseline(conn, run)?;
    Ok(Gate {
        run,
        baseline,
        baseline_date,
        introduced: broken(conn, run, baseline, request, true)?,
        inverse: inverse(conn, run, baseline, request)?,
        mode: request.inverse,
        redirected: redirected(conn, run, baseline, request)?,
        errors: links(conn, run, request, "report_errors")?,
        already: broken(conn, run, baseline, request, false)?,
    })
}

/// The line naming the baseline a run was checked against.
pub(super) fn baseline_line(baseline: RunId, date: &str) -> String {
    format!("against baseline run {}, crawled on {date}", baseline.0)
}

/// The run `run` was checked against and its date.
pub(super) fn baseline(conn: &Connection, run: RunId) -> Result<(RunId, String), Error> {
    let named: Option<u64> = conn.query_row(
        "SELECT baseline_run FROM runs WHERE run_id = ?",
        [run.0],
        |row| row.get(0),
    )?;
    let base = RunId(named.ok_or(Error::NoBaseline(run))?);
    let mut stmt =
        // `::DATE` does not cast a `TIMESTAMPTZ`; through `::TIMESTAMP` it does.
        conn.prepare("SELECT started_at::TIMESTAMP::DATE::VARCHAR FROM runs WHERE run_id = ?")?;
    let mut rows = stmt.query([base.0])?;
    match rows.next()? {
        Some(row) => Ok((base, row.get(0)?)),
        None => Err(Error::NoSuchRun(base)),
    }
}

/// `base`, the links of the baseline run with the key a CI link is matched on.
fn base_links(sql: &mut Sql, base: RunId, request: &Request) {
    sql.push(
        "WITH base AS (SELECT v.src_url_id, t.url AS target, coalesce(v.fragment, '') AS frag, \
         v.outcome, v.redirected FROM link_verdicts v \
         LEFT JOIN targets t ON t.run_id = v.run_id AND t.url_id = v.target_url_id \
         WHERE v.run_id = ",
    )
    .bind(base.0)
    .push(" AND ")
    .kinds("v.kind", request)
    .push(") ");
}

/// The broken links of `run` that the baseline has not broken, with
/// `introduced`, else those it has.
fn broken(
    conn: &Connection,
    run: RunId,
    base: RunId,
    request: &Request,
    introduced: bool,
) -> Result<Table, Error> {
    let mut sql = Sql::new("");
    base_links(&mut sql, base, request);
    sql.push(
        "SELECT * FROM (SELECT b.page, b.raw_href, b.target, b.fragment, b.kind, b.outcome, \
         b.reason, b.status::INTEGER AS status, ",
    );
    if introduced {
        // The least certain state when a key holds several links.
        sql.push(
            "coalesce((SELECT arg_min(o.outcome, CASE o.outcome WHEN 'error' THEN 0 \
             WHEN 'unchecked' THEN 1 ELSE 2 END) FROM base o WHERE ",
        )
        .push(SAME_KEY)
        .push("), 'new') AS baseline, ");
    }
    sql.push("b.source_repo, b.source_path FROM report_broken b WHERE ")
        .kinds("b.kind", request)
        .push(" AND ");
    sql.scope(run, request, LINK)
        .push(if introduced {
            " AND NOT EXISTS (SELECT 1 FROM base o WHERE "
        } else {
            " AND EXISTS (SELECT 1 FROM base o WHERE "
        })
        .push(SAME_KEY)
        .push(" AND o.outcome IN ")
        .push(BROKEN)
        .push(
            ") QUALIFY row_number() OVER (PARTITION BY b.src_url_id, b.target, \
             coalesce(b.fragment, '') ORDER BY b.raw_href, b.kind) = 1) \
             ORDER BY outcome DESC, page, raw_href, fragment",
        )
        .limit(request);
    sql.fetch(conn)
}

/// The redirected links of `run` that the baseline has not redirected.
fn redirected(
    conn: &Connection,
    run: RunId,
    base: RunId,
    request: &Request,
) -> Result<Table, Error> {
    let mut sql = Sql::new("");
    base_links(&mut sql, base, request);
    sql.push(
        "SELECT * FROM (SELECT b.page, b.raw_href, b.target, b.fragment, b.kind, b.landing, \
         b.chain, b.source_repo, b.source_path FROM report_redirects b WHERE ",
    )
    .kinds("b.kind", request)
    .push(" AND ");
    sql.scope(run, request, LINK)
        .push(" AND NOT EXISTS (SELECT 1 FROM base o WHERE ")
        .push(SAME_KEY)
        .push(
            " AND o.redirected) QUALIFY row_number() OVER (PARTITION BY b.src_url_id, b.target, \
             coalesce(b.fragment, '') ORDER BY b.raw_href, b.kind) = 1) \
             ORDER BY page, raw_href, fragment",
        )
        .limit(request);
    sql.fetch(conn)
}

#[cfg(test)]
mod tests;
