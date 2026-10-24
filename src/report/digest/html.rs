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

use self::severity::{Level, legend, mark};
use super::document::{Digest, utc};
use crate::report::table::{Cell, Table};

mod severity;

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
        let ids = pages(&mut out, self);
        repos(&mut out, self, &ids);
        fix_once(&mut out, self);
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

/// A page's or a repo's summary count: its red links and its amber ones, each
/// behind its dot and a level with none left out, then the issues in all. Red
/// is `broken_url` and `error`, amber `broken_anchor` (see [`severity::Level::of_link`]).
fn tally(red: i64, amber: i64) -> String {
    let mut parts = Vec::new();
    for (level, n, noun) in [
        (Level::Broken, red, "broken link"),
        (Level::Attention, amber, "missing fragment"),
    ] {
        if n > 0 {
            parts.push(format!(
                "<span class=\"cn\">{} {}</span>",
                level.dot(),
                counted(n, noun)
            ));
        }
    }
    format!(
        "{} <small>({} total)</small>",
        parts.join(", "),
        counted(red + amber, "issue")
    )
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

/// What a link's outcome and reason say, in words: a missing fragment is a
/// fragment, `anchor` being the Kind column's `<a href>` as `--kind` names it.
fn problem(outcome: Option<&str>, reason: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    match outcome {
        Some("broken_anchor") => parts.push("fragment missing".to_owned()),
        Some("broken_url") => parts.push("URL broken".to_owned()),
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

/// A table of already-rendered cells; a header marked `true` is a number, and
/// one with no name is the severity dot's column. A table with no row is the
/// word for it, not a header over nothing.
fn table(out: &mut String, head: &[(&str, bool)], body: &[Vec<String>]) {
    if body.is_empty() {
        out.push_str("<p class=\"muted\">None.</p>\n");
        return;
    }
    out.push_str("<div class=\"scroll\"><table>\n<thead><tr>");
    for (name, numeric) in head {
        w!(out, "<th{}>{name}</th>", column_class(name, *numeric));
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for row in body {
        out.push_str("<tr>");
        for (cell, (name, numeric)) in row.iter().zip(head) {
            w!(out, "<td{}>{cell}</td>", column_class(name, *numeric));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table></div>\n");
}

/// The class of a column's cells.
fn column_class(name: &str, numeric: bool) -> &'static str {
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
fn open_section(out: &mut String, id: &str, heading: &str, note: Option<&str>) {
    w!(
        out,
        "<section id=\"{id}\">\n<details class=\"sect\">\n<summary><h2>{heading}</h2>"
    );
    if let Some(note) = note {
        w!(out, "<small>{note}</small>");
    }
    out.push_str("</summary>\n");
}

fn close_section(out: &mut String) {
    out.push_str("</details>\n</section>\n");
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
        "<nav><a href=\"#figures\">Figures</a> <a href=\"#pages\">Pages</a> \
         <a href=\"#repos\">Repos</a> <a href=\"#fix-once\">Targets</a> \
         <a href=\"#errors\">Errors</a> <a href=\"#housekeeping\">Housekeeping</a></nav>\n\
         </div></header>\n",
    );
}

fn figures(out: &mut String, d: &Digest) {
    let t = &d.totals;
    out.push_str("<section id=\"figures\">\n<h2>Figures</h2>\n<div class=\"figures\">\n");
    for (label, n, class) in [
        ("Pages checked", t.pages, ""),
        ("Links on them", t.links, ""),
        ("Links to a broken URL", t.broken_url, " bad"),
        ("Links to a missing fragment", t.broken_anchor, " bad"),
        ("Links that errored", t.error, " warn"),
    ] {
        w!(
            out,
            "<div class=\"fig{class}\"><span class=\"n\">{}</span><span class=\"l\">{label}</span></div>\n",
            num(n)
        );
    }
    w!(
        out,
        "</div>\n<p class=\"muted\">Every figure but the first counts links on the pages \
         checked. The {} broken links are those to a broken URL and those to a missing fragment. \
         {} links were not checked; the housekeeping says why. A link that errored could not be \
         asked, which does not make it broken.</p>\n{}</section>\n",
        num(t.broken_url + t.broken_anchor),
        num(t.unchecked),
        legend()
    );
}

fn fix_once(out: &mut String, d: &Digest) {
    open_section(
        out,
        "fix-once",
        "Broken targets, by the links to them",
        None,
    );
    out.push_str(
        "<p>Fix once, fix many: each row is a target that more than one broken link names, \
         from whatever pages, the most links first. Restoring the target in its own source, the \
         last column, mends every link in its row.</p>\n",
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
                    mark(Level::of_link(r.text("outcome"), r.text("reason"))),
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
                ("", false),
                ("Links to it", true),
                ("Linking pages", true),
                ("Broken target", false),
                ("Problem", false),
                ("Status", true),
                ("Target's source", false),
            ],
            &body,
        );
    }
    if !single.is_empty() {
        w!(
            out,
            "<p>Targets named by one link each: {}. Each is in the drill-down of the page its \
             link is on, among the pages above.</p>\n",
            num(single.len() as i64)
        );
    }
    close_section(out);
}

/// The id of each page's drill-down, by the page's URL.
type Ids = HashMap<String, String>;

fn pages(out: &mut String, d: &Digest) -> Ids {
    open_section(out, "pages", "Pages, by the broken links on them", None);
    out.push_str(
        "<p>The pages holding the most broken links first, those to a broken URL or that errored \
         (red) before those to a missing fragment (amber), a link counted on the page it is \
         written on. Open one for its links: <i>Written as</i> is the link in the page, <i>Resolves to</i> the \
         URL it names, <i>Kind</i> the element holding it, <code>anchor</code> being an \
         <code>&lt;a href&gt;</code>, and <i>Problem</i> and <i>Status</i> what the target \
         answered.</p>\n",
    );
    let mut links: HashMap<&str, Vec<Row>> = HashMap::new();
    for row in rows(&d.broken) {
        links
            .entry(row.text("page").unwrap_or(""))
            .or_default()
            .push(row);
    }
    let mut errored: HashMap<&str, Vec<Row>> = HashMap::new();
    for row in rows(&d.errors) {
        errored
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
        w!(
            out,
            "<details id=\"{id}\">\n<summary><span class=\"rank\">{}</span><span class=\"ttl\">{}</span>\
             <span class=\"cnt\">{}</span></summary>\n",
            i + 1,
            esc(p.text("title").unwrap_or_else(|| short(url))),
            tally(p.int("broken_url") + p.int("error"), p.int("broken_anchor"))
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
        let page_links = links.get(url).map_or(&[][..], Vec::as_slice);
        let page_errors = errored.get(url).map_or(&[][..], Vec::as_slice);
        let hinted = page_links.iter().any(|r| r.text("hint").is_some());
        // The red links together, a link that errored after the broken URLs,
        // then the amber ones.
        let mut found: Vec<(Option<Level>, Vec<String>)> = page_links
            .iter()
            .map(|r| {
                let mut cells = vec![
                    mark(Level::of_link(r.text("outcome"), r.text("reason"))),
                    format!("<code>{}</code>", esc(r.text("raw_href").unwrap_or(""))),
                    target_link(r.text("target").unwrap_or(""), None),
                    r.text("fragment")
                        .map_or_else(|| "-".to_owned(), |f| format!("<code>{}</code>", esc(f))),
                    esc(r.text("kind").unwrap_or("")),
                    problem(r.text("outcome"), r.text("reason")),
                ];
                if hinted {
                    cells.push(r.text("hint").map_or_else(
                        || "-".to_owned(),
                        |hint| format!("<code>{}</code>", esc(hint)),
                    ));
                }
                cells.push(status_cell(r.int("status")));
                (Level::of_link(r.text("outcome"), r.text("reason")), cells)
            })
            .collect();
        found.extend(page_errors.iter().map(|r| {
            let mut cells = vec![
                mark(Some(Level::Broken)),
                format!("<code>{}</code>", esc(r.text("raw_href").unwrap_or(""))),
                target_link(r.text("target").unwrap_or(""), None),
                r.text("fragment")
                    .map_or_else(|| "-".to_owned(), |f| format!("<code>{}</code>", esc(f))),
                esc(r.text("kind").unwrap_or("")),
                problem(Some("error"), r.text("reason")),
            ];
            if hinted {
                cells.push("-".to_owned());
            }
            cells.push(status_cell(r.int("status")));
            (Some(Level::Broken), cells)
        }));
        found.sort_by_key(|(level, _)| std::cmp::Reverse(*level));
        let body: Vec<Vec<String>> = found.into_iter().map(|(_, cells)| cells).collect();
        let mut columns = vec![
            ("", false),
            ("Written as", false),
            ("Resolves to", false),
            ("Fragment", false),
            ("Kind", false),
            ("Problem", false),
        ];
        if hinted {
            columns.push(("Hint", false));
        }
        columns.push(("Status", true));
        table(out, &columns, &body);
        out.push_str("</details>\n");
    }
    close_section(out);
    ids
}

fn repos(out: &mut String, d: &Digest, ids: &Ids) {
    open_section(
        out,
        "repos",
        "Repos, by the broken links on their pages",
        None,
    );
    out.push_str(
        "<p>The repos whose pages hold the most broken links first, ranked as the pages are, a \
         link counted against the repo of the page it is written on, which may not be the repo of \
         its target. Open one for its pages.</p>\n",
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
        w!(
            out,
            "<details>\n<summary><span class=\"rank\">{}</span><span class=\"ttl\">{name}</span>\
             <span class=\"cnt\">{}</span></summary>\n<ul>\n",
            i + 1,
            tally(r.int("broken_url") + r.int("error"), r.int("broken_anchor"))
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
        out.push_str("<p>No repo's pages hold a broken link.</p>\n");
    } else if clean > 0 {
        w!(
            out,
            "<p class=\"muted\">{} more repos' pages hold no broken link.</p>\n",
            num(clean as i64)
        );
    }
    close_section(out);
}

fn errors(out: &mut String, d: &Digest) {
    open_section(out, "errors", "Links that errored", None);
    out.push_str(
        "<p>Links whose target could not be asked, apart from the broken ones: the target did \
         not answer, which does not make it wrong.</p>\n",
    );
    if d.errors.rows.is_empty() {
        out.push_str("<p>No link errored.</p>\n");
    } else {
        let body: Vec<Vec<String>> = rows(&d.errors)
            .map(|r| {
                let page = r.text("page").unwrap_or("");
                vec![
                    mark(Some(Level::Broken)),
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
                ("", false),
                ("On page", false),
                ("Page's source", false),
                ("Written as", false),
                ("Resolves to", false),
                ("Kind", false),
                ("Reason", false),
                ("Status", true),
            ],
            &body,
        );
    }
    close_section(out);
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

/// One landing of the redirected links as a `<details>` of its own: its summary
/// the dot, the landing and its counts, inside the first hops and the pages the
/// links are on, a list so that no cell grows as wide as the longest.
fn redirect(out: &mut String, r: &Row) {
    let landing = r.text("landing").unwrap_or("");
    w!(
        out,
        "<details>\n<summary>{}<span class=\"ttl\">{}</span><span class=\"cnt\">{}, {}</span></summary>\n",
        mark(Some(Level::of_redirect(r.text("reason")))),
        esc(short(landing)),
        counted(r.int("links"), "link"),
        counted(r.int("urls"), "redirecting URL")
    );
    w!(out, "<p>Landing {}", a(landing, short(landing)));
    if let Some(reason) = r.text("reason") {
        w!(out, " \u{b7} Reason <code>{}</code>", esc(reason));
    }
    w!(
        out,
        "</p>\n<p>First hops <code>{}</code></p>\n",
        esc(r.text("chain").unwrap_or(""))
    );
    let pages = r.list("pages");
    w!(
        out,
        "<p>Linking pages: {}</p>\n<ul>\n",
        num(pages.len() as i64)
    );
    for page in pages {
        w!(out, "<li>{}</li>\n", a(page, short(page)));
    }
    out.push_str("</ul>\n</details>\n");
}

fn housekeeping(out: &mut String, d: &Digest) {
    open_section(
        out,
        "housekeeping",
        "Housekeeping",
        Some("Redirects to update, sitemap entries, orphans, unlisted pages, unchecked links"),
    );
    let links = |t: &Table| format!("{} links", num(rows(t).map(|r| r.int("links")).sum()));
    let entries = |t: &Table| num(t.rows.len() as i64);
    fold(out, "Redirected links", &links(&d.redirects), |out| {
        out.push_str("<p>The links whose target redirects, by where it lands, the most links first: update them to the landing. Open one for the redirecting URLs, the hops the first took and the pages the links are written on. A landing marked as a ring is reached by a URL differing from it only by a final <code>/</code>, and comes last.</p>\n");
        if d.redirects.rows.is_empty() {
            out.push_str("<p class=\"muted\">None.</p>\n");
        }
        for r in rows(&d.redirects) {
            redirect(out, &r);
        }
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
        unchecked(out, d)
    });
    close_section(out);
}

/// The groups of a reason, or the checked hosts, listed in full up to this
/// many; the rest are counted.
const MOST: usize = 5;
const MOST_HOSTS: usize = 10;

/// What an `unchecked` link's reason means, in a reader's words, and the
/// setting that would check its links where there is one; the manual says it
/// precisely. `None` for a reason this `bruce` does not write.
fn unchecked_meaning(reason: &str) -> Option<&'static str> {
    Some(match reason {
        "out_of_scope" => {
            "Links to the site's pages in other languages, which the language menu links every \
             page to. To check them, add a <code>[[root]]</code> for those languages."
        }
        "host_not_allowlisted" => {
            "Links to other websites, which this run was not set up to check. To check a site's \
             links, add it to the <code>allowlist</code>; the sites that were checked are listed \
             at the end."
        }
        "skipped" => {
            "Links that match a <code>skip</code> pattern in the configuration, left alone on \
             purpose. These are usually pages a check cannot judge, such as a forge's \"new \
             issue\" form or a login page."
        }
        "scheme" => {
            "Links that do not lead to a web page but to something else a browser can open, \
             mostly email addresses (<code>mailto:</code>), so there is nothing to fetch. A link \
             starting with anything else, such as <code>file:</code> or a mistyped \
             <code>hhttps:</code>, cannot work for a reader and is listed as broken instead."
        }
        "robots" => {
            "Links to pages the site asks robots not to fetch, in its <code>robots.txt</code>, \
             and the run respected that. On a site this run checks, the <code>robots</code> \
             setting for its host can turn this off."
        }
        "robots_unavailable" => {
            "Links to a site whose <code>robots.txt</code>, which says what robots may fetch, \
             could not be read: the site answered with an error, or sent the robot somewhere it \
             could not follow, so the run left the whole site alone. Usually the site was failing \
             at the time."
        }
        "externals_skipped" => {
            "Links to other websites that a <code>check-dir</code> run was told not to check \
             (<code>--externals skip</code>), with no earlier result to reuse."
        }
        "too_deep" => "Sitemaps nested too many levels inside other sitemaps, so not read.",
        "no_anchors" => {
            "Links to a spot (after <code>#</code>) in a file the run did not read as a page, \
             such as an image or a file too large to read. The file was found; the spot was not \
             checked."
        }
        "text_fragment" => {
            "Links to some words on a page (<code>#:~:text=</code>), which the browser finds \
             when it opens it. The page was found; whether the words are on it was not checked."
        }
        "cut" => "Links the run stopped before getting to.",
        _ => return None,
    })
}

/// The first `most` of `rows`, ranked, as a table of their `column`, links
/// and targets, and a count of the rest, of their links and of the most any
/// of them holds.
fn most(out: &mut String, label: &str, column: &str, rows: &[Row], most: usize) {
    let body: Vec<Vec<String>> = rows
        .iter()
        .take(most)
        .map(|r| {
            vec![
                r.text(column)
                    .map_or_else(|| "<span class=\"muted\">-</span>".to_owned(), esc),
                num(r.int("links")),
                num(r.int("targets")),
            ]
        })
        .collect();
    table(
        out,
        &[(label, false), ("Links", true), ("Targets they name", true)],
        &body,
    );
    if rows.len() > most {
        let rest = &rows[most..];
        let each = match rest {
            [first, _, ..] => format!(", none more than {}", num(first.int("links"))),
            _ => String::new(),
        };
        w!(
            out,
            "<p class=\"muted\">And {} more, holding {}{each}.</p>\n",
            num(rest.len() as i64),
            counted(rest.iter().map(|r| r.int("links")).sum(), "link")
        );
    }
}

/// The unchecked links: each reason, what it means and where its links
/// point most, then the hosts beyond the roots whose links were checked.
fn unchecked(out: &mut String, d: &Digest) {
    out.push_str(
        "<p>Links this run did not check, grouped by why. They are not counted as broken, as \
         the run does not know whether they work; a link it knows cannot work is listed as \
         broken instead. A run reads the pages under the roots named at the top, checks the \
         links to the rest of their websites but their other languages, and checks links to \
         other websites only when its <code>allowlist</code> names them.</p>\n",
    );
    let mut groups: HashMap<&str, Vec<Row>> = HashMap::new();
    for row in rows(&d.unchecked_groups) {
        groups
            .entry(row.text("reason").unwrap_or(""))
            .or_default()
            .push(row);
    }
    if d.unchecked.rows.is_empty() {
        out.push_str("<p class=\"muted\">None.</p>\n");
    }
    for r in rows(&d.unchecked) {
        let reason = r.text("reason").unwrap_or("");
        w!(
            out,
            "<h3><code>{}</code> <small>{}, to {}</small></h3>\n<p>{}</p>\n",
            esc(reason),
            counted(r.int("links"), "link"),
            counted(r.int("targets"), "target"),
            unchecked_meaning(reason).unwrap_or(
                "<span class=\"muted\">This version of bruce has no description for this \
                 reason.</span>"
            )
        );
        let of_reason = groups.get(reason).map_or(&[][..], Vec::as_slice);
        most(out, "Where they point", "group", of_reason, MOST);
    }
    out.push_str(
        "<h3>External hosts checked</h3>\n<p>The other websites whose links this run did \
         check: those on its <code>allowlist</code> that these pages link to.</p>\n",
    );
    let hosts: Vec<Row> = rows(&d.checked_hosts).collect();
    most(out, "Host", "host", &hosts, MOST_HOSTS);
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
