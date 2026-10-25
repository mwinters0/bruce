//! What the comparison with an earlier run adds to the document: the line in
//! the header, each figure's change (D73), and the sections of the breakage
//! new and fixed since (D74).

use std::fmt::Write as _;

use super::super::cause::CAUSES;
use super::super::document::{Compared, Digest, Totals};
use super::markup::{
    Row, a, close_section, day, esc, num, open_section, rows, short, source, table, w, when,
};
use super::severity::{Level, mark};
use crate::report::table::Table;

/// The header's sub-title: the day the compared run started, its time too
/// where another run started that day, and which run it is.
pub(super) fn subtitle(d: &Digest) -> String {
    let Some(c) = &d.compared else {
        return "No earlier run to compare with".to_owned();
    };
    let run = &c.run;
    let started = if c.shares_day {
        when(run.started)
    } else {
        day(run.started)
    };
    let which = if c.file {
        "baseline file".to_owned()
    } else {
        format!("run #{}", run.id.0)
    };
    let unfinished = if run.finished.is_none() {
        ", never finished"
    } else {
        ""
    };
    format!("Compared with {started} ({which}{unfinished})")
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

/// The legend's line for the bar marking what is new (D75), none where there
/// was no comparison.
pub(super) fn new_legend(d: &Digest) -> String {
    let Some(c) = &d.compared else {
        return String::new();
    };
    format!(
        "<p class=\"legend\"><span class=\"newbar\" aria-hidden=\"true\"></span> A bar at the \
         left edge marks a row or a drill-down that is new since {}. A drill-down that is not \
         new counts the new links inside it.</p>\n",
        then(c)
    )
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
             does not now.",
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

/// The links of a table of sections 3 and 4, one row each, `now` heading a column
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

/// Whether sections 3 and 4 are shown: each is left out where it would list
/// nothing, as with no comparison, which the sub-title says.
pub(super) fn shown(d: &Digest) -> (bool, bool) {
    (
        !d.new_breakage.rows.is_empty(),
        !d.fixed_breakage.rows.is_empty(),
    )
}

/// Section 3: the links broken since the compared run, each under its cause.
pub(super) fn broken_since(out: &mut String, d: &Digest) {
    let (Some(c), (true, _)) = (&d.compared, shown(d)) else {
        return;
    };
    open_section(
        out,
        "broken-since",
        "Broken since the previous run",
        Some("Each new break under its cause"),
    );
    let new = d.new_breakage.rows.len();
    w!(
        out,
        "<p>{} to a broken URL or a missing fragment {} broken since {}. A new break is laid \
         at the door of what the compared run held, never of a page having changed.</p>\n",
        num(new as i64),
        if new == 1 { "is" } else { "are" },
        then(c)
    );
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
    close_section(out);
}

/// Section 4: the links the compared run had broken that are not broken now,
/// each with what it became.
pub(super) fn fixed_since(out: &mut String, d: &Digest) {
    let (Some(c), (_, true)) = (&d.compared, shown(d)) else {
        return;
    };
    open_section(out, "fixed-since", "Fixed since the previous run", None);
    let fixed = d.fixed_breakage.rows.len();
    w!(
        out,
        "<p>{} broken in {} {} not broken in this one: mended, or gone with the page. The \
         problem is what each was then. The source shown is the linking page's.</p>\n",
        num(fixed as i64),
        then(c),
        if fixed == 1 { "is" } else { "are" }
    );
    let all: Vec<Row> = rows(&d.fixed_breakage).collect();
    links(out, &all, true, "Linking page's source");
    close_section(out);
}
