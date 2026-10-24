//! The digest as JSON: the document a database loads, whose shape
//! `docs/manual/digest.md` documents field by field and a test holds to it.

use super::document::{Digest, utc};
use crate::report::table::{Format, quoted};

/// The version of the document's shape. It changes only with a change a
/// loader would misread: a field renamed, removed or given another meaning,
/// never one added (D64).
pub const FORMAT_VERSION: u32 = 1;

/// An object of `fields`, one to a line, each value already JSON.
fn object(fields: &[(&str, String)]) -> String {
    let lines: Vec<String> = fields
        .iter()
        .map(|(name, value)| format!("  {}: {value}", quoted(name)))
        .collect();
    format!("{{\n{}\n}}\n", lines.join(",\n"))
}

/// A number, or `null`.
fn number(n: Option<i64>) -> String {
    n.map_or_else(|| "null".to_owned(), |n| n.to_string())
}

fn strings(items: &[String]) -> String {
    let items: Vec<String> = items.iter().map(|s| quoted(s)).collect();
    format!("[{}]", items.join(","))
}

impl Digest {
    /// The JSON document, under [`FORMAT_VERSION`].
    pub fn json(&self) -> String {
        render(self)
    }
}

fn render(d: &Digest) -> String {
    let run = &d.run;
    let finished = run.finished.map(|at| utc(at).1);
    let run_object = format!(
        "{{\"id\":{},\"tool_version\":{},\"started_at\":{},\"finished_at\":{},\
         \"duration_seconds\":{},\"roots\":{}}}",
        run.id.0,
        quoted(&run.tool_version),
        quoted(&utc(run.started).1),
        finished
            .as_deref()
            .map_or_else(|| "null".to_owned(), quoted),
        number(run.finished.map(|at| at - run.started)),
        strings(&run.roots),
    );
    let t = &d.totals;
    let totals = format!(
        "{{\"pages\":{},\"links\":{},\"broken_url\":{},\"broken_anchor\":{},\"error\":{},\
         \"unchecked\":{}}}",
        t.pages, t.links, t.broken_url, t.broken_anchor, t.error, t.unchecked
    );
    let json = |table: &crate::report::Table| table.render(Format::Json);
    let housekeeping = format!(
        "{{\"redirects\":{},\"orphans\":{},\"unlisted\":{},\"sitemap_entries\":{},\
         \"roots_without_sitemap\":{},\"unchecked\":{},\"unchecked_groups\":{},\
         \"checked_hosts\":{}}}",
        json(&d.redirects),
        json(&d.orphans),
        json(&d.unlisted),
        json(&d.sitemap_entries),
        strings(&d.roots_without_sitemap),
        json(&d.unchecked),
        json(&d.unchecked_groups),
        json(&d.checked_hosts),
    );
    object(&[
        ("format_version", FORMAT_VERSION.to_string()),
        ("tool_version", quoted(env!("CARGO_PKG_VERSION"))),
        ("run", run_object),
        ("compared_with", "null".to_owned()),
        ("totals", totals),
        ("broken_targets", json(&d.broken_targets)),
        ("pages", json(&d.pages)),
        ("broken", json(&d.broken)),
        ("by_repo", json(&d.by_repo)),
        ("by_component", json(&d.by_component)),
        ("errors", json(&d.errors)),
        ("housekeeping", housekeeping),
    ])
}
