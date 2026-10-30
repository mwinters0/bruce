//! The policy a run judges its links by, and records (D78).
//!
//! A site's judging settings live in config, and a run stores the policy it
//! was judged by, so a report reads the run and never the file. Every setting
//! has a default, today's value (D77); [`Policy::default`] is the policy of a
//! run given no file. The settings are added as their slices land, each as
//! one more [`Policy::entries`] row, so a new setting changes no schema.

/// The judging settings of a run, each at its default until config sets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Whether a redirect that only adds or drops a final `/` is `ok`, reason
    /// `pedantic_trailing_slash`, rather than a redirect to correct (D70).
    pub pedantic_trailing_slash: bool,
}

impl Default for Policy {
    fn default() -> Policy {
        Policy {
            pedantic_trailing_slash: true,
        }
    }
}

impl Policy {
    /// The run-wide settings as the `(key, value)` rows the store records:
    /// every setting, the defaults included, so a run's policy is whole in its
    /// own rows and a later default changes none (D78).
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        vec![(
            "pedantic_trailing_slash",
            self.pedantic_trailing_slash.to_string(),
        )]
    }
}

#[cfg(test)]
mod tests;
