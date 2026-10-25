//! The document's markup helpers: escaping, numbers, links, the table, a
//! section's opening and a table's row found by column name. They depend on no
//! part of the document, so the parts that use them (`compared`, `severity`)
//! stand beside `html` and not under a cycle with it.

use std::fmt::Write as _;

use super::super::document::utc;
use crate::report::table::{Cell, Table};

/// Appends formatted text to a `String`, which cannot fail.
macro_rules! w {
    ($out:expr, $($arg:tt)*) => {{ let _ = write!($out, $($arg)*); }};
}
pub(super) use w;

/// One row of a table, its cells found by column name.
pub(super) struct Row<'a> {
    pub(super) columns: &'a [String],
    pub(super) cells: &'a [Cell],
}

impl<'a> Row<'a> {
    pub(super) fn cell(&self, name: &str) -> &'a Cell {
        let at = self
            .columns
            .iter()
            .position(|c| c == name)
            .unwrap_or_else(|| panic!("the digest's table has no column `{name}`"));
        &self.cells[at]
    }

    pub(super) fn text(&self, name: &str) -> Option<&'a str> {
        self.cell(name).text().filter(|s| !s.is_empty())
    }

    pub(super) fn int(&self, name: &str) -> i64 {
        match self.cell(name) {
            Cell::Int(n) => *n,
            _ => 0,
        }
    }

    /// The row is new since the compared run (D75).
    pub(super) fn is_new(&self) -> bool {
        matches!(self.cell("new"), Cell::Bool(true))
    }

    pub(super) fn list(&self, name: &str) -> &'a [String] {
        match self.cell(name) {
            Cell::List(items) => items,
            _ => &[],
        }
    }
}

pub(super) fn rows(table: &Table) -> impl Iterator<Item = Row<'_>> {
    table.rows.iter().map(|cells| Row {
        columns: &table.columns,
        cells,
    })
}
/// `s` safe in text and in a quoted attribute.
pub(super) fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// `n` with a comma between thousands.
pub(super) fn num(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        out.insert(0, '-');
    }
    out
}
/// `url` without its scheme, to show.
pub(super) fn short(url: &str) -> &str {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
}

/// A link to `url` labelled `label`, or the label alone where `url` is not
/// an `http` or `https` one.
pub(super) fn a(url: &str, label: &str) -> String {
    a_html(url, &esc(label))
}

/// [`a`], its label already HTML.
pub(super) fn a_html(url: &str, label: &str) -> String {
    if url.starts_with("https://") || url.starts_with("http://") {
        format!("<a href=\"{}\">{label}</a>", esc(url))
    } else {
        label.to_owned()
    }
}
/// A page's file, as a link to the forge's view of it where the run holds one.
pub(super) fn source(repo: Option<&str>, path: Option<&str>, url: Option<&str>) -> String {
    match (url, path, repo) {
        (Some(url), Some(path), _) => a(url, path),
        (Some(url), None, _) => a(url, short(url)),
        (None, Some(path), Some(repo)) => {
            format!("{} <small>in {}</small>", esc(path), a(repo, short(repo)))
        }
        (None, None, Some(repo)) => a(repo, short(repo)),
        _ => "<span class=\"muted\">-</span>".to_owned(),
    }
}

/// The time as a `<time>`, in UTC.
pub(super) fn when(seconds: i64) -> String {
    let (_, rfc) = utc(seconds);
    let shown = format!("{} UTC", rfc.trim_end_matches('Z').replace('T', " "));
    format!("<time datetime=\"{rfc}\">{shown}</time>")
}

/// The UTC day of the time as a `<time>`.
pub(super) fn day(seconds: i64) -> String {
    let (date, _) = utc(seconds);
    format!("<time datetime=\"{date}\">{date}</time>")
}

pub(super) fn duration(seconds: i64) -> String {
    let (h, m, s) = (seconds / 3_600, seconds % 3_600 / 60, seconds % 60);
    match (h, m) {
        (0, 0) => format!("{s} s"),
        (0, _) => format!("{m} min {s} s"),
        _ => format!("{h} h {m} min {s} s"),
    }
}
/// What a screen reader reads of the bar marking a row or a drill-down new;
/// the bar is the class `new`.
pub(super) const NEW: &str = "<span class=\"vh\">New: </span>";

/// A table of already-rendered cells; a header marked `true` is a number, and
/// one with no name is the severity dot's column. A table with no row is the
/// word for it, not a header over nothing.
pub(super) fn table(out: &mut String, head: &[(&str, bool)], body: &[Vec<String>]) {
    let body: Vec<(bool, &[String])> = body.iter().map(|r| (false, r.as_slice())).collect();
    rows_of(out, head, &body);
}

/// [`table`], each row marked new where its flag says (D75).
pub(super) fn marked(out: &mut String, head: &[(&str, bool)], body: &[(bool, Vec<String>)]) {
    let body: Vec<(bool, &[String])> = body.iter().map(|(n, r)| (*n, r.as_slice())).collect();
    rows_of(out, head, &body);
}

fn rows_of(out: &mut String, head: &[(&str, bool)], body: &[(bool, &[String])]) {
    if body.is_empty() {
        out.push_str("<p class=\"muted\">None.</p>\n");
        return;
    }
    out.push_str("<div class=\"scroll\"><table>\n<thead><tr>");
    for (name, numeric) in head {
        w!(out, "<th{}>{name}</th>", column_class(name, *numeric));
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for (new, row) in body {
        out.push_str(if *new { "<tr class=\"new\">" } else { "<tr>" });
        for (i, (cell, (name, numeric))) in row.iter().zip(head).enumerate() {
            let said = if *new && i == 0 { NEW } else { "" };
            w!(out, "<td{}>{said}{cell}</td>", column_class(name, *numeric));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table></div>\n");
}

/// A `<details>`'s opening tag, marked new where `new` says (D75), and what a
/// screen reader reads of the mark, for the start of its summary.
pub(super) fn details(new: bool, id: Option<&str>) -> (String, &'static str) {
    let id = id.map_or_else(String::new, |id| format!(" id=\"{id}\""));
    if new {
        (format!("<details class=\"new\"{id}>"), NEW)
    } else {
        (format!("<details{id}>"), "")
    }
}

/// The class of a column's cells.
pub(super) fn column_class(name: &str, numeric: bool) -> &'static str {
    match (name.is_empty(), numeric) {
        (true, _) => " class=\"dot\"",
        (_, true) => " class=\"num\"",
        _ => "",
    }
}

/// Opens the section `id` under its `heading`, a `<details>` that starts
/// collapsed, `note` beside the heading. The id is the `<section>`'s, outside the
/// `<details>`, so a link to it lands on the heading in every browser, one that
/// opens no closed `<details>` for a fragment included.
pub(super) fn open_section(out: &mut String, id: &str, heading: &str, note: Option<&str>) {
    w!(
        out,
        "<section id=\"{id}\">\n<details class=\"sect\">\n<summary><h2>{heading}</h2>"
    );
    if let Some(note) = note {
        w!(out, "<small>{note}</small>");
    }
    out.push_str("</summary>\n");
}

pub(super) fn close_section(out: &mut String) {
    out.push_str("</details>\n</section>\n");
}
