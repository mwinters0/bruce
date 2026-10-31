//! The severity of a row, read off a link's outcome and reason through the
//! run's severity map (D82), and the dot that marks it. HTML only: the JSON's
//! outcome and reason and its policy decide the level, and it gains no field. A dot marks a finding, a row naming a link or a
//! target or a part of a page's or repo's count, and never a heading. A dot is filled for broken and attention and a ring
//! for pedantic, so the three read apart without their colours; each beside a
//! finding is named for a screen reader, the colour repeating the row's own
//! text, and the legend's are hidden, its words beside them.

use std::fmt::Write as _;

use super::super::document::Digest;
use super::guide::Entry;
use super::markup::w;
use crate::config::Severity;

/// The class and name of a level that has a dot.
fn class(level: Severity) -> &'static str {
    match level {
        Severity::None => "",
        Severity::Pedantic => "pedantic",
        Severity::Attention => "attention",
        Severity::Broken => "broken",
    }
}

fn name(level: Severity) -> &'static str {
    match level {
        Severity::None => "",
        Severity::Pedantic => "Pedantic",
        Severity::Attention => "Attention",
        Severity::Broken => "Broken",
    }
}

/// The dot of `level`, a table's first cell or a drill-down summary's first
/// child, named by its level for a screen reader; nothing for none.
pub(super) fn mark(level: Severity) -> String {
    if level == Severity::None {
        return String::new();
    }
    format!(
        "<span class=\"sev {}\" role=\"img\" aria-label=\"{}\"></span>",
        class(level),
        name(level)
    )
}

/// The dot of `level` and a space after it, nothing for none.
pub(super) fn mark_before(level: Severity) -> String {
    let dot = mark(level);
    if dot.is_empty() { dot } else { dot + " " }
}

/// The level of a figure's class: `bad` for broken, `warn` for attention.
pub(super) fn figure_class(level: Severity) -> &'static str {
    match level {
        Severity::Broken => " bad",
        Severity::Attention => " warn",
        _ => "",
    }
}

/// What a kind of finding is called in the legend, `after` being the kinds
/// already named at its level.
fn kind_words(kind: &str, after: &[&str], d: &Digest) -> String {
    match kind {
        "broken_url" => "a link to a broken URL".to_owned(),
        "broken_anchor" => "a link to a missing fragment".to_owned(),
        "error"
            if after
                .iter()
                .any(|k| ["broken_url", "broken_anchor"].contains(k)) =>
        {
            "one that could not be verified".to_owned()
        }
        "error" => "a link that could not be verified".to_owned(),
        "redirected" => "a redirect to update".to_owned(),
        "pedantic_trailing_slash" => format!(
            "a redirect differing from its landing only by a final <code>/</code>{}",
            Entry::Pedantic.info_if(d.judged)
        ),
        reason => format!("a link with the reason <code>{reason}</code>"),
    }
}

/// `items` as prose: "a", "a, or b", "a, b, or c".
fn listed(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [a, b] => format!("{a}, or {b}"),
        [init @ .., last] => format!("{}, or {last}", init.join(", ")),
    }
}

/// The legend, one line naming each level some kind has, with the kinds at
/// it, the dots hidden from a screen reader that reads the words beside them,
/// and a line naming the kinds at none, which the digest does not list, nor
/// count on a page or a repo, its figures being the run's outcome totals (D82).
pub(super) fn legend(d: &Digest) -> String {
    let items: Vec<String> = [Severity::Broken, Severity::Attention, Severity::Pedantic]
        .into_iter()
        .filter_map(|level| {
            let kinds = d.severity.kinds_at(level);
            if kinds.is_empty() {
                return None;
            }
            let words: Vec<String> = kinds
                .iter()
                .enumerate()
                .map(|(i, kind)| kind_words(kind, &kinds[..i], d))
                .collect();
            Some(format!(
                "<span class=\"sev {}\" aria-hidden=\"true\"></span> <b>{}</b>, {}",
                class(level),
                name(level),
                listed(&words)
            ))
        })
        .collect();
    let mut out = String::new();
    if !items.is_empty() {
        w!(
            out,
            "<p class=\"legend\">A dot marks the severity of a link or a target: {}.</p>\n",
            items.join("; ")
        );
    }
    let quiet = d.severity.kinds_at(Severity::None);
    if !quiet.is_empty() {
        let words: Vec<String> = quiet
            .iter()
            .enumerate()
            .map(|(i, kind)| kind_words(kind, &quiet[..i], d))
            .collect();
        w!(
            out,
            "<p class=\"legend\">Not listed, nor counted on a page or a repo, this report being set to treat them as no finding: {}.</p>\n",
            listed(&words)
        );
    }
    out
}
