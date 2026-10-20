use std::cmp::Ordering;

use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

/// Pages of a baseline, each its URL and `(component, version, module,
/// rel_path)`, and the component-versions a build holds.
struct Pages {
    pages: Vec<(Url, Vec<(String, String)>)>,
    built: Vec<(&'static str, Option<&'static str>)>,
}

impl Pages {
    fn new(pages: &[(&str, &str, Option<&str>, &str, &str)]) -> Pages {
        let facts = |c: &str, v: Option<&str>, m: &str, p: &str| {
            let mut facts = vec![(COMPONENT.to_owned(), c.to_owned())];
            facts.extend(v.map(|v| (VERSION.to_owned(), v.to_owned())));
            facts.push((MODULE.to_owned(), m.to_owned()));
            facts.push((REL_PATH.to_owned(), p.to_owned()));
            facts
        };
        Pages {
            pages: pages
                .iter()
                .map(|(u, c, v, m, p)| (url(u), facts(c, *v, m, p)))
                .collect(),
            built: Vec::new(),
        }
    }
}

impl Lookup for Pages {
    fn pages_with_facts(&self, facts: &[(&str, &str)]) -> Result<Vec<Url>, BoxError> {
        Ok(self
            .pages
            .iter()
            .filter(|(_, held)| {
                facts
                    .iter()
                    .all(|(k, v)| held.iter().any(|(hk, hv)| hk == k && hv == v))
            })
            .map(|(u, _)| u.clone())
            .collect())
    }

    fn facts(&self, page: &Url) -> Result<Vec<(String, String)>, BoxError> {
        Ok(self
            .pages
            .iter()
            .find(|(u, _)| u == page)
            .map(|(_, f)| f.clone())
            .unwrap_or_default())
    }

    fn built(&self, facts: &[(&str, &str)]) -> bool {
        let get = |key| facts.iter().find(|(k, _)| *k == key).map(|(_, v)| *v);
        self.built.iter().any(|(c, v)| {
            get(COMPONENT) == Some(c) && (get(VERSION).is_none() || get(VERSION) == *v)
        })
    }
}

fn site() -> Pages {
    Pages::new(&[
        (
            "https://x.org/guide/1.0/install/",
            "guide",
            Some("1.0"),
            "ROOT",
            "install.adoc",
        ),
        (
            "https://x.org/guide/2.0/install/",
            "guide",
            Some("2.0"),
            "ROOT",
            "install.adoc",
        ),
        (
            "https://x.org/guide/1.10/install/",
            "guide",
            Some("1.10"),
            "ROOT",
            "install.adoc",
        ),
        (
            "https://x.org/ops/admin/partitions/",
            "ops",
            Some("master"),
            "admin",
            "partitions.adoc",
        ),
        (
            "https://x.org/ops/",
            "ops",
            Some("master"),
            "ROOT",
            "index.adoc",
        ),
    ])
}

fn resolved(reference: &str, pages: &Pages) -> Option<String> {
    resolve(reference, pages).unwrap().map(String::from)
}

#[test]
fn a_refspec_is_its_version_component_module_page_and_fragment() {
    let spec = |version, component, module, rel_path, fragment| Refspec {
        version,
        component,
        module,
        rel_path,
        fragment,
    };
    assert_eq!(
        parse("1.0@guide::install.adoc#first"),
        Some(spec(
            Some("1.0"),
            "guide",
            "ROOT",
            "install.adoc",
            Some("first")
        ))
    );
    assert_eq!(
        parse("ops:admin:a/b.adoc"),
        Some(spec(None, "ops", "admin", "a/b.adoc", None))
    );
    assert_eq!(
        parse("ops:admin:page$x.adoc#"),
        None,
        "a family before the module is not a page"
    );
    assert_eq!(
        parse("guide::page$install.adoc"),
        None,
        "the family comes first, so this one is `guide::page$…` with no component"
    );
    assert_eq!(
        parse("page$ops:admin:x.adoc"),
        Some(spec(None, "ops", "admin", "x.adoc", None))
    );
    assert_eq!(parse("install.adoc"), None, "the page's own component");
    assert_eq!(
        parse("admin:install.adoc"),
        None,
        "the page's own component"
    );
    assert_eq!(parse(":admin:install.adoc"), None);
    assert_eq!(parse("guide::"), None);
    assert_eq!(parse("image$ops:admin:x.png"), None);
}

#[test]
fn a_refspec_resolves_to_the_page_with_its_coordinates() {
    let pages = site();
    assert_eq!(
        resolved("ops:admin:partitions.adoc#layout", &pages).as_deref(),
        Some("https://x.org/ops/admin/partitions/#layout")
    );
    assert_eq!(
        resolved("1.0@guide::install.adoc", &pages).as_deref(),
        Some("https://x.org/guide/1.0/install/")
    );
    assert_eq!(
        resolved("ops::index.adoc#", &pages).as_deref(),
        Some("https://x.org/ops/")
    );
    assert_eq!(resolved("3.0@guide::install.adoc", &pages), None);
    assert_eq!(resolved("guide::missing.adoc", &pages), None);
    assert_eq!(resolved("guide:admin:install.adoc", &pages), None);
    assert_eq!(resolved("nobody::install.adoc", &pages), None);
}

#[test]
fn no_version_is_the_latest_the_baseline_holds() {
    let pages = site();
    assert_eq!(
        resolved("guide::install.adoc#first-boot", &pages).as_deref(),
        Some("https://x.org/guide/2.0/install/#first-boot")
    );
}

#[test]
fn a_refspec_into_a_component_version_the_build_holds_stays_unresolved() {
    let mut pages = site();
    pages.built = vec![("guide", Some("2.0"))];
    assert_eq!(
        resolved("guide::install.adoc", &pages),
        None,
        "latest, held"
    );
    assert_eq!(resolved("2.0@guide::install.adoc", &pages), None);
    assert_eq!(
        resolved("1.0@guide::install.adoc", &pages).as_deref(),
        Some("https://x.org/guide/1.0/install/"),
        "a version the build lacks is the baseline's"
    );
    assert_eq!(
        resolved("ops:admin:partitions.adoc", &pages).as_deref(),
        Some("https://x.org/ops/admin/partitions/")
    );
}

#[test]
fn versions_order_as_antora_lists_them() {
    let desc = |mut versions: Vec<&'static str>| {
        versions.sort_by(|a, b| compare(b, a));
        versions
    };
    assert_eq!(
        desc(vec!["1.9", "1.10", "2.0", "v1.2.3", "1.2.3-rc.1", ""]),
        ["", "2.0", "1.10", "1.9", "v1.2.3", "1.2.3-rc.1"]
    );
    assert_eq!(compare("master", "1.0"), Ordering::Greater);
    assert_eq!(compare("xenial", "trusty"), Ordering::Greater);
    assert_eq!(compare("1.0", "1.0.0"), Ordering::Equal);
    assert_eq!(compare("1.0-beta", "1.0"), Ordering::Less);
    assert_eq!(compare("", ""), Ordering::Equal);
}
