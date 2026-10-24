//! What the comparison with an earlier run adds to the document: the line in
//! the header, each figure's change and the section of anchors removed since
//! (D73).

use std::fmt::Write as _;

use super::super::document::{Compared, Digest, Totals};
use super::markup::{
    Row, a, close_section, duration, esc, num, open_section, rows, short, source, table, w, when,
};
use super::severity::{Level, mark};

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
