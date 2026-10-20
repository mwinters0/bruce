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
    assert_eq!(resolved("nobody::install.adoc", &pages), None);
}

#[test]
fn a_refspec_no_page_has_the_coordinates_of_names_its_url() {
    let pages = site();
    let named = |reference| resolved(reference, &pages);
    // A page-alias, or a page the baseline lacks: the layout of I16 under the
    // prefix of the component-version, the latest where no version is given.
    assert_eq!(
        named("guide::missing.adoc#x").as_deref(),
        Some("https://x.org/guide/2.0/missing/#x")
    );
    assert_eq!(
        named("1.0@guide:admin:a/b.adoc").as_deref(),
        Some("https://x.org/guide/1.0/admin/a/b/")
    );
    // A bare version names no segment, and the module `ROOT` none.
    assert_eq!(
        named("ops::old.adoc").as_deref(),
        Some("https://x.org/ops/old/")
    );
    assert_eq!(
        named("ops:admin:old.adoc").as_deref(),
        Some("https://x.org/ops/admin/old/")
    );
    // The one URL of `spins.adoc` and `spins/index.adoc`, and the root's.
    assert_eq!(
        named("ops:admin:spins/index.adoc").as_deref(),
        Some("https://x.org/ops/admin/spins/")
    );
    assert_eq!(
        named("1.0@guide:admin:index.adoc").as_deref(),
        Some("https://x.org/guide/1.0/admin/")
    );
    assert_eq!(
        named("1.0@guide::index.adoc").as_deref(),
        Some("https://x.org/guide/1.0/")
    );
    // No page of the component-version, so no prefix to name a URL under.
    assert_eq!(named("3.0@guide::missing.adoc"), None);
    assert_eq!(named("nobody::missing.adoc"), None);
}

#[test]
fn a_page_with_no_edit_link_is_named_by_its_component_version_alone() {
    let mut pages = Pages::new(&[]);
    // The page of a component's version carries no module or `rel_path`.
    pages.pages.push((
        url("https://x.org/a%20b/2.0/"),
        vec![
            (COMPONENT.to_owned(), "a b".to_owned()),
            (VERSION.to_owned(), "2.0".to_owned()),
        ],
    ));
    assert_eq!(
        resolved("2.0@a b::the page.adoc", &pages).as_deref(),
        Some("https://x.org/a%20b/2.0/the%20page/")
    );
}

#[test]
fn a_page_at_a_segment_that_is_not_its_version_names_no_prefix() {
    let mut pages = Pages::new(&[]);
    // The latest of a component whose latest version is served as `latest`.
    pages.pages.push((
        url("https://x.org/fedora/latest/"),
        vec![
            (COMPONENT.to_owned(), "fedora".to_owned()),
            (VERSION.to_owned(), "f44".to_owned()),
        ],
    ));
    assert_eq!(resolved("f44@fedora::new.adoc", &pages), None);
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
