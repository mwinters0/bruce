//! A declarative profile (D33): a profile a config defines, with a content
//! region, fact rules, source rules, an edit link and a generator pattern, and no
//! resolution or views of its own (D38), as `generic` has none.
//!
//! One that extends a compiled profile (D94) wraps it: its Extract is the base's with
//! the extension's handlers, finish step and source rules added (its sources
//! first, its facts under its own name, the base's under the base's) and its
//! region replacing the base's where it states one; it detects a generator by
//! its pattern, else by the base's; it resolves and gives the sitemap's
//! sentence as the base does. Its Views are none, the base being in every
//! registry that holds the extension and installing its own once.
//!
//! The name and region outlive the registry that holds them, because
//! [`Profile::name`] and the region are `'static` and a config's are known
//! only at run time; [`interned`] hands the same ones to every registry built
//! from the same definition.
//!
//! A fact rule reads the first element that gives a value (D33): an attribute
//! is kept as written, an empty one included, and an element without it gives
//! none; without `attr` the value is the element's text, trimmed, which an
//! end tag closes, so an element the parser closes implicitly gives the text it
//! held at the page's end. A `pattern` that does not match gives none, so a
//! later element is read.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use lol_html::end_tag;
use regex::Regex;
use url::Url;

use super::{
    BoxError, Extract, GENERIC_REGION, Lookup, Profile, Sink, View, compiled, element, interned,
    text,
};
use crate::config::{Definition, Fact};
use crate::forge;

/// A profile built from a [`Definition`].
pub struct Declarative {
    name: &'static str,
    base: Option<Box<dyn Profile>>,
    generator: Option<Regex>,
    region: Option<&'static [&'static str]>,
    edit_link: Option<String>,
    facts: Arc<[Rule]>,
    sources: Arc<[forge::Rule]>,
}

/// One fact rule with its patterns compiled.
struct Rule {
    key: String,
    selector: String,
    attr: Option<String>,
    pattern: Option<Regex>,
}

/// What one rule has read of the page so far.
#[derive(Default)]
struct Reading {
    /// The fact, once an element has given it.
    value: Option<String>,
    /// The text of the element being read, undecoded, until its end tag.
    text: String,
    /// Whether an element is being read, so that none inside it or after it
    /// starts again.
    reading: bool,
}

impl Declarative {
    /// The profile `definition` describes, over `base`, the compiled profile
    /// it extends. `Err` names what cannot be built, which a definition the
    /// loader validated never holds.
    pub fn new(
        definition: &Definition,
        base: Option<Box<dyn Profile>>,
    ) -> Result<Declarative, String> {
        let generator = match &definition.generator {
            Some(g) => Some(Regex::new(g).map_err(|e| format!("generator: {e}"))?),
            None => None,
        };
        let region: Option<&'static [&'static str]> = if definition.region.is_empty() {
            None
        } else {
            for selector in &definition.region {
                compiled(selector).map_err(|e| format!("region: {selector:?}: {e}"))?;
            }
            Some(interned::list(&definition.region))
        };
        if let Some(selector) = &definition.edit_link {
            compiled(selector).map_err(|e| format!("edit_link: {selector:?}: {e}"))?;
        }
        let facts: Vec<Rule> = definition
            .facts
            .iter()
            .map(Rule::new)
            .collect::<Result<_, _>>()?;
        let sources: Vec<forge::Rule> = definition
            .sources
            .iter()
            .enumerate()
            .map(|(i, rule)| rule.build().map_err(|e| format!("source[{i}]: {e}")))
            .collect::<Result<_, _>>()?;
        Ok(Declarative {
            name: interned::text(&definition.name),
            base,
            generator,
            region,
            edit_link: definition.edit_link.clone(),
            facts: facts.into(),
            sources: sources.into(),
        })
    }
}

impl Rule {
    fn new(fact: &Fact) -> Result<Rule, String> {
        compiled(&fact.selector)
            .map_err(|e| format!("fact {}: selector {:?}: {e}", fact.key, fact.selector))?;
        let pattern = match &fact.pattern {
            Some(p) => Some(Regex::new(p).map_err(|e| format!("fact {}: pattern: {e}", fact.key))?),
            None => None,
        };
        Ok(Rule {
            key: fact.key.clone(),
            selector: fact.selector.clone(),
            attr: fact.attr.clone(),
            pattern,
        })
    }

    /// The fact `read` (an attribute as written, or an element's text,
    /// trimmed) gives: the pattern's one capture where there is a pattern, the
    /// whole where there is none.
    fn value(&self, read: &str) -> Option<String> {
        match &self.pattern {
            Some(pattern) => pattern
                .captures(read)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_owned()),
            None => Some(read.to_owned()),
        }
    }
}

/// Ends the reading of a text rule's element: its text, decoded and trimmed,
/// becomes the fact if the rule gives one.
fn close(rule: &Rule, reading: &mut Reading) {
    reading.reading = false;
    let raw = std::mem::take(&mut reading.text);
    let decoded = htmlize::unescape(raw);
    let trimmed = decoded.trim();
    if !trimmed.is_empty() && reading.value.is_none() {
        reading.value = rule.value(trimmed);
    }
}

impl Profile for Declarative {
    fn name(&self) -> &'static str {
        self.name
    }

    fn namespace(&self) -> &'static str {
        self.base
            .as_ref()
            .map_or(self.name, |base| base.namespace())
    }

    fn detect(&self, generator: &str) -> bool {
        match (&self.generator, &self.base) {
            (Some(pattern), _) => pattern.is_match(generator),
            (None, Some(base)) => base.detect(generator),
            (None, None) => false,
        }
    }

    fn extract<'h>(&self, sink: &'h RefCell<Sink>) -> Extract<'h> {
        let mut extract = match &self.base {
            Some(base) => base.extract(sink),
            None => Extract::region(GENERIC_REGION),
        };
        if let Some(region) = self.region {
            extract.region = region;
        }
        extract.sources = if extract.sources.is_empty() {
            Arc::clone(&self.sources)
        } else {
            self.sources
                .iter()
                .chain(extract.sources.iter())
                .cloned()
                .collect()
        };
        let name = self.name;
        let readings: Rc<RefCell<Vec<Reading>>> = Rc::new(RefCell::new(
            self.facts.iter().map(|_| Reading::default()).collect(),
        ));
        for (i, rule) in self.facts.iter().enumerate() {
            let (read, all) = (Rc::clone(&readings), Arc::clone(&self.facts));
            if let Some(attr) = rule.attr.clone() {
                extract.handlers.push(element(&rule.selector, move |el| {
                    let mut readings = read.borrow_mut();
                    if readings[i].value.is_none()
                        && let Some(written) = el.get_attribute(&attr)
                    {
                        readings[i].value = all[i].value(&written);
                    }
                    Ok(())
                }));
                continue;
            }
            extract.handlers.push(element(&rule.selector, move |el| {
                {
                    let mut readings = read.borrow_mut();
                    let reading = &mut readings[i];
                    if reading.value.is_some() || reading.reading || el.end_tag_handlers().is_none()
                    {
                        return Ok(());
                    }
                    reading.reading = true;
                    reading.text.clear();
                }
                let (read, all) = (Rc::clone(&read), Arc::clone(&all));
                el.on_end_tag(end_tag!(move |_| {
                    close(&all[i], &mut read.borrow_mut()[i]);
                    Ok(())
                }))
            }));
            let read = Rc::clone(&readings);
            extract.handlers.push(text(&rule.selector, move |chunk| {
                let mut readings = read.borrow_mut();
                if readings[i].reading {
                    readings[i].text.push_str(chunk.as_str());
                }
                Ok(())
            }));
        }
        if let Some(selector) = &self.edit_link {
            extract.handlers.push(element(selector, move |el| {
                let mut sink = sink.borrow_mut();
                if sink.preferred_edit_link().is_none()
                    && let Some(href) = el.get_attribute("href")
                {
                    sink.prefer_edit_link(href);
                }
                Ok(())
            }));
        }
        if !self.facts.is_empty() {
            let rules = Arc::clone(&self.facts);
            let base = extract.finish.take();
            extract.finish = Some(Box::new(move |settled, sink| {
                if let Some(base) = base {
                    base(settled, sink);
                }
                let mut readings = readings.borrow_mut();
                for (rule, reading) in rules.iter().zip(readings.iter_mut()) {
                    if reading.reading {
                        close(rule, reading);
                    }
                    if let Some(value) = reading.value.take() {
                        sink.fact_in(name, &rule.key, value);
                    }
                }
            }));
        }
        extract
    }

    fn resolve(
        &self,
        reference: &str,
        from: &Url,
        lookup: &dyn Lookup,
    ) -> Result<Option<Url>, BoxError> {
        match &self.base {
            Some(base) => base.resolve(reference, from, lookup),
            None => Ok(None),
        }
    }

    fn views(&self) -> &'static [View] {
        &[]
    }

    fn sitemap_note(&self) -> Option<&'static str> {
        self.base.as_ref().and_then(|base| base.sitemap_note())
    }
}

#[cfg(test)]
mod tests;
