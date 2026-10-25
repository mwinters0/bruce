//! What the comparison with an earlier run adds to the document: the line in
//! the header, each figure's change and the section of anchors removed since
//! (D73).

use std::fmt::Write as _;

use super::super::cause::CAUSES;
use super::super::document::{Compared, Digest, Totals};
use super::markup::{
    Row, a, close_section, duration, esc, num, open_section, rows, short, source, table, w, when,
};
use super::severity::{Level, mark};
use crate::report::table::Table;

/// The header's "Compared with" line.
pub(super) fn line(d: &Digest) -> String {
    let Some(c) = &d.compared else {
        return "no earlier run".to_owned();
    };
    let run = &c.run;
    let which = if c.file {
        "the run of a baseline file".to_owned()
    } else {
        format!("run {}", run.id.0)
    };
    let finished = match run.finished {
        Some(at) => format!(
            "finished {}, after <span class=\"duration\">{}</span>",
            when(at),
            duration(at - run.started)
        ),
        None => "never finished, so its figures are incomplete".to_owned(),
    };
    format!("{which}, started {}, {finished}", when(run.started))
}

/// A figure's change since the compared run, as the sign and the amount, and
/// a class saying whether it is for the better: a rise in what is broken is
/// not. The class repeats what the text says, never carrying it alone.
fn change(now: i64, then: i64, broken: bool) -> String {
    let by = now - then;
    let (text, class) = match by {
        0 => ("no change".to_owned(), ""),
        n if n > 0 => (format!("+{}", num(n)), if broken { " worse" } else { "" }),
        n => (
            format!("\u{2212}{}", num(-n)),
            if broken { " better" } else { "" },
        ),
    };
    format!("<span class=\"d{class}\">{text}</span>")
}

/// The change of each figure, in the order [`super::figures`] lists them, or
/// none where there was no comparison.
pub(super) fn changes(d: &Digest) -> Option<[String; 5]> {
    let Compared { totals: then, .. } = d.compared.as_ref()?;
    let now: &Totals = &d.totals;
    Some([
        change(now.pages, then.pages, false),
        change(now.links, then.links, false),
        change(now.broken_url, then.broken_url, true),
        change(now.broken_anchor, then.broken_anchor, true),
        change(now.error, then.error, true),
    ])
}

/// What the figures' change is since, for the paragraph beside them.
pub(super) fn since(d: &Digest) -> String {
    match &d.compared {
        None => String::new(),
        Some(c) if c.file => {
            " Each figure's change is since the run of the baseline file.".to_owned()
        }
        Some(c) => format!(" Each figure's change is since run {}.", c.run.id.0),
    }
}

/// Section 7: the pages that lost an anchor links still name.
pub(super) fn anchors_removed(out: &mut String, d: &Digest) {
    open_section(
        out,
        "anchors-removed",
        "Anchors removed since the compared run",
        None,
    );
    let Some(c) = &d.compared else {
        out.push_str(
            "<p>There was no earlier run to compare with, so no anchor is known to have been \
             removed.</p>\n",
        );
        close_section(out);
        return;
    };
    let then = if c.file {
        "the baseline file's run".to_owned()
    } else {
        format!("run {}", c.run.id.0)
    };
    w!(
        out,
        "<p>The pages that had an anchor in {then} and lost it, while links still name it: the \
         links to a missing fragment that the page's own change broke, to restore in the \
         page's source. The most linked first; the linking pages are in the JSON.</p>\n"
    );
    if d.anchors_removed.rows.is_empty() {
        out.push_str("<p>No anchor was removed that a link names.</p>\n");
    } else {
        let body: Vec<Vec<String>> = rows(&d.anchors_removed).map(row).collect();
        table(
            out,
            &[
                ("", false),
                ("Links to it", true),
                ("Linking pages", true),
                ("Page that lost the anchor", false),
                ("Page's source", false),
            ],
            &body,
        );
    }
    close_section(out);
}

fn row(r: Row) -> Vec<String> {
    let page = r.text("page").unwrap_or("");
    let anchor = r.text("anchor").unwrap_or("");
    vec![
        mark(Level::of_link(Some("broken_anchor"), None)),
        num(r.int("links")),
        num(r.int("pages")),
        format!("{} <code>#{}</code>", a(page, short(page)), esc(anchor)),
        source(r.text("source_repo"), r.text("source_path"), None),
    ]
}

/// What the compared run is called in a sentence.
fn then(c: &Compared) -> String {
    if c.file {
        "the baseline file's run".to_owned()
    } else {
        format!("run {}", c.run.id.0)
    }
}

/// A cause's heading, what it says of the link, and the heading of the
/// source column, which says whose source it is.
fn cause_words(cause: &str) -> (&'static str, &'static str, &'static str) {
    match cause {
        "target_removed" => (
            "Target removed",
            "The link was fine in the compared run and its target answers with an error now: \
             the page, or the page it redirects to, is gone.",
            "Target's source, as the compared run recorded it",
        ),
        "target_moved" => (
            "Target moved",
            "The link was fine in the compared run and its target now redirects somewhere it \
             did not, where the link breaks.",
            "Target's source, as the compared run recorded it",
        ),
        "anchor_removed" => (
            "Anchor removed",
            "The link was fine in the compared run, its target page had the fragment then, and \
             does not now. They are the links behind the anchors-removed section's rows.",
            "Target's source, as the compared run recorded it",
        ),
        "link_added" => (
            "Link added",
            "The page was in the compared run and did not hold this link.",
            "Linking page's source",
        ),
        "new_page" => (
            "New page",
            "The compared run did not check the page the link is on.",
            "Linking page's source",
        ),
        "unexplained" => (
            "No cause shown",
            "The same link was in the compared run, and nothing the comparison held explains \
             the difference: usually a rule that judges the link differently since, such as a \
             doubled slash.",
            "Linking page's source",
        ),
        _ => (
            "First judged now",
            "The compared run could not judge the link (it errored, or was left unchecked), \
             so this is no regression.",
            "Linking page's source",
        ),
    }
}

/// A target with its fragment, linked.
fn target(r: &Row) -> String {
    let target = r.text("target").unwrap_or("");
    match r.text("fragment") {
        Some(f) => a(&format!("{target}#{f}"), &format!("{}#{f}", short(target))),
        None => a(target, short(target)),
    }
}

/// What a link answers, in words.
fn problem(r: &Row) -> String {
    let what = match r.text("outcome") {
        Some("broken_anchor") => "fragment missing",
        _ => "URL broken",
    };
    match r.text("reason") {
        Some(reason) => format!("{what} \u{b7} {}", esc(reason)),
        None => what.to_owned(),
    }
}

/// The status of a target, or a dash where it never answered.
fn status(r: &Row) -> String {
    match r.int("status") {
        0 => "<span class=\"muted\">-</span>".to_owned(),
        n => n.to_string(),
    }
}

/// The links of a table of section 3, one row each, `now` heading a column
/// that says what a fixed link became, and `whose` the source shown.
fn links(out: &mut String, rows: &[Row], now: bool, whose: &str) {
    let body: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            let page = r.text("page").unwrap_or("");
            let mut cells = vec![
                mark(Level::of_link(r.text("outcome"), r.text("reason"))),
                a(page, short(page)),
                format!("<code>{}</code>", esc(r.text("raw_href").unwrap_or(""))),
                target(r),
                problem(r),
                status(r),
            ];
            if now {
                cells.push(became(r));
            }
            cells.push(source(
                r.text("source_repo"),
                r.text("source_path"),
                r.text("source_url"),
            ));
            cells
        })
        .collect();
    let mut head = vec![
        ("", false),
        ("On page", false),
        ("Written as", false),
        ("Resolves to", false),
        ("Problem", false),
        ("Status", true),
    ];
    if now {
        head.push(("Now", false));
    }
    head.push((whose, false));
    table(out, &head, &body);
}

/// A closed `<details>` of `title`, its summary counting `n` links.
fn fold(out: &mut String, title: &str, n: usize, inner: impl FnOnce(&mut String)) {
    w!(
        out,
        "<details>\n<summary><span class=\"ttl\">{title}</span><span class=\"cnt\">{} link{}\
         </span></summary>\n",
        num(n as i64),
        if n == 1 { "" } else { "s" }
    );
    inner(out);
    out.push_str("</details>\n");
}

/// The rows of `table`, those whose `column` is `value`.
fn of<'a>(table: &'a Table, column: &str, value: &str) -> Vec<Row<'a>> {
    rows(table)
        .filter(|r| r.text(column) == Some(value))
        .collect()
}

/// What a link became, in words.
fn became(r: &Row) -> String {
    match r.text("became") {
        Some("ok") => "Works".to_owned(),
        Some("error") => "Errors".to_owned(),
        Some("unchecked") => "Unchecked".to_owned(),
        Some("link_removed") => "Link removed".to_owned(),
        Some("page_removed") => "Page not checked".to_owned(),
        _ => "Not counted".to_owned(),
    }
}

/// Section 3: the breakage new and fixed since the compared run, each new link
/// under its cause.
pub(super) fn since_last_run(out: &mut String, d: &Digest) {
    open_section(
        out,
        "since",
        "Broken since the compared run",
        Some("New and fixed breakage, each new link with its cause"),
    );
    let Some(c) = &d.compared else {
        out.push_str(
            "<p>There was no earlier run to compare with, so no break is known to be new.</p>\n",
        );
        close_section(out);
        return;
    };
    let (new, fixed) = (d.new_breakage.rows.len(), d.fixed_breakage.rows.len());
    w!(
        out,
        "<p>{} to a broken URL or a missing fragment {} broken since {}, and {} {} fixed. A \
         new break is laid at the door of what the compared run held, never of a page having \
         changed.</p>\n",
        num(new as i64),
        if new == 1 { "is" } else { "are" },
        then(c),
        num(fixed as i64),
        if fixed == 1 { "was" } else { "were" }
    );
    out.push_str("<h3>New</h3>\n");
    if new == 0 {
        out.push_str("<p class=\"muted\">None.</p>\n");
    }
    for cause in CAUSES {
        let found = of(&d.new_breakage, "cause", cause);
        if found.is_empty() {
            continue;
        }
        let (title, meaning, whose) = cause_words(cause);
        fold(out, title, found.len(), |out| {
            w!(out, "<p>{meaning}</p>\n");
            links(out, &found, false, whose);
        });
    }
    out.push_str("<h3>Fixed</h3>\n");
    if fixed == 0 {
        out.push_str("<p class=\"muted\">None.</p>\n");
    } else {
        let all: Vec<Row> = rows(&d.fixed_breakage).collect();
        fold(out, "Broken then, not now", all.len(), |out| {
            out.push_str(
                "<p>The links the compared run had broken that are not broken in this one: \
                 mended, or gone with the page. The problem is what each was then. The source \
                 shown is the linking page's.</p>\n",
            );
            links(out, &all, true, "Linking page's source");
        });
    }
    close_section(out);
}
