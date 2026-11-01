use super::*;

struct Stub {
    name: &'static str,
    claims: &'static str,
}

impl Profile for Stub {
    fn name(&self) -> &'static str {
        self.name
    }

    fn detect(&self, generator: &str) -> bool {
        !self.claims.is_empty() && generator.starts_with(self.claims)
    }

    fn extract<'h>(&self, _sink: &'h RefCell<Sink>) -> Extract<'h> {
        Extract::region(&[])
    }

    fn resolve(&self, _: &str, _: &Url, _: &dyn Lookup) -> Result<Option<Url>, BoxError> {
        Ok(None)
    }

    fn views(&self) -> &'static [View] {
        &[]
    }
}

fn stub(name: &'static str, claims: &'static str) -> Box<dyn Profile> {
    Box::new(Stub { name, claims })
}

fn registry() -> Registry {
    Registry::new(stub("plain", "")).with(stub("fancy", "Fancy"))
}

#[test]
fn auto_picks_the_profile_claiming_the_generator() {
    let chosen = registry()
        .choose(&ProfileChoice::Auto, Some("Fancy 1.0"))
        .unwrap()
        .name();
    assert_eq!(chosen, "fancy");
}

#[test]
fn auto_falls_back_without_a_claim_or_a_generator() {
    let r = registry();
    for generator in [Some("Other 2"), None] {
        let chosen = r.choose(&ProfileChoice::Auto, generator).unwrap().name();
        assert_eq!(chosen, "plain");
    }
}

#[test]
fn a_named_profile_ignores_the_generator() {
    let r = registry();
    let choice = ProfileChoice::Named("plain".into());
    assert_eq!(
        r.choose(&choice, Some("Fancy 1.0")).unwrap().name(),
        "plain"
    );
}

#[test]
fn an_unknown_name_is_refused_with_the_name() {
    let err = registry()
        .choose(&ProfileChoice::Named("nope".into()), None)
        .err()
        .unwrap();
    assert_eq!(err.to_string(), "unknown profile \"nope\"");
}

#[test]
#[should_panic(expected = "registered twice")]
fn a_name_registered_twice_panics() {
    let _ = registry().with(stub("fancy", ""));
}

#[test]
fn facts_are_recorded_under_the_profiles_namespace() {
    let mut facts = Facts::new("antora");
    facts.record("component", "handbook");
    facts.record("module", "ROOT");
    assert_eq!(
        facts.pairs(),
        [
            ("antora.component".to_string(), "handbook".to_string()),
            ("antora.module".to_string(), "ROOT".to_string())
        ]
    );
}

#[test]
fn the_sink_hands_the_core_what_a_profile_wrote() {
    let mut sink = Sink::new("antora");
    sink.fact("module", "ROOT");
    sink.tag_link("comp::page.adoc");
    sink.prefer_edit_link("https://forge.example/edit");
    assert_eq!(sink.take_link_tag().as_deref(), Some("comp::page.adoc"));
    assert_eq!(sink.take_link_tag(), None, "a tag belongs to one link");
    assert_eq!(
        sink.preferred_edit_link(),
        Some("https://forge.example/edit")
    );
    let fr = Url::parse("https://docs.example/fr/").unwrap();
    sink.other_locale(fr.clone());
    sink.other_locale(fr.clone());
    assert_eq!(sink.take_other_locales(), [fr], "once each");
    assert!(sink.take_other_locales().is_empty());
    assert_eq!(
        sink.into_facts().pairs(),
        [("antora.module".to_string(), "ROOT".to_string())]
    );
}

fn locale(page: &str, href: &str) -> Option<String> {
    let page = Url::parse(page).unwrap();
    locale_prefix(&page, &page.join(href).unwrap()).map(String::from)
}

/// docs.fp.o's menu names the page under each locale, its own first, and
/// the 404's names each locale's home (I25).
#[test]
fn a_locale_is_where_a_menu_link_departs_from_the_page() {
    let a11y = "https://docs.fedoraproject.org/en-US/a11y/";
    let ar = Some("https://docs.fedoraproject.org/ar/".to_owned());
    assert_eq!(locale(a11y, "../../ar/a11y/"), ar);
    assert_eq!(locale(a11y, "../../en-US/a11y/"), None, "its own locale");
    let deep = "https://docs.fedoraproject.org/en-US/fedora/f39/";
    assert_eq!(locale(deep, "../../../ar/fedora/f39/"), ar);
    let missing = "https://docs.fedoraproject.org/en-US/no-such/";
    assert_eq!(locale(missing, "/en-US/../ar"), ar, "the 404's");
    assert_eq!(locale(missing, "/en-US/../en-US"), None);
    assert_eq!(
        locale("https://e.org/docs/en/a/?q#f", "/docs/pt_BR/a/?q#f").as_deref(),
        Some("https://e.org/docs/pt_BR/"),
        "below the host's root, with no query or fragment"
    );
    assert_eq!(locale("https://e.org/en/a", "/en/a/"), None);
    assert_eq!(locale(a11y, "https://other.org/ar/a11y/"), None);
    assert_eq!(locale(a11y, "http://docs.fedoraproject.org/ar/a11y/"), None);
}
