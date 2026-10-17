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

    fn extract<'h>(&self, _facts: &'h RefCell<Facts>) -> ElementHandlers<'h> {
        Vec::new()
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
