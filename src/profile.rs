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
use crate::forge::EditLink;

/// An error a profile or the store behind its [`Lookup`] hands back.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// What a profile contributes to the core's single rewriting pass: element
/// handlers keyed by selector, in registration order (D38).
pub type ElementHandlers<'h> = Vec<(Cow<'static, Selector>, ElementContentHandlers<'h>)>;

/// A profile's extension of the core (D38). Detect, Extract and Views are
/// built; Resolve is declared here and given a body by the Antora resolver
/// (P5).
pub trait Profile: Send + Sync {
    /// The name a root's `profile` key gives, and the namespace of the
    /// profile's facts and views (D37).
    fn name(&self) -> &'static str;

    /// Detect: whether the profile claims a page whose
    /// `<meta name="generator">` content is `generator` (D35). The fallback
    /// profile claims none; it is what no other claims.
    fn detect(&self, generator: &str) -> bool;

    /// Extract: what the profile adds to the core's one `HtmlRewriter` for a
    /// page: its content region, its handlers and its finish step (D38). The
    /// handlers write into `sink` and run on one element in registration
    /// order, after the core's title, anchor and link readers and before the
    /// core takes the link's tag, and none sees an ancestor, so context is a
    /// flag an earlier handler set.
    fn extract<'h>(&self, sink: &'h RefCell<Sink>) -> Extract<'h>;

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
/// underscore, by convention and unchecked, so two profiles' views collide
/// only where one profile's name is another's and `_` more.
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

/// A profile's finish step (D38). It owns what it reads, the core having
/// taken the sink back by the time it runs, so it borrows nothing.
pub type Finish = Box<dyn FnOnce(&Settled, &mut Sink)>;

/// What a profile's Extract hook hands the core for one page (D38).
pub struct Extract<'h> {
    /// Selectors for the elements that hold the page's content (D34). The
    /// core turns them into its own descendant handlers and decides each
    /// anchor's placement when the page ends, since a page with no region
    /// anywhere makes every placement `unknown`. It is also the content
    /// hash's default scope (D15). Each is one compound
    /// selector with no list or combinator, which the core extends with
    /// `[id]`, ` [id]` and ` a[name]`.
    pub region: &'static [&'static str],
    /// Handlers added to the core's pass.
    pub handlers: ElementHandlers<'h>,
    /// Runs once the pass has ended, over the core's settled result. It may
    /// share what the profile's handlers gathered (an `Rc`), which is how a
    /// handler's reading reaches it.
    pub finish: Option<Finish>,
}

impl Extract<'_> {
    /// An Extract with `region` and nothing else.
    pub fn region(region: &'static [&'static str]) -> Self {
        Extract {
            region,
            handlers: Vec::new(),
            finish: None,
        }
    }
}

/// The core's settled result for a page, which a profile's finish step reads
/// where a handler could not: a value like the chosen edit link is known
/// only when the page has ended (D38).
#[derive(Debug, Clone, Copy)]
pub struct Settled<'a> {
    /// The page's URL.
    pub url: &'a Url,
    /// The page's edit link, the core's ranked choice or the profile's own
    /// preference (D36); `None` where the page has none.
    pub edit_link: Option<&'a EditLink>,
}

/// What a profile's handlers and finish step write to (D38): the page's
/// facts, a tag for the link being read, and a preferred edit link. The core
/// reads the sink after the pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sink {
    facts: Facts,
    link_tag: Option<String>,
    edit_link: Option<String>,
}

impl Sink {
    /// An empty sink for the profile named `namespace`.
    pub fn new(namespace: &'static str) -> Sink {
        Sink {
            facts: Facts::new(namespace),
            link_tag: None,
            edit_link: None,
        }
    }

    /// Records the fact `key` (without the namespace) as `value` (D37).
    pub fn fact(&mut self, key: &str, value: impl Into<String>) {
        self.facts.record(key, value);
    }

    /// Tags the link the core is reading, for the profile's Resolve to
    /// find: `tag` is what Resolve is handed as the reference (I6). The
    /// core takes it after the profile's handlers have run, with the links
    /// the element yielded, so it must be set by a handler on that link's
    /// own element; one set elsewhere is dropped.
    pub fn tag_link(&mut self, tag: impl Into<String>) {
        self.link_tag = Some(tag.into());
    }

    /// Prefers `href` as the page's edit link over the core's ranking
    /// (D36), resolved against the page as any other href. One that names
    /// no web page is ignored.
    pub fn prefer_edit_link(&mut self, href: impl Into<String>) {
        self.edit_link = Some(href.into());
    }

    /// The tag set since the last call, taken by the core.
    pub fn take_link_tag(&mut self) -> Option<String> {
        self.link_tag.take()
    }

    /// The edit link a profile preferred, if one did.
    pub fn preferred_edit_link(&self) -> Option<&str> {
        self.edit_link.as_deref()
    }

    /// The facts recorded.
    pub fn into_facts(self) -> Facts {
        self.facts
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
