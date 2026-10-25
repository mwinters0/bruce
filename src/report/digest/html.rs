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

use self::markup::{
    Row, a, a_html, close_section, details, duration, esc, marked, num, open_section, rows, short,
    source, table, w, when,
};
use self::severity::{Level, legend, mark};
use super::document::Digest;
use crate::report::table::Table;
use crate::store::{DOUBLED_SLASH, WRONG_TRAILING_SLASH};

mod compared;
mod markup;
mod severity;

const STYLE: &str = include_str!("html/style.css");

impl Digest {
    /// The HTML document.
    pub fn html(&self) -> String {
        let mut out = String::with_capacity(256 * 1024);
        head(&mut out, self);
        out.push_str("<main>\n");
        figures(&mut out, self);
        compared::broken_since(&mut out, self);
        compared::fixed_since(&mut out, self);
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

/// `n` of `noun`, pluralised.
fn counted(n: i64, noun: &str) -> String {
    format!("{} {noun}{}", num(n), if n == 1 { "" } else { "s" })
}

/// A page's or a repo's summary count: its red links, those broken through a
/// link to a URL that doubles a slash (D71) and its amber ones, each behind its
/// dot and a level with none left out, and no total. Red is
/// `broken_url` and `error`, amber `broken_anchor` (see
/// [`severity::Level::of_link`]); the links broken through are red too.
fn tally(red: i64, through: i64, amber: i64) -> String {
    let mut parts = Vec::new();
    for (level, n, words) in [
        (Level::Broken, red, counted(red, "broken link")),
        (
            Level::Broken,
            through,
            format!("{} broken through it", num(through)),
        ),
        (Level::Attention, amber, counted(amber, "missing fragment")),
    ] {
        if n > 0 {
            parts.push(format!("<span class=\"cn\">{} {words}</span>", level.dot()));
        }
    }
    parts.join(", ")
}

/// A page's or a repo's summary count, [`tally`]'s, and how many of its links
/// are new where it is not new itself (D75).
fn summary_count(r: &Row) -> String {
    let mut count = tally(
        r.int("broken_url") + r.int("error"),
        r.int("through"),
        r.int("broken_anchor"),
    );
    let new = r.int("new_links");
    if !r.is_new() && new > 0 {
        w!(count, ", <span class=\"cn\">{} new</span>", num(new));
    }
    count
}

/// `text` escaped, each doubled slash in its path marked: a run of `/` after
/// anything but a `:` or a `/`, ahead of any `?` or `#`, as `doubled_slash`
/// reads one (D71).
fn slashes(text: &str) -> String {
    let path = &text[..text.find(['?', '#']).unwrap_or(text.len())];
    let bytes = path.as_bytes();
    let mut out = String::new();
    let (mut from, mut i) = (0, 1);
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' && !matches!(bytes[i - 1], b':' | b'/') {
            let end = i + bytes[i..].iter().take_while(|&&b| b == b'/').count();
            w!(out, "{}<mark>{}</mark>", esc(&path[from..i]), &path[i..end]);
            (from, i) = (end, end);
        } else {
            i += 1;
        }
    }
    out.push_str(&esc(&text[from..]));
    out
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

/// `text` escaped, the final `/` of its path marked where it has one, as a
/// `wrong_trailing_slash` landing's is.
fn final_slash(text: &str) -> String {
    let at = text.find(['?', '#']).unwrap_or(text.len());
    match text[..at].strip_suffix('/') {
        Some(bare) => format!("{}<mark>/</mark>{}", esc(bare), esc(&text[at..])),
        None => esc(text),
    }
}

/// A broken link's target, at `fragment` where there is one: for a
/// `doubled_slash` one its doubled slash marked, and under it the `landing`
/// that doubles one where the target does not (D71); for a
/// `wrong_trailing_slash` one the `landing` it redirected to with its final
/// `/` marked, and the `fix` under it (D72).
fn target_cell(r: &Row, fragment: Option<&str>) -> String {
    let target = r.text("target").unwrap_or("");
    if r.text("reason") == Some(WRONG_TRAILING_SLASH)
        && let Some(fix) = r.text("fix")
    {
        let mut cell = target_link(target, fragment);
        if let Some(landing) = r.text("landing") {
            w!(
                cell,
                "<br><small>\u{2192} lands on {}</small>",
                a_html(landing, &final_slash(short(landing)))
            );
        }
        w!(
            cell,
            "<br><small>\u{2192} try {}</small>",
            a(fix, short(fix))
        );
        return cell;
    }
    if r.text("reason") != Some(DOUBLED_SLASH) {
        return target_link(target, fragment);
    }
    let mut cell = match fragment {
        Some(fragment) => a_html(
            &format!("{target}#{fragment}"),
            &slashes(&format!("{}#{fragment}", short(target))),
        ),
        None => a_html(target, &slashes(short(target))),
    };
    if let Some(landing) = r.text("landing") {
        w!(
            cell,
            "<br><small>\u{2192} lands on {}</small>",
            a_html(landing, &slashes(short(landing)))
        );
    }
    cell
}

/// A broken link as its page writes it, a doubled slash marked.
fn written_as(r: &Row) -> String {
    let raw = r.text("raw_href").unwrap_or("");
    if r.text("reason") == Some(DOUBLED_SLASH) {
        format!("<code>{}</code>", slashes(raw))
    } else {
        format!("<code>{}</code>", esc(raw))
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
        "<header><div>\n<h1>Link check digest <span class=\"date\">{}</span></h1>\n\
         <p class=\"subtitle\">{}</p>\n<dl class=\"meta\">\n",
        d.date(),
        compared::subtitle(d)
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
    out.push_str("</dl>\n");
    if d.unfinished {
        out.push_str(
            "<p class=\"warn-box\">This run never finished, so the digest is incomplete.</p>\n",
        );
    }
    let (broken, fixed) = compared::shown(d);
    w!(
        out,
        "<nav><a href=\"#figures\">Figures</a> {}{}<a href=\"#pages\">Pages</a> \
         <a href=\"#repos\">Repos</a> <a href=\"#fix-once\">Targets</a> \
         <a href=\"#errors\">Errors</a> <a href=\"#housekeeping\">Housekeeping</a></nav>\n\
         </div></header>\n",
        if broken {
            "<a href=\"#broken-since\">Broken since</a> "
        } else {
            ""
        },
        if fixed {
            "<a href=\"#fixed-since\">Fixed since</a> "
        } else {
            ""
        }
    );
}

fn figures(out: &mut String, d: &Digest) {
    let t = &d.totals;
    out.push_str("<section id=\"figures\">\n<h2>Figures</h2>\n<div class=\"figures\">\n");
    let changes = compared::changes(d);
    for (i, (label, n, class)) in [
        ("Pages checked", t.pages, ""),
        ("Links on them", t.links, ""),
        ("Links to a broken URL", t.broken_url, " bad"),
        ("Links to a missing fragment", t.broken_anchor, " warn"),
        ("Links that errored", t.error, " warn"),
    ]
    .into_iter()
    .enumerate()
    {
        let change = changes.as_ref().map_or("", |c| c[i].as_str());
        w!(
            out,
            "<div class=\"fig{class}\"><span class=\"n\">{}</span><span class=\"l\">{label}</span>{change}</div>\n",
            num(n)
        );
    }
    w!(
        out,
        "</div>\n<p class=\"muted\">Every figure but the first counts links on the pages \
         checked. The {} broken links are those to a broken URL and those to a missing fragment. \
         {} links were not checked; the housekeeping says why. A link that errored could not be \
         asked, which does not make it broken.{}</p>\n{}</section>\n",
        num(t.broken_url + t.broken_anchor),
        num(t.unchecked),
        compared::since(d),
        legend() + &compared::new_legend(d)
    );
}

fn fix_once(out: &mut String, d: &Digest) {
    open_section(out, "fix-once", "By: Link Target", None);
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
        let body: Vec<(bool, Vec<String>)> = many
            .iter()
            .map(|r| {
                let cells = vec![
                    mark(Level::of_link(r.text("outcome"), r.text("reason"))),
                    num(r.int("links")),
                    num(r.int("pages")),
                    target_cell(r, r.text("fragment")),
                    problem(r.text("outcome"), r.text("reason")),
                    status_cell(r.int("status")),
                    source(
                        r.text("source_repo"),
                        r.text("source_path"),
                        r.text("source_url"),
                    ),
                ];
                (r.is_new(), cells)
            })
            .collect();
        marked(
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
    open_section(out, "pages", "By: Source Page", None);
    out.push_str(
        "<i>Written as</i> is the link in the page. <i>Resolves to</i> is the URL it landed at. \
         <i>Kind</i> is the type of element holding it (<code>anchor</code> is an <code>&lt;a \
         href&gt;</code>). <i>Problem</i> and <i>Status</i> are what the target \
         answered.</p>\n\
         <p>A link to a URL with a doubled slash in its path is broken whatever the URL answered, \
         reason <code>doubled_slash</code>: the page linking there is the one to correct. The \
         doubled slash is marked, and where a redirect lands on it, the landing is under \
         <i>Resolves to</i>. Its \
         row also counts the links <i>broken through it</i>, the relative links on the pages \
         such a link reaches, which the doubled slash breaks, and ranks by them; those pages \
         count only their absolute links, or none where a page at the URL without the doubled \
         slash was checked.</p>\n\
         <p>A link to a URL that answered 404 where the same URL with its final \
         <code>/</code> added or dropped answers is broken all the same, reason \
         <code>wrong_trailing_slash</code>, and <i>Resolves to</i> names that URL to link to \
         instead. Where a redirect lands on the 404, the landing is shown with its final \
         <code>/</code> marked, and the URL named is the landing's with it toggled.</p>\n",
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
        let (open, said) = details(p.is_new(), Some(&id));
        w!(
            out,
            "{open}\n<summary>{said}<span class=\"rank\">{}</span><span class=\"ttl\">{}</span>\
             <span class=\"cnt\">{}</span></summary>\n",
            i + 1,
            esc(p.text("title").unwrap_or_else(|| short(url))),
            summary_count(&p)
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
        let mut found: Vec<(Option<Level>, bool, Vec<String>)> = page_links
            .iter()
            .map(|r| {
                let mut cells = vec![
                    mark(Level::of_link(r.text("outcome"), r.text("reason"))),
                    written_as(r),
                    target_cell(r, None),
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
                (
                    Level::of_link(r.text("outcome"), r.text("reason")),
                    r.is_new(),
                    cells,
                )
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
            (Some(Level::Broken), r.is_new(), cells)
        }));
        found.sort_by_key(|(level, _, _)| std::cmp::Reverse(*level));
        let body: Vec<(bool, Vec<String>)> = found
            .into_iter()
            .map(|(_, new, cells)| (new, cells))
            .collect();
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
        marked(out, &columns, &body);
        out.push_str("</details>\n");
    }
    close_section(out);
    ids
}

fn repos(out: &mut String, d: &Digest, ids: &Ids) {
    open_section(out, "repos", "By: Source Page Repo", None);
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
        let (open, said) = details(r.is_new(), None);
        w!(
            out,
            "{open}\n<summary>{said}<span class=\"rank\">{}</span><span class=\"ttl\">{name}</span>\
             <span class=\"cnt\">{}</span></summary>\n<ul>\n",
            i + 1,
            summary_count(&r)
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
        let body: Vec<(bool, Vec<String>)> = rows(&d.errors)
            .map(|r| {
                let page = r.text("page").unwrap_or("");
                let cells = vec![
                    mark(Some(Level::Broken)),
                    a(page, short(page)),
                    r.text("source_url")
                        .map_or_else(|| "-".to_owned(), |s| a(s, "source")),
                    format!("<code>{}</code>", esc(r.text("raw_href").unwrap_or(""))),
                    target_link(r.text("target").unwrap_or(""), r.text("fragment")),
                    esc(r.text("kind").unwrap_or("")),
                    problem(None, r.text("reason")),
                    status_cell(r.int("status")),
                ];
                (r.is_new(), cells)
            })
            .collect();
        marked(
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
    let (open, said) = details(r.is_new(), None);
    w!(
        out,
        "{open}\n<summary>{said}{}<span class=\"ttl\">{}</span><span class=\"cnt\">{}, {}</span></summary>\n",
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
            let body: Vec<(bool, Vec<String>)> = rows(&d.sitemap_entries)
                .map(|r| {
                    let page = r.text("page").unwrap_or("");
                    let cells = vec![
                        a(page, short(page)),
                        problem(r.text("outcome"), r.text("reason")),
                        status_cell(r.int("status")),
                    ];
                    (r.is_new(), cells)
                })
                .collect();
            marked(
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
    let body: Vec<(bool, Vec<String>)> = rows(t)
        .map(|r| {
            let page = r.text("page").unwrap_or("");
            let cells = vec![
                a(page, r.text("title").unwrap_or_else(|| short(page))),
                a(page, short(page)),
                source(r.text("source_repo"), r.text("source_path"), None),
            ];
            (r.is_new(), cells)
        })
        .collect();
    marked(
        out,
        &[("Title", false), ("Page", false), ("Source", false)],
        &body,
    );
}

#[cfg(test)]
mod tests;
