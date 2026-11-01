//! A declared profile's name and region, interned for the process.
//! [`Profile::name`](super::Profile::name) and the region are `'static`, and a
//! config's are known only at run time. Each distinct text is leaked the first
//! time anything asks for it and handed out again after, as [`selectors`](super::selectors)
//! keeps its parses, so a registry built again (as each test builds one)
//! adds nothing once its definitions have been seen: the cache is bounded by
//! the distinct definitions, never by builds or pages.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

static TEXTS: LazyLock<Mutex<HashSet<&'static str>>> = LazyLock::new(Default::default);
static LISTS: LazyLock<Mutex<HashMap<Vec<String>, &'static [&'static str]>>> =
    LazyLock::new(Default::default);

/// `text` as a `'static` str, the same one for every ask of the same text.
pub fn text(text: &str) -> &'static str {
    let mut texts = TEXTS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(found) = texts.get(text) {
        return found;
    }
    let leaked: &'static str = Box::leak(text.into());
    texts.insert(leaked);
    leaked
}

/// `texts` as a `'static` slice of `'static` strs, the same one for every ask of
/// the same list. Two threads asking at once each leak a slice, and the first
/// stored is the one handed out.
pub fn list(texts: &[String]) -> &'static [&'static str] {
    if let Some(found) = LISTS.lock().unwrap_or_else(|e| e.into_inner()).get(texts) {
        return found;
    }
    let interned: Vec<&'static str> = texts.iter().map(|t| text(t)).collect();
    let leaked: &'static [&'static str] = Box::leak(interned.into_boxed_slice());
    LISTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entry(texts.to_vec())
        .or_insert(leaked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_asked_for_again_is_the_same_str() {
        let first = text("interned-once");
        let second = text(&String::from("interned-once"));
        assert!(std::ptr::eq(first, second));
        assert!(!std::ptr::eq(first, text("interned-other")));
    }

    #[test]
    fn a_list_asked_for_again_is_the_same_slice_of_the_same_strs() {
        let written = vec!["div.one".to_string(), "main.two".to_string()];
        let first = list(&written);
        let second = list(&written.clone());
        assert!(std::ptr::eq(first, second));
        assert!(std::ptr::eq(first[0], text("div.one")));
        assert_eq!(first, ["div.one", "main.two"]);
    }
}
