//! The model as a view reads it: the [`Digest`] and its derived part as one
//! tree of values for a template (D95). Every key is a fact the run recorded
//! or a judgement of the severity map, a level spelled as the store spells it
//! and a reason as the tables hold it, and no key is a sentence, so a view in
//! any language words them. A table's row is a map from its column names to
//! its cells, a missing cell `none`, so a view naming a column the digest
//! lacks fails (strict undefined names) and never prints nothing.

use minijinja::value::Value;

use super::super::cause::CAUSES;
use super::super::document::{Compared, Digest, Run, Totals};
use super::{Changes, Counts, KINDS, LEVELS, Page, Row, differing, listed, rows};
use crate::config::{Reason, Severity};
use crate::report::digest::json::FORMAT_VERSION;
use crate::report::table::{Cell, Table};

/// The tree a view is rendered over.
pub(in crate::report::digest) fn context(d: &Digest) -> Value {
    let ranked = super::pages(d);
    let (pages, ids) = pages_value(&ranked);
    let repos = repos_value(d, &ranked, &ids);
    Value::from_pairs([
        (
            "bruce",
            pairs([
                ("version", env!("CARGO_PKG_VERSION").into()),
                ("model_version", FORMAT_VERSION.into()),
            ]),
        ),
        ("date", d.date().into()),
        ("run", run_value(&d.run)),
        (
            "compared",
            d.compared.as_ref().map_or(none(), compared_value),
        ),
        ("unfinished", d.unfinished.into()),
        ("trial", opt(d.trial.as_deref())),
        ("totals", totals_value(&d.totals)),
        ("figures", figures_value(d)),
        ("legend", legend_value(d)),
        ("judged", judged_value(d)),
        (
            "pedantic",
            level(
                d.severity
                    .of_redirect(Some(Reason::PedanticTrailingSlash.as_str())),
            )
            .into(),
        ),
        ("broken_since", breakage(d, &d.new_breakage, true)),
        ("fixed_since", breakage(d, &d.fixed_breakage, false)),
        ("pages", pages),
        ("pages_any", (!d.pages.rows.is_empty()).into()),
        ("repos", repos),
        ("targets", targets_value(d)),
        ("errors", errors_value(d)),
        ("redirects", redirects_value(d)),
        ("sitemap", sitemap_value(d)),
        ("orphans", table_value(&d.orphans)),
        ("unlisted", table_value(&d.unlisted)),
        ("roots_without_sitemap", strings(&d.roots_without_sitemap)),
        ("unchecked", unchecked_value(d)),
        ("checked_hosts", table_value(&d.checked_hosts)),
        ("settings", settings_value(d)),
        ("setting_changes", setting_changes_value(d)),
    ])
}

fn pairs<const N: usize>(items: [(&'static str, Value); N]) -> Value {
    Value::from_pairs(items)
}

fn none() -> Value {
    Value::from(())
}

fn opt(s: Option<&str>) -> Value {
    s.map_or(none(), Value::from)
}

fn strings<S: AsRef<str>>(items: &[S]) -> Value {
    items.iter().map(|s| Value::from(s.as_ref())).collect()
}

/// A level as the store spells it.
fn level(level: Severity) -> &'static str {
    level.as_str()
}

fn run_value(run: &Run) -> Value {
    pairs([
        ("id", run.id.0.into()),
        ("tool_version", run.tool_version.as_str().into()),
        ("started", run.started.into()),
        ("finished", run.finished.map_or(none(), Value::from)),
        ("roots", strings(&run.roots)),
    ])
}

fn compared_value(c: &Compared) -> Value {
    pairs([
        ("run", run_value(&c.run)),
        ("totals", totals_value(&c.totals)),
        ("file", c.file.into()),
        ("shares_day", c.shares_day.into()),
    ])
}

fn totals_value(t: &Totals) -> Value {
    pairs([
        ("pages", t.pages.into()),
        ("links", t.links.into()),
        ("broken_url", t.broken_url.into()),
        ("broken_anchor", t.broken_anchor.into()),
        ("error", t.error.into()),
        ("unchecked", t.unchecked.into()),
    ])
}

fn judged_value(d: &Digest) -> Value {
    pairs([
        ("doubled_slash", d.judged.doubled_slash.into()),
        (
            "pedantic_trailing_slash",
            d.judged.pedantic_trailing_slash.into(),
        ),
        ("reach", d.judged.reach.as_str().into()),
    ])
}

/// The five figures in the order the digest lists them: the figure, its change
/// since the compared run (none where there was no comparison), the level of
/// the outcome it counts and whether a rise in it is for the worse.
fn figures_value(d: &Digest) -> Value {
    let t = &d.totals;
    let change = super::figure_changes(d);
    let of = |outcome: Option<&str>| level(d.severity.of_link(outcome, None));
    [
        ("pages", t.pages, "none", false),
        ("links", t.links, "none", false),
        ("broken_url", t.broken_url, of(Some("broken_url")), true),
        (
            "broken_anchor",
            t.broken_anchor,
            of(Some("broken_anchor")),
            true,
        ),
        ("error", t.error, of(Some("error")), true),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (key, n, level, broken))| {
        pairs([
            ("key", key.into()),
            ("n", n.into()),
            ("change", change.map_or(none(), |c| c[i].into())),
            ("level", level.into()),
            ("broken", broken.into()),
        ])
    })
    .collect()
}

/// The kinds at each level that has some, the worst first, and those at none,
/// which the digest lists nowhere (D82).
fn legend_value(d: &Digest) -> Value {
    let levels: Value = [Severity::Broken, Severity::Attention, Severity::Pedantic]
        .into_iter()
        .filter_map(|l| {
            let kinds = d.severity.kinds_at(l);
            (!kinds.is_empty())
                .then(|| pairs([("level", level(l).into()), ("kinds", strings(&kinds))]))
        })
        .collect();
    pairs([
        ("levels", levels),
        ("quiet", strings(&d.severity.kinds_at(Severity::None))),
    ])
}

fn cell_value(cell: &Cell) -> Value {
    match cell {
        Cell::Null => none(),
        Cell::Text(s) if s.is_empty() => none(),
        Cell::Text(s) => s.as_str().into(),
        Cell::Int(n) => (*n).into(),
        Cell::Bool(b) => (*b).into(),
        Cell::List(items) => strings(items),
    }
}

/// A row as a map from its column names to its cells, with `extra` keys beside
/// them: `is_new` where the table marks rows new (D75).
fn row_value(r: &Row, extra: Vec<(&'static str, Value)>) -> Value {
    let mut out: Vec<(&str, Value)> = r
        .columns
        .iter()
        .zip(r.cells)
        .map(|(name, cell)| (name.as_str(), cell_value(cell)))
        .collect();
    if r.columns.iter().any(|c| c == "new" || c == "new_listed") {
        out.push(("is_new", r.is_new().into()));
    }
    out.extend(extra);
    Value::from_pairs(out)
}

fn table_value(t: &Table) -> Value {
    rows(t).map(|r| row_value(&r, Vec::new())).collect()
}

/// A link of `t` as a row with its level, or none where the map makes it no
/// finding (D92).
fn finding(d: &Digest, r: &Row) -> Option<Value> {
    let l = d.severity.of_link(r.text("outcome"), r.text("reason"));
    l.is_finding()
        .then(|| row_value(r, vec![("level", level(l).into())]))
}

/// Section 3 or 4: none where the table is empty, else the findings of the
/// table and, for the links broken since, the causes with some, in the order a
/// maintainer reads them (D74).
fn breakage(d: &Digest, t: &Table, with_causes: bool) -> Value {
    if t.rows.is_empty() {
        return none();
    }
    let causes: Value = if with_causes {
        CAUSES
            .iter()
            .filter_map(|cause| {
                let of: Vec<Value> = rows(t)
                    .filter(|r| r.text("cause") == Some(cause))
                    .filter_map(|r| finding(d, &r))
                    .collect();
                (!of.is_empty())
                    .then(|| pairs([("cause", (*cause).into()), ("rows", Value::from(of))]))
            })
            .collect()
    } else {
        Value::from(Vec::<Value>::new())
    };
    let all: Vec<Value> = rows(t).filter_map(|r| finding(d, &r)).collect();
    pairs([("rows", Value::from(all)), ("causes", causes)])
}

/// A count's parts: the links of each kind at each level, red first, with the
/// reasons that put them off their outcome's level.
pub(in crate::report::digest) fn counts_value(counts: &Counts) -> Value {
    let mut parts = Vec::new();
    for l in LEVELS.into_iter().rev() {
        for kind in KINDS {
            let n = counts.at(kind, l);
            if n == 0 {
                continue;
            }
            parts.push(pairs([
                ("level", level(l).into()),
                ("kind", kind.name().into()),
                ("n", n.into()),
                ("reasons", strings(&counts.reasons(kind, l))),
            ]));
        }
    }
    Value::from(parts)
}

/// The ranked pages with a finding, and the id of each one's drill-down by its
/// URL.
fn pages_value(ranked: &[Page]) -> (Value, std::collections::HashMap<String, String>) {
    let mut ids = std::collections::HashMap::new();
    let out: Vec<Value> = ranked
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let id = format!("p{}", i + 1);
            ids.insert(p.row.text("page").unwrap_or("").to_owned(), id.clone());
            let findings: Vec<Value> = p
                .findings
                .iter()
                .map(|f| {
                    pairs([
                        ("row", row_value(&f.row, Vec::new())),
                        ("level", level(f.level).into()),
                        ("is_new", f.is_new.into()),
                        ("error", f.error.into()),
                    ])
                })
                .collect();
            pairs([
                ("row", row_value(&p.row, Vec::new())),
                ("id", id.into()),
                ("rank", (i as i64 + 1).into()),
                ("counts", counts_value(&p.counts)),
                ("new_links", p.new_links.into()),
                ("hinted", p.hinted.into()),
                ("findings", Value::from(findings)),
            ])
        })
        .collect();
    (Value::from(out), ids)
}

/// The repos holding a finding, ranked, each with its pages that do; the count
/// of those holding none, and whether there is no repo holding one.
fn repos_value(
    d: &Digest,
    ranked: &[Page],
    ids: &std::collections::HashMap<String, String>,
) -> Value {
    let mut clean = 0;
    let by_repo = super::repos(d, ranked);
    let mut listed = Vec::new();
    for (i, repo) in by_repo.iter().enumerate() {
        if repo.pages == 0 {
            clean += 1;
            continue;
        }
        let name = repo.row.text("repo");
        let pages: Vec<Value> = ranked
            .iter()
            .filter(|p| p.row.text("source_repo") == name)
            .map(|p| {
                let url = p.row.text("page").unwrap_or("");
                pairs([
                    ("id", ids.get(url).map_or(none(), |id| id.as_str().into())),
                    (
                        "title",
                        p.row.text("title").unwrap_or_else(|| short(url)).into(),
                    ),
                ])
            })
            .collect();
        listed.push(pairs([
            ("row", row_value(&repo.row, Vec::new())),
            ("rank", (i as i64 + 1).into()),
            ("counts", counts_value(&repo.counts)),
            ("new_links", repo.new_links.into()),
            ("pages", Value::from(pages)),
        ]));
    }
    pairs([
        ("listed", Value::from(listed)),
        ("clean", clean.into()),
        ("all_clean", (by_repo.len() == clean).into()),
    ])
}

/// `url` without its scheme.
fn short(url: &str) -> &str {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
}

fn targets_value(d: &Digest) -> Value {
    let t = super::targets(d);
    let many: Vec<Value> = t
        .many
        .iter()
        .map(|r| {
            let l = d.severity.of_link(r.text("outcome"), r.text("reason"));
            row_value(r, vec![("level", level(l).into())])
        })
        .collect();
    pairs([
        ("many", Value::from(many)),
        ("single", (t.single as i64).into()),
        ("hidden_many", t.hidden_many.into()),
        ("held", t.held.into()),
        ("all", t.all.into()),
    ])
}

/// The links that could not be verified: whether there are any, and those the
/// map makes a finding.
fn errors_value(d: &Digest) -> Value {
    let found: Vec<Value> = rows(&d.errors)
        .filter(|r| listed(&d.severity, Some("error"), r.text("reason")))
        .map(|r| {
            let l = d.severity.of_link(Some("error"), r.text("reason"));
            row_value(&r, vec![("level", level(l).into())])
        })
        .collect();
    pairs([
        ("any", (!d.errors.rows.is_empty()).into()),
        ("rows", Value::from(found)),
    ])
}

/// The redirected links by landing: whether there are any, and those the map
/// makes a finding (D92).
fn redirects_value(d: &Digest) -> Value {
    let found: Vec<Value> = rows(&d.redirects)
        .filter_map(|r| {
            let l = d.severity.of_redirect(r.text("reason"));
            l.is_finding()
                .then(|| row_value(&r, vec![("level", level(l).into())]))
        })
        .collect();
    pairs([
        ("any", (!d.redirects.rows.is_empty()).into()),
        ("rows", Value::from(found)),
    ])
}

/// The sitemap's entries no page was fetched for, and what the crawled pages'
/// generators say of their sitemaps, each a key and a text (D87).
fn sitemap_value(d: &Digest) -> Value {
    let notes: Vec<Value> = d
        .sitemap_notes
        .iter()
        .map(|n| pairs([("key", n.key.into()), ("text", n.text.into())]))
        .collect();
    pairs([
        ("notes", Value::from(notes)),
        ("entries", table_value(&d.sitemap_entries)),
    ])
}

/// The `unchecked` links by reason, each with the groups its links point at.
fn unchecked_value(d: &Digest) -> Value {
    rows(&d.unchecked)
        .map(|r| {
            let reason = r.text("reason");
            let groups: Vec<Value> = rows(&d.unchecked_groups)
                .filter(|g| g.text("reason") == reason)
                .map(|g| row_value(&g, Vec::new()))
                .collect();
            row_value(&r, vec![("groups", Value::from(groups))])
        })
        .collect()
}

/// The settings off their defaults as a file writes them (D91).
fn settings_value(d: &Digest) -> Value {
    differing(d)
        .iter()
        .map(|e| Value::from(e.written_as()))
        .collect()
}

/// The rows that differ between the runs (D91), split by whether they judge:
/// none where there was no comparison.
fn setting_changes_value(d: &Digest) -> Value {
    if d.compared.is_none() {
        return none();
    }
    let side = |c: &&crate::report::digest::changes::Change| {
        let written =
            |e: &Option<crate::config::Entry>| e.as_ref().map_or(none(), |e| e.written_as().into());
        (c.from.is_some() || c.to.is_some())
            .then(|| pairs([("from", written(&c.from)), ("to", written(&c.to))]))
    };
    let Changes { judging, rest } = Changes::of(d);
    pairs([
        ("count", (d.policy_changes.len() as i64).into()),
        ("rest", rest.iter().filter_map(side).collect()),
        ("judging", judging.iter().filter_map(side).collect()),
    ])
}
