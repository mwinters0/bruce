//! The severity of a row, read off a link's outcome and reason, and the dot
//! that marks it. HTML only: the JSON's outcome and reason decide the level,
//! and it gains no field. A dot is filled for broken and attention and a ring
//! for pedantic, so the three read apart without their colours; each is named
//! for a screen reader, the colour repeating the row's own text.

use crate::store::PEDANTIC_TRAILING_SLASH;

/// How much a link matters to whoever fixes the site, the least first so that
/// the highest of several is their `max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Level {
    /// A redirect differing from its landing only by a final `/`.
    Pedantic,
    /// An `error`, or a redirect that is not pedantic.
    Attention,
    /// `broken_url` and `broken_anchor`.
    Broken,
}

impl Level {
    /// The level a link's outcome and reason give it, `None` for an `unchecked`
    /// one and for an `ok` link that is not a pedantic redirect.
    pub(super) fn of_link(outcome: Option<&str>, reason: Option<&str>) -> Option<Level> {
        match (outcome, reason) {
            (Some("broken_url" | "broken_anchor"), _) => Some(Level::Broken),
            (Some("error"), _) => Some(Level::Attention),
            (Some("ok"), Some(PEDANTIC_TRAILING_SLASH)) => Some(Level::Pedantic),
            _ => None,
        }
    }

    /// The level of a redirected link, whose row has a reason and no outcome.
    pub(super) fn of_redirect(reason: Option<&str>) -> Level {
        if reason == Some(PEDANTIC_TRAILING_SLASH) {
            Level::Pedantic
        } else {
            Level::Attention
        }
    }

    fn class(self) -> &'static str {
        match self {
            Level::Pedantic => "pedantic",
            Level::Attention => "attention",
            Level::Broken => "broken",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Level::Pedantic => "Pedantic",
            Level::Attention => "Attention",
            Level::Broken => "Broken",
        }
    }

    /// The dot, named by its level for a screen reader.
    pub(super) fn dot(self) -> String {
        format!(
            "<span class=\"sev {}\" role=\"img\" aria-label=\"{}\"></span>",
            self.class(),
            self.name()
        )
    }
}

/// The dot of `level`, a table's first cell or the start of a section's
/// summary, nothing for none.
pub(super) fn mark(level: Option<Level>) -> String {
    level.map(Level::dot).unwrap_or_default()
}

/// The highest of `levels`.
pub(super) fn highest(levels: impl IntoIterator<Item = Option<Level>>) -> Option<Level> {
    levels.into_iter().flatten().max()
}

/// The legend, one line naming the three, its dots hidden from a screen reader
/// that reads the words beside them.
pub(super) fn legend() -> String {
    let item = |level: Level, what: &str| {
        format!(
            "<span class=\"sev {}\" aria-hidden=\"true\"></span> <b>{}</b>, {what}",
            level.class(),
            level.name()
        )
    };
    format!(
        "<p class=\"legend\">A dot marks the severity of a link or a target: {}; {}; {}.</p>\n",
        item(
            Level::Broken,
            "a link to a broken URL or a missing fragment"
        ),
        item(
            Level::Attention,
            "a link that errored, or a redirect to update"
        ),
        item(
            Level::Pedantic,
            "a redirect differing from its landing only by a final <code>/</code>"
        ),
    )
}
