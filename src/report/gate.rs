//! The gate: what a CI run broke that its baseline had not (D9), and the
//! sections `check-dir` prints beside it.
//!
//! A link is keyed by its page's `url_id` (D26), its target URL and its
//! fragment, the same in every run, and a run's `baseline_run` names the run it was checked
//! against. A CI run holds the links of built pages alone, a copied page
//! having none (D52), so a baseline link on a page not built is absent from
//! it and is no one's to call fixed. **Introduced** is a link whose outcome
//! the CI run's `fail` list names (D83; `broken_url` and `broken_anchor` by
//! default) and whose key the baseline does not have at one of them: absent,
//! `ok`, `unchecked` or another outcome, which the row names. Several links
//! sharing a key are one row. An `error` fails only where the list names it
//! (D16) and is listed either way, and breakage the baseline had stays a
//! count until asked for. A `broken_url` or `broken_anchor` the list leaves
//! out is **introduced, not failing** where the baseline lacks the key at one
//! of the left-out outcomes, a section of its own (D83). The **inverse**
//! section is the baseline's links from pages outside the build to what the
//! change removed (`inverse`): those whose outcome, once the change deploys,
//! the list names fail the gate while the request's mode is `Fail`, the rest
//! are listed apart.

use duckdb::Connection;

use super::error::Error;
use super::inverse::{counts, inverse, split};
use super::links::errors;
use super::request::{InverseMode, Request};
use super::sql::{Cols, Sql, pedantic};
use super::table::{Format, Table, quoted};
use crate::config::FailOn;
use crate::store::{JudgedBy, RunId};

/// The filter columns of a `report_broken` or `report_redirects` row aliased `b`.
const LINK: Cols = Cols {
    run: "b.run_id",
    repo: "b.source_repo",
    page: "b.src_url_id",
};

/// A baseline link `o` has the key of the CI link `b`.
const SAME_KEY: &str = "o.src_url_id = b.src_url_id AND o.target IS NOT DISTINCT FROM b.target \
                        AND o.frag = coalesce(b.fragment, '')";

/// The gate's sections over one CI run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub run: RunId,
    pub baseline: RunId,
    /// The baseline run's date, which its verdicts are as old as.
    pub baseline_date: String,
    /// Broken here and not in the baseline, each with its `baseline` state:
    /// `new` where the baseline has no such link. These fail the gate, in the
    /// outcomes of the CI run's `fail` list (D83).
    pub introduced: Table,
    /// Broken here and not in the baseline in an outcome the `fail` list leaves
    /// out, with the same columns. Never failing (D83).
    pub introduced_not_failing: Table,
    /// Baseline links from pages outside the build to what this run removed
    /// from it, to an outcome the `fail` list names. These fail the gate
    /// unless `mode` is `Warn`.
    pub inverse: Table,
    /// The same, to an outcome the list leaves out. Never failing (D83).
    pub inverse_not_failing: Table,
    /// Whether `inverse` fails the gate (`--inverse`).
    pub mode: InverseMode,
    /// Redirected here and not in the baseline. Never failing (D7).
    pub redirected: Table,
    /// Links whose outcome is `error`, listed whether or not the `fail` list
    /// names it (D16).
    pub errors: Table,
    /// Failing here and in the baseline, in the outcomes of the same list.
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
        let title = match self.mode {
            InverseMode::Fail => "inverse breakage",
            InverseMode::Warn => "inverse breakage (warning only)",
        };
        [
            section(
                "introduced breakage not failing",
                &self.introduced_not_failing,
                true,
            ),
            inverse_section(title, &self.inverse),
            inverse_section("inverse breakage not failing", &self.inverse_not_failing),
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

    /// One JSON object holding every section; the breakage already in the
    /// baseline is a count, and its links only with `all`.
    pub fn json(&self, all: bool) -> String {
        let links = if all {
            self.already.render(Format::Json)
        } else {
            "null".to_owned()
        };
        format!(
            "{{\"run\":{},\"baseline_run\":{},\"baseline_date\":{},\"introduced\":{},\
             \"introduced_not_failing\":{},\"inverse\":{},\"inverse_not_failing\":{},\
             \"redirected\":{},\"errors\":{},\"already_broken\":{{\"count\":{},\"links\":{}}}}}",
            self.run.0,
            self.baseline.0,
            quoted(&self.baseline_date),
            self.introduced.render(Format::Json),
            self.introduced_not_failing.render(Format::Json),
            self.inverse.render(Format::Json),
            self.inverse_not_failing.render(Format::Json),
            self.redirected.render(Format::Json),
            self.errors.render(Format::Json),
            self.already.rows.len(),
            links,
        )
    }
}

/// An inverse section: its links, and how many pages across how many
/// components link to what was removed (D56).
fn inverse_section(title: &str, table: &Table) -> String {
    let links = table.rows.len();
    if links == 0 {
        return format!("{title}: none");
    }
    let (pages, groups) = counts(table);
    format!(
        "{title}: {links} links, {pages} inbound pages across {groups} components\n{}",
        table.render(Format::Text)
    )
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
    // The baseline's links are judged by the CI run's policy (D81).
    let _judged = JudgedBy::new(conn, run.0)?;
    let fail = recorded_fail(conn, run)?;
    let left_out = left_out(&fail);
    let (inverse, inverse_not_failing) = split(inverse(conn, run, baseline, request)?, &fail);
    Ok(Gate {
        run,
        baseline,
        baseline_date,
        introduced: broken(conn, run, baseline, request, &fail, true)?,
        introduced_not_failing: broken(conn, run, baseline, request, &left_out, true)?,
        inverse,
        inverse_not_failing,
        mode: request.inverse,
        redirected: redirected(conn, run, baseline, request)?,
        errors: errors(conn, run, request)?,
        already: broken(conn, run, baseline, request, &fail, false)?,
    })
}

/// The outcomes the policy in force for the CI run fails the gate on (D83):
/// the recorded one, or a trial's (D84); the default where it has no row.
fn recorded_fail(conn: &Connection, run: RunId) -> Result<Vec<FailOn>, Error> {
    let value: Option<String> = conn.query_row(
        "SELECT max(value) FROM policy_in_force WHERE run_id = ? AND host IS NULL AND key = 'gate.fail'",
        [run.0],
        |row| row.get(0),
    )?;
    Ok(FailOn::from_row(value.as_deref()))
}

/// The broken outcomes `fail` leaves out: what is reported and fails nothing
/// (D83). An `error` is listed in its own section either way.
fn left_out(fail: &[FailOn]) -> Vec<FailOn> {
    [FailOn::BrokenUrl, FailOn::BrokenAnchor]
        .into_iter()
        .filter(|o| !fail.contains(o))
        .collect()
}

/// The outcomes as a SQL list, which matches nothing where there are none. They
/// are a closed set, written into the text.
fn outcomes(fail: &[FailOn]) -> String {
    if fail.is_empty() {
        return "(NULL)".to_owned();
    }
    let quoted: Vec<String> = fail.iter().map(|o| format!("'{}'", o.as_str())).collect();
    format!("({})", quoted.join(", "))
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

/// The links of `run` in an outcome of `fail` (the outcomes the gate fails on,
/// or the ones it leaves out) that the baseline does not have at one, with
/// `introduced`, else those it has.
fn broken(
    conn: &Connection,
    run: RunId,
    base: RunId,
    request: &Request,
    fail: &[FailOn],
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
    // An `error` is no `report_broken` row, so its view joins by name where the
    // list has it; `hint` is the broken rows' alone.
    sql.push(if fail.contains(&FailOn::Error) {
        "b.source_repo, b.source_path FROM (SELECT * FROM report_broken UNION ALL BY NAME \
         SELECT * FROM report_errors) b WHERE b.outcome IN "
    } else {
        "b.source_repo, b.source_path FROM report_broken b WHERE b.outcome IN "
    })
    .push(&outcomes(fail))
    .push(" AND ")
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
        .push(&outcomes(fail))
        .push(
            ") QUALIFY row_number() OVER (PARTITION BY b.src_url_id, b.target, \
             coalesce(b.fragment, '') ORDER BY b.raw_href, b.kind) = 1) \
             ORDER BY outcome DESC, page, raw_href, fragment",
        )
        .limit(request);
    sql.fetch(conn)
}

/// The redirected links of `run` that the baseline has not redirected, the
/// pedantic ones (D70) last.
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
         b.reason, b.chain, b.source_repo, b.source_path FROM report_redirects b WHERE ",
    )
    .kinds("b.kind", request)
    .push(" AND ");
    sql.scope(run, request, LINK)
        .push(" AND NOT EXISTS (SELECT 1 FROM base o WHERE ")
        .push(SAME_KEY)
        .push(
            " AND o.redirected) QUALIFY row_number() OVER (PARTITION BY b.src_url_id, b.target, \
             coalesce(b.fragment, '') ORDER BY b.raw_href, b.kind) = 1) ORDER BY ",
        )
        .push(&pedantic("reason"))
        .push(", page, raw_href, fragment")
        .limit(request);
    sql.fetch(conn)
}

#[cfg(test)]
mod tests;
