//! Profiles: the trait a site generator's extension implements, and the
//! profiles compiled in (D32–D35, D38). Only `cli` names a concrete one (D39),
//! to build the [`Registry`] everyone else reads through.
//!
//! A profile never reaches the database: `store` asks it for its [`View`]s and
//! hands it a [`Lookup`] when it resolves (D39).

pub mod antora;
pub mod generic;

use std::borrow::Cow;
use std::cell::RefCell;

use lol_html::{ElementContentHandlers, Selector};
use url::Url;

use crate::config::ProfileChoice;

/// An error a profile or the store behind its [`Lookup`] hands back.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// What a profile contributes to the core's single rewriting pass: element
/// handlers keyed by selector, in registration order (D38).
pub type ElementHandlers<'h> = Vec<(Cow<'static, Selector>, ElementContentHandlers<'h>)>;

/// A profile's extension of the core (D38). Detect and Views are built;
/// Extract and Resolve are declared here and given bodies by the extractor
/// (P2) and the Antora resolver (P5).
pub trait Profile: Send + Sync {
    /// The name a root's `profile` key gives, and the namespace of the
    /// profile's facts and views (D37).
    fn name(&self) -> &'static str;

    /// Detect: whether the profile claims a page whose
    /// `<meta name="generator">` content is `generator` (D35). The fallback
    /// profile claims none; it is what no other claims.
    fn detect(&self, generator: &str) -> bool;

    /// Extract: the handlers the core adds to its one `HtmlRewriter`, which
    /// record this profile's facts into `facts`, draw the content region
    /// (D34), may override the edit link (D36) and tag links the core cannot
    /// read (I6). Handlers on one element run in registration order, and
    /// none sees an ancestor, so context is a flag an earlier handler set
    /// (D38).
    fn extract<'h>(&self, facts: &'h RefCell<Facts>) -> ElementHandlers<'h>;

    /// Resolve: the URL a reference tagged by [`Profile::extract`] names, as
    /// found on the page at `from`, through the pages `lookup` knows. `None`
    /// for a reference this profile cannot resolve; `generic` has no
    /// reference that is not a URL (D38).
    fn resolve(
        &self,
        reference: &str,
        from: &Url,
        lookup: &dyn Lookup,
    ) -> Result<Option<Url>, BoxError>;

    /// Views: the SQL views pivoting this profile's `page_facts` keys into
    /// columns (D37), created when a database is opened.
    fn views(&self) -> &'static [View];
}

/// One SQL view a profile ships. `name` begins with the profile's name and an
/// underscore, so two profiles' views cannot collide.
#[derive(Debug, Clone, Copy)]
pub struct View {
    /// The view's name in the database.
    pub name: &'static str,
    /// The `SELECT` the view is defined as.
    pub select: &'static str,
}

/// The facts one profile records about one page (D37), each key kept under
/// the profile's name as `<profile>.<key>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    namespace: &'static str,
    pairs: Vec<(String, String)>,
}

impl Facts {
    /// An empty set for the profile named `namespace`.
    pub fn new(namespace: &'static str) -> Facts {
        Facts {
            namespace,
            pairs: Vec::new(),
        }
    }

    /// Records `key` (without the namespace) as `value`.
    pub fn record(&mut self, key: &str, value: impl Into<String>) {
        self.pairs
            .push((format!("{}.{key}", self.namespace), value.into()));
    }

    /// Every fact as a full key and its value, in the order recorded.
    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }
}

/// What the store lets a profile ask of the run it resolves in (D39).
pub trait Lookup {
    /// The URLs of the run's pages carrying every `(full key, value)` pair
    /// given (D37), in no particular order.
    fn pages_with_facts(&self, facts: &[(&str, &str)]) -> Result<Vec<Url>, BoxError>;
}

/// The profiles compiled in, built by `cli` (D39). One is the fallback.
pub struct Registry {
    profiles: Vec<Box<dyn Profile>>,
}

impl Registry {
    /// A registry whose only profile is `fallback`, the one `auto` picks when
    /// no other claims a page (D35).
    pub fn new(fallback: Box<dyn Profile>) -> Registry {
        Registry {
            profiles: vec![fallback],
        }
    }

    /// Adds `profile`. Panics on a name already registered, which is a bug in
    /// whoever builds the registry, not an input.
    pub fn with(mut self, profile: Box<dyn Profile>) -> Registry {
        assert!(
            self.get(profile.name()).is_none(),
            "profile {} registered twice",
            profile.name()
        );
        self.profiles.push(profile);
        self
    }

    /// The profile called `name`.
    pub fn get(&self, name: &str) -> Option<&dyn Profile> {
        self.profiles
            .iter()
            .find(|p| p.name() == name)
            .map(|p| &**p)
    }

    /// Every profile, the fallback first.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Profile> {
        self.profiles.iter().map(|p| &**p)
    }

    /// The profile for a page: the root's own if it names one, else the first
    /// that claims the page's generator meta, else the fallback (D35).
    /// `Err` carries a name the registry does not hold.
    pub fn choose(
        &self,
        choice: &ProfileChoice,
        generator: Option<&str>,
    ) -> Result<&dyn Profile, UnknownProfile> {
        match choice {
            ProfileChoice::Named(name) => {
                self.get(name).ok_or_else(|| UnknownProfile(name.clone()))
            }
            ProfileChoice::Auto => Ok(generator
                .and_then(|g| self.iter().find(|p| p.detect(g)))
                .unwrap_or_else(|| &*self.profiles[0])),
        }
    }
}

/// A root named a profile that is not compiled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownProfile(pub String);

impl std::fmt::Display for UnknownProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown profile {:?}", self.0)
    }
}

impl std::error::Error for UnknownProfile {}

#[cfg(test)]
mod tests;
