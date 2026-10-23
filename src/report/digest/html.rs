//! The digest as one HTML document: the same [`Digest`] the JSON is written
//! from, rendered with no script and no outside file, so it stands alone
//! wherever it is copied (D64). Every section's rows arrive ranked, and are
//! written in that order.
//!
//! Dark on every screen and light in print, one stylesheet inline, drill-downs
//! as `<details>` and navigation as in-page anchors. Every text is escaped, and
//! only an `http` or `https` URL becomes a link, so a page's own markup or a
//! `javascript:` href cannot reach the document as anything but text.

use std::collections::HashMap;
use std::fmt::Write as _;

use super::document::{Digest, utc};
use crate::report::table::{Cell, Table};

const STYLE: &str = include_str!("html/style.css");

macro_rules! w {
    ($out:expr, $($arg:tt)*) => {{ let _ = write!($out, $($arg)*); }};
}

impl Digest {
    /// The HTML document.
    pub fn html(&self) -> String {
        let mut out = String::with_capacity(256 * 1024);
        head(&mut out, self);
        out.push_str("<main>\n");
        figures(&mut out, self);
        fix_once(&mut out, self);
        let ids = pages(&mut out, self);
        repos(&mut out, self, &ids);
        errors(&mut out, self);
        housekeeping(&mut out, self);
        out.push_str("</main>\n");
        w!(
            out,
            "<footer>Written by bruce <span class=\"version\">{}</span>, document format {}. The \
             JSON digest of this day holds the same data, and every broken and error row.\
             </footer>\n</body>\n</html>\n",
            esc(env!("CARGO_PKG_VERSION")),
            super::FORMAT_VERSION
        );
        out
    }
}

/// One row of a table, its cells found by column name.
struct Row<'a> {
    columns: &'a [String],
    cells: &'a [Cell],
}

impl<'a> Row<'a> {
    fn cell(&self, name: &str) -> &'a Cell {
        let at = self
            .columns
            .iter()
            .position(|c| c == name)
            .unwrap_or_else(|| panic!("the digest's table has no column `{name}`"));
        &self.cells[at]
    }

    fn text(&self, name: &str) -> Option<&'a str> {
        self.cell(name).text().filter(|s| !s.is_empty())
    }

    fn int(&self, name: &str) -> i64 {
        match self.cell(name) {
            Cell::Int(n) => *n,
            _ => 0,
        }
    }

    fn list(&self, name: &str) -> &'a [String] {
        match self.cell(name) {
            Cell::List(items) => items,
            _ => &[],
        }
    }
}

fn rows(table: &Table) -> impl Iterator<Item = Row<'_>> {
    table.rows.iter().map(|cells| Row {
        columns: &table.columns,
        cells,
    })
}

/// `s` safe in text and in a quoted attribute.
fn esc(s: &str) -> String {
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
fn num(n: i64) -> String {
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

/// `n` of `noun`, pluralised.
fn counted(n: i64, noun: &str) -> String {
    format!("{} {noun}{}", num(n), if n == 1 { "" } else { "s" })
}

/// `url` without its scheme, to show.
fn short(url: &str) -> &str {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
}

/// A link to `url` labelled `label`, or the label alone where `url` is not
/// an `http` or `https` one.
fn a(url: &str, label: &str) -> String {
    if url.starts_with("https://") || url.starts_with("http://") {
        format!("<a href=\"{}\">{}</a>", esc(url), esc(label))
    } else {
        esc(label)
    }
}

/// A link to `target`, at `fragment` where there is one, shown without its
/// scheme.
fn target_link(target: &str, fragment: Option<&str>) -> String {
    match fragment {
        Some(fragment) => a(
            &format!("{target}#{fragment}"),
            &format!("{}#{fragment}", short(target)),
        ),
        None => a(target, short(target)),
    }
}

/// A page's file, as a link to the forge's view of it where the run holds one.
fn source(repo: Option<&str>, path: Option<&str>, url: Option<&str>) -> String {
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
fn when(seconds: i64) -> String {
    let (_, rfc) = utc(seconds);
    let shown = format!("{} UTC", rfc.trim_end_matches('Z').replace('T', " "));
    format!("<time datetime=\"{rfc}\">{shown}</time>")
}

fn duration(seconds: i64) -> String {
    let (h, m, s) = (seconds / 3_600, seconds % 3_600 / 60, seconds % 60);
    match (h, m) {
        (0, 0) => format!("{s} s"),
        (0, _) => format!("{m} min {s} s"),
        _ => format!("{h} h {m} min {s} s"),
    }
}

/// What a link's outcome and reason say, in words.
fn problem(outcome: Option<&str>, reason: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    match outcome {
        Some("broken_anchor") => parts.push("anchor missing".to_owned()),
        Some("broken_url") => parts.push("broken".to_owned()),
        Some(other) => parts.push(other.replace('_', " ")),
        None => {}
    }
    if let Some(reason) = reason {
        parts.push(reason.to_owned());
    }
    esc(&parts.join(" \u{b7} "))
}

fn status_cell(status: i64) -> String {
    if status > 0 {
        status.to_string()
    } else {
        "<span class=\"muted\">-</span>".to_owned()
    }
}

/// A table of already-rendered cells; a header marked `true` is a number. A
/// table with no row is the word for it, not a header over nothing.
fn table(out: &mut String, head: &[(&str, bool)], body: &[Vec<String>]) {
    if body.is_empty() {
        out.push_str("<p class=\"muted\">None.</p>\n");
        return;
    }
    out.push_str("<div class=\"scroll\"><table>\n<thead><tr>");
    for (name, numeric) in head {
        let class = if *numeric { " class=\"num\"" } else { "" };
        w!(out, "<th{class}>{name}</th>");
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for row in body {
        out.push_str("<tr>");
        for (cell, (_, numeric)) in row.iter().zip(head) {
            let class = if *numeric { " class=\"num\"" } else { "" };
            w!(out, "<td{class}>{cell}</td>");
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table></div>\n");
}

fn head(out: &mut String, d: &Digest) {
    let run = &d.run;
    let first = run.roots.first().map_or("", |r| short(r));
    let more = match run.roots.len() {
        0 | 1 => String::new(),
        n => format!(" and {} more", n - 1),
    };
    w!(
        out,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <meta name=\"color-scheme\" content=\"dark light\">\n\
         <title>Link check digest {} \u{2013} {}{}</title>\n<style>\n{STYLE}</style>\n</head>\n<body>\n",
        d.date(),
        esc(first),
        esc(&more)
    );
    w!(
        out,
        "<header><div>\n<h1>Link check digest <span class=\"date\">{}</span></h1>\n<dl class=\"meta\">\n",
        d.date()
    );
    let roots: Vec<String> = run.roots.iter().map(|r| a(r, short(r))).collect();
    w!(out, "<dt>Roots</dt><dd>{}</dd>\n", roots.join(", "));
    let finished = match run.finished {
        Some(at) => format!(
            "finished {}, after <span class=\"duration\">{}</span>",
            when(at),
            duration(at - run.started)
        ),
        None => "never finished".to_owned(),
    };
    w!(
        out,
        "<dt>Run</dt><dd>{}, started {}, {finished}</dd>\n",
        run.id.0,
        when(run.started)
    );
    w!(
        out,
        "<dt>Crawled by</dt><dd>bruce <span class=\"version\">{}</span></dd>\n",
        esc(&run.tool_version)
    );
    out.push_str("<dt>Compared with</dt><dd>no earlier run</dd>\n</dl>\n");
    if d.unfinished {
        out.push_str(
            "<p class=\"warn-box\">This run never finished, so the digest is incomplete.</p>\n",
        );
    }
    out.push_str(
        "<nav><a href=\"#figures\">Figures</a> <a href=\"#fix-once\">Fix once</a> \
         <a href=\"#pages\">Pages</a> <a href=\"#repos\">Repos</a> \
         <a href=\"#errors\">Errors</a> <a href=\"#housekeeping\">Housekeeping</a></nav>\n\
         </div></header>\n",
    );
}

fn figures(out: &mut String, d: &Digest) {
    let t = &d.totals;
    out.push_str("<section id=\"figures\">\n<h2>Figures</h2>\n<div class=\"figures\">\n");
    for (label, n, class) in [
        ("Pages checked", t.pages, ""),
        ("Links checked", t.links, ""),
        ("Broken URLs", t.broken_url, " bad"),
        ("Broken anchors", t.broken_anchor, " bad"),
        ("Errors", t.error, " warn"),
    ] {
        w!(
            out,
            "<div class=\"fig{class}\"><span class=\"n\">{}</span><span class=\"l\">{label}</span></div>\n",
            num(n)
        );
    }
    w!(
        out,
        "</div>\n<p class=\"muted\">{} links were not checked; the housekeeping says why. An \
         error is a link that could not be asked, not necessarily a broken one.</p>\n</section>\n",
        num(t.unchecked)
    );
}

fn fix_once(out: &mut String, d: &Digest) {
    out.push_str(
        "<section id=\"fix-once\">\n<h2>Fix once, fix many</h2>\n<p>The broken targets that more \
         than one link names, the most links first. Restoring one in the repo of its page \
         removes every link in its row.</p>\n",
    );
    let (many, single): (Vec<Row>, Vec<Row>) =
        rows(&d.broken_targets).partition(|r| r.int("links") > 1);
    if many.is_empty() {
        out.push_str("<p>No broken target is named by more than one link.</p>\n");
    } else {
        let held: i64 = many.iter().map(|r| r.int("links")).sum();
        let all = d.totals.broken_url + d.totals.broken_anchor;
        w!(
            out,
            "<p class=\"muted\">Targets named by more than one link: {}, holding {} of the {} broken links.</p>\n",
            num(many.len() as i64),
            num(held),
            num(all)
        );
        let body: Vec<Vec<String>> = many
            .iter()
            .map(|r| {
                vec![
                    num(r.int("links")),
                    num(r.int("pages")),
                    target_link(r.text("target").unwrap_or(""), r.text("fragment")),
                    problem(r.text("outcome"), r.text("reason")),
                    status_cell(r.int("status")),
                    source(
                        r.text("source_repo"),
                        r.text("source_path"),
                        r.text("source_url"),
                    ),
                ]
            })
            .collect();
        table(
            out,
            &[
                ("Links", true),
                ("Pages", true),
                ("Target", false),
                ("Problem", false),
                ("Status", true),
                ("Fix in", false),
            ],
            &body,
        );
    }
    if !single.is_empty() {
        w!(
            out,
            "<p>Targets named by one link each: {}. They are in the pages below.</p>\n",
            num(single.len() as i64)
        );
    }
    out.push_str("</section>\n");
}

/// The id of each page's drill-down, by the page's URL.
type Ids = HashMap<String, String>;

fn pages(out: &mut String, d: &Digest) -> Ids {
    out.push_str(
        "<section id=\"pages\">\n<h2>Pages by broken links</h2>\n<p>The pages holding the most \
         broken links first. Open one for its links.</p>\n",
    );
    let mut links: HashMap<&str, Vec<Row>> = HashMap::new();
    for row in rows(&d.broken) {
        links
            .entry(row.text("page").unwrap_or(""))
            .or_default()
            .push(row);
    }
    let mut ids = Ids::new();
    if d.pages.rows.is_empty() {
        out.push_str("<p>No page holds a broken link.</p>\n");
    }
    for (i, p) in rows(&d.pages).enumerate() {
        let url = p.text("page").unwrap_or("");
        let id = format!("p{}", i + 1);
        ids.insert(url.to_owned(), id.clone());
        let (u, an, er) = (p.int("broken_url"), p.int("broken_anchor"), p.int("error"));
        let mut counts = format!("{}, {}", counted(u, "URL"), counted(an, "fragment"));
        if er > 0 {
            w!(counts, ", {} error", num(er));
        }
        w!(
            out,
            "<details id=\"{id}\">\n<summary><span class=\"rank\">{}</span><span class=\"ttl\">{}</span>\
             <span class=\"cnt\"><b>{}</b> broken <small>({counts}; {} links)</small></span></summary>\n",
            i + 1,
            esc(p.text("title").unwrap_or_else(|| short(url))),
            num(u + an),
            num(p.int("links"))
        );
        w!(
            out,
            "<p>Page {} \u{b7} Source {}</p>\n",
            a(url, short(url)),
            source(
                p.text("source_repo"),
                p.text("source_path"),
                p.text("source_url")
            )
        );
        let body: Vec<Vec<String>> = links
            .get(url)
            .into_iter()
            .flatten()
            .map(|r| {
                vec![
                    format!("<code>{}</code>", esc(r.text("raw_href").unwrap_or(""))),
                    target_link(r.text("target").unwrap_or(""), None),
                    r.text("fragment")
                        .map_or_else(|| "-".to_owned(), |f| format!("<code>{}</code>", esc(f))),
                    esc(r.text("kind").unwrap_or("")),
                    problem(r.text("outcome"), r.text("reason")),
                    status_cell(r.int("status")),
                ]
            })
            .collect();
        table(
            out,
            &[
                ("Link", false),
                ("Target", false),
                ("Fragment", false),
                ("Kind", false),
                ("Problem", false),
                ("Status", true),
            ],
            &body,
        );
        out.push_str("</details>\n");
    }
    out.push_str("</section>\n");
    ids
}

fn repos(out: &mut String, d: &Digest, ids: &Ids) {
    out.push_str(
        "<section id=\"repos\">\n<h2>By repo</h2>\n<p>The repos whose pages hold the most broken \
         links first. Open one for its pages.</p>\n",
    );
    let mut of_repo: HashMap<Option<&str>, Vec<(&str, &str)>> = HashMap::new();
    for p in rows(&d.pages) {
        let url = p.text("page").unwrap_or("");
        of_repo
            .entry(p.text("source_repo"))
            .or_default()
            .push((url, p.text("title").unwrap_or_else(|| short(url))));
    }
    let mut clean = 0;
    for (i, r) in rows(&d.by_repo).enumerate() {
        let repo = r.text("repo");
        if r.int("broken_pages") == 0 {
            clean += 1;
            continue;
        }
        let name = match repo {
            Some(repo) => a(repo, short(repo)),
            None => "<span class=\"muted\">No source recorded</span>".to_owned(),
        };
        let (u, an, er) = (r.int("broken_url"), r.int("broken_anchor"), r.int("error"));
        let mut counts = format!("{}, {}", counted(u, "URL"), counted(an, "fragment"));
        if er > 0 {
            w!(counts, ", {} error", num(er));
        }
        w!(
            out,
            "<details>\n<summary><span class=\"rank\">{}</span><span class=\"ttl\">{name}</span>\
             <span class=\"cnt\"><b>{}</b> broken <small>({counts}; on {} of {} pages)</small></span></summary>\n<ul>\n",
            i + 1,
            num(u + an),
            num(r.int("broken_pages")),
            num(r.int("pages"))
        );
        for (url, title) in of_repo.get(&repo).into_iter().flatten() {
            let label = esc(title);
            match ids.get(*url) {
                Some(id) => w!(out, "<li><a href=\"#{id}\">{label}</a></li>\n"),
                None => w!(out, "<li>{label}</li>\n"),
            }
        }
        out.push_str("</ul>\n</details>\n");
    }
    if d.by_repo.rows.len() == clean {
        out.push_str("<p>No repo holds a broken link.</p>\n");
    } else if clean > 0 {
        w!(
            out,
            "<p class=\"muted\">{} more repos hold no broken link.</p>\n",
            num(clean as i64)
        );
    }
    out.push_str("</section>\n");
}

fn errors(out: &mut String, d: &Digest) {
    out.push_str(
        "<section id=\"errors\">\n<h2>Errors</h2>\n<p>Links that could not be asked, apart from \
         the broken ones: the target did not answer, which does not make it wrong.</p>\n",
    );
    if d.errors.rows.is_empty() {
        out.push_str("<p>No link errored.</p>\n");
    } else {
        let body: Vec<Vec<String>> = rows(&d.errors)
            .map(|r| {
                let page = r.text("page").unwrap_or("");
                vec![
                    a(page, short(page)),
                    r.text("source_url")
                        .map_or_else(|| "-".to_owned(), |s| a(s, "source")),
                    format!("<code>{}</code>", esc(r.text("raw_href").unwrap_or(""))),
                    target_link(r.text("target").unwrap_or(""), r.text("fragment")),
                    esc(r.text("kind").unwrap_or("")),
                    problem(None, r.text("reason")),
                    status_cell(r.int("status")),
                ]
            })
            .collect();
        table(
            out,
            &[
                ("Page", false),
                ("Source", false),
                ("Link", false),
                ("Target", false),
                ("Kind", false),
                ("Reason", false),
                ("Status", true),
            ],
            &body,
        );
    }
    out.push_str("</section>\n");
}

/// A nested `<details>` of `title` holding `inner`, collapsed, its summary
/// carrying `count`, the size of what it holds in the unit the list is of.
fn fold(out: &mut String, title: &str, count: &str, inner: impl FnOnce(&mut String)) {
    w!(
        out,
        "<details>\n<summary><span class=\"ttl\">{title}</span><span class=\"cnt\">{count}</span></summary>\n"
    );
    inner(out);
    out.push_str("</details>\n");
}

fn housekeeping(out: &mut String, d: &Digest) {
    out.push_str(
        "<section id=\"housekeeping\">\n<h2>Housekeeping</h2>\n<details class=\"house\">\n\
         <summary>Redirects to update, sitemap entries, orphans, unlisted pages, unchecked links</summary>\n",
    );
    let links = |t: &Table| format!("{} links", num(rows(t).map(|r| r.int("links")).sum()));
    let entries = |t: &Table| num(t.rows.len() as i64);
    fold(out, "Redirected links", &links(&d.redirects), |out| {
        out.push_str("<p>The links that reach their page through a redirect, by landing URL: update them to the landing.</p>\n");
        let body: Vec<Vec<String>> = rows(&d.redirects)
            .map(|r| {
                let landing = r.text("landing").unwrap_or("");
                let pages: Vec<String> = r.list("pages").iter().map(|p| a(p, short(p))).collect();
                vec![
                    a(landing, short(landing)),
                    num(r.int("links")),
                    num(r.int("urls")),
                    esc(r.text("chain").unwrap_or("")),
                    pages.join("<br>"),
                ]
            })
            .collect();
        table(
            out,
            &[
                ("Landing", false),
                ("Links", true),
                ("URLs", true),
                ("First hops", false),
                ("On pages", false),
            ],
            &body,
        );
    });
    fold(
        out,
        "Sitemap entries no page was fetched for",
        &entries(&d.sitemap_entries),
        |out| {
            out.push_str("<p>Antora writes a sitemap from what it publishes, so an entry that did not answer is a build or deploy fault.</p>\n");
            let body: Vec<Vec<String>> = rows(&d.sitemap_entries)
                .map(|r| {
                    let page = r.text("page").unwrap_or("");
                    vec![
                        a(page, short(page)),
                        problem(r.text("outcome"), r.text("reason")),
                        status_cell(r.int("status")),
                    ]
                })
                .collect();
            table(
                out,
                &[("Entry", false), ("Answer", false), ("Status", true)],
                &body,
            );
        },
    );
    fold(out, "Orphans", &entries(&d.orphans), |out| {
        out.push_str("<p>Pages the sitemap lists that no page links to.</p>\n");
        page_list(out, &d.orphans);
    });
    fold(out, "Unlisted pages", &entries(&d.unlisted), |out| {
        out.push_str("<p>Pages missing from the sitemap of a root that read one.</p>\n");
        if !d.roots_without_sitemap.is_empty() {
            let roots: Vec<String> = d
                .roots_without_sitemap
                .iter()
                .map(|r| a(r, short(r)))
                .collect();
            w!(
                out,
                "<p>Read no sitemap, so no page of them can be unlisted: {}.</p>\n",
                roots.join(", ")
            );
        }
        page_list(out, &d.unlisted);
    });
    fold(out, "Unchecked links", &links(&d.unchecked), |out| {
        out.push_str("<p>Links not checked, by why.</p>\n");
        let body: Vec<Vec<String>> = rows(&d.unchecked)
            .map(|r| {
                vec![
                    esc(r.text("reason").unwrap_or("")),
                    num(r.int("links")),
                    num(r.int("targets")),
                ]
            })
            .collect();
        table(
            out,
            &[("Reason", false), ("Links", true), ("Targets", true)],
            &body,
        );
    });
    out.push_str("</details>\n</section>\n");
}

/// The pages of `orphans` or `unlisted`: `page`, `title` and the source.
fn page_list(out: &mut String, t: &Table) {
    let body: Vec<Vec<String>> = rows(t)
        .map(|r| {
            let page = r.text("page").unwrap_or("");
            vec![
                a(page, r.text("title").unwrap_or_else(|| short(page))),
                a(page, short(page)),
                source(r.text("source_repo"), r.text("source_path"), None),
            ]
        })
        .collect();
    table(
        out,
        &[("Title", false), ("Page", false), ("Source", false)],
        &body,
    );
}

#[cfg(test)]
mod tests;
