//! What a link came to: D16's outcome and the reason beside it.
//!
//! Both are closed sets of what the store holds, so each is an enum here, in
//! the layer the severity map validates against (D39), and every writer and
//! reader of a reason names a variant. [`Reason::takes_level`] has no
//! wildcard arm: a reason added to the enum is a decision about `[severity]`
//! the compiler asks for.

/// One of D16's five outcomes, as `targets.outcome` stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    BrokenUrl,
    BrokenAnchor,
    Error,
    Unchecked,
}

impl Outcome {
    /// The value the tables hold.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::BrokenUrl => "broken_url",
            Outcome::BrokenAnchor => "broken_anchor",
            Outcome::Error => "error",
            Outcome::Unchecked => "unchecked",
        }
    }

    /// The outcomes `[severity]` may set a level for: `ok` is no finding and
    /// `unchecked` keeps none (D76).
    pub const LEVELLED: [Outcome; 3] = [Outcome::BrokenUrl, Outcome::BrokenAnchor, Outcome::Error];
}

/// The reasons, each with the value the tables hold. One list makes the enum,
/// [`Reason::ALL`] and [`Reason::as_str`], so none is left out of either.
macro_rules! reasons {
    ($($(#[$doc:meta])* $variant:ident => $value:literal,)*) => {
        /// A reason a link or target carries, as `targets.reason` and
        /// `link_verdicts` spell it: the cause that tells two kinds of one
        /// outcome apart.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Reason {
            $($(#[$doc])* $variant,)*
        }

        impl Reason {
            /// Every reason.
            pub const ALL: &'static [Reason] = &[$(Reason::$variant,)*];

            /// The value the tables hold.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Reason::$variant => $value,)*
                }
            }
        }
    };
}

reasons! {
        /// A name that does not exist, a `broken_url`.
        Nxdomain => "nxdomain",
        Timeout => "timeout",
        Tls => "tls",
        Connect => "connect",
        Reset => "reset",
        Other => "other",
        /// A page or sitemap answered and could not be read as one, an `error`.
        Unreadable => "unreadable",
        RedirectLoop => "redirect_loop",
        TooManyRedirects => "too_many_redirects",
        /// A 3xx that led nowhere (D16).
        BadRedirect => "bad_redirect",
        /// A page over the body cap, `ok` (D16).
        TooLarge => "too_large",
        /// An Antora `xref:` naming no page.
        UnresolvedXref => "unresolved_xref",
        /// A URL in a component-version the build holds whole and lacks (D22).
        NotBuilt => "not_built",
        /// A scheme no reader of a public web page can follow (D66).
        BadScheme => "bad_scheme",
        /// An href that resolves to no URL (D26).
        InvalidUrl => "invalid_url",
        /// A link to a URL whose path doubles a slash (D71).
        DoubledSlash => "doubled_slash",
        /// A broken 4xx whose URL with its final `/` toggled answers 2xx (D72).
        WrongTrailingSlash => "wrong_trailing_slash",
        /// A redirect differing from its landing only by a final `/` (D70).
        PedanticTrailingSlash => "pedantic_trailing_slash",
        /// A URL on a root's host outside every scope, under another locale's
        /// prefix (D10).
        OutOfScope => "out_of_scope",
        /// A URL on no root's host that the allowlist does not name (D10).
        HostNotAllowlisted => "host_not_allowlisted",
        /// A URL `skip` matched (D50).
        Skipped => "skipped",
        /// A URL `--externals=skip` neither copies nor asks (D54).
        ExternalsSkipped => "externals_skipped",
        /// A scheme a public web page uses but is not `http(s)` (D66).
        Scheme => "scheme",
        /// robots.txt forbids the URL.
        Robots => "robots",
        /// robots.txt could not be read, so the host is left alone.
        RobotsUnavailable => "robots_unavailable",
        /// A sitemap listed by an index three files deep.
        TooDeep => "too_deep",
        /// A target the run stopped before asking about.
        Cut => "cut",
        /// A text fragment (`#:~:text=`), which names words, not an anchor.
        TextFragment => "text_fragment",
        /// A fragment on a target answered with no page read for its anchors.
        NoAnchors => "no_anchors",
}

impl Reason {
    /// The reason a stored value names, none for one `bruce` does not write.
    pub fn parse(s: &str) -> Option<Reason> {
        Reason::ALL.iter().copied().find(|r| r.as_str() == s)
    }

    /// Whether a stored `reason` is this one.
    pub fn is(self, stored: Option<&str>) -> bool {
        stored == Some(self.as_str())
    }

    /// Whether `[severity]` may set a level for this reason: those of a link
    /// that is broken, errored or a redirect to correct. The reasons of an
    /// `unchecked` link take none (D76), and `no_anchors` and `text_fragment`
    /// are theirs.
    pub fn takes_level(self) -> bool {
        match self {
            Reason::Nxdomain
            | Reason::Timeout
            | Reason::Tls
            | Reason::Connect
            | Reason::Reset
            | Reason::Other
            | Reason::Unreadable
            | Reason::RedirectLoop
            | Reason::TooManyRedirects
            | Reason::BadRedirect
            | Reason::TooLarge
            | Reason::UnresolvedXref
            | Reason::NotBuilt
            | Reason::BadScheme
            | Reason::InvalidUrl
            | Reason::DoubledSlash
            | Reason::WrongTrailingSlash
            | Reason::PedanticTrailingSlash => true,
            Reason::OutOfScope
            | Reason::HostNotAllowlisted
            | Reason::Skipped
            | Reason::ExternalsSkipped
            | Reason::Scheme
            | Reason::Robots
            | Reason::RobotsUnavailable
            | Reason::TooDeep
            | Reason::Cut
            | Reason::TextFragment
            | Reason::NoAnchors => false,
        }
    }

    /// The reasons `[severity]` accepts, in the enum's order.
    pub fn levelled() -> impl Iterator<Item = Reason> {
        Reason::ALL.iter().copied().filter(|r| r.takes_level())
    }
}

/// `sql` with each `$reason(<name>)` replaced by the quoted value of that
/// reason, so a statement names a reason as the code does, by a variant's
/// name in snake case. An unknown name panics, being a defect in the text.
pub fn expand(sql: &str) -> String {
    const OPEN: &str = "$reason(";
    let mut out = String::with_capacity(sql.len());
    let mut rest = sql;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let after = &rest[at + OPEN.len()..];
        let end = after
            .find(')')
            .unwrap_or_else(|| panic!("unclosed {OPEN} in a statement"));
        let name = &after[..end];
        let reason = Reason::parse(name).unwrap_or_else(|| panic!("{name:?} is no reason"));
        out.push('\'');
        out.push_str(reason.as_str());
        out.push('\'');
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests;
