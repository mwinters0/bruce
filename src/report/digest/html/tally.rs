//! A page's or a repo's summary count in words (D82), over the counts
//! [`model`](super::super::model) derives: its links by level, red first, each
//! kind of a level behind the level's dot and a kind of none left out, and no
//! total. Under D76's map: the broken links, those broken through a link to a
//! URL that doubles a slash (D88), the unverifiable ones, then the missing
//! fragments. A part the map moved off its outcome's level names the reasons
//! that did, so no two parts differ by their dots alone; the links broken
//! through a doubled slash name theirs already.

use super::guide::Entry;
use super::markup::{counted, num};
use super::severity::mark_before;
use crate::config::Severity;
use crate::report::digest::model::{Counts, KINDS, Kind, LEVELS};

/// The reasons that moved a part, as code: "(`a`, `b`)", empty where none did.
fn reasons(counts: &Counts, kind: Kind, level: Severity) -> String {
    let reasons: Vec<String> = counts
        .reasons(kind, level)
        .iter()
        .map(|r| format!("<code>{r}</code>"))
        .collect();
    if reasons.is_empty() {
        String::new()
    } else {
        format!(" ({})", reasons.join(", "))
    }
}

pub(super) fn tally(counts: &Counts) -> String {
    let mut parts = Vec::new();
    for level in LEVELS.into_iter().rev() {
        for kind in KINDS {
            let n = counts.at(kind, level);
            if n == 0 {
                continue;
            }
            let words = match kind {
                Kind::Broken => counted(n, "broken link") + &reasons(counts, kind, level),
                Kind::Through => {
                    format!("{} broken through it{}", num(n), Entry::DoubledSlash.info())
                }
                Kind::Unverifiable => {
                    format!("{} unverifiable{}", num(n), reasons(counts, kind, level))
                }
                Kind::Fragment => counted(n, "missing fragment") + &reasons(counts, kind, level),
            };
            parts.push(format!(
                "<span class=\"cn\">{}{words}</span>",
                mark_before(level)
            ));
        }
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests;
