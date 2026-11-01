//! The extractor over the saved pages (D24). Hand-written assertions for each
//! property a decision names, each citing it; beside them one summary
//! snapshot a fixture, which `cargo insta review` shows a change to.

mod common;

use std::collections::BTreeMap;

use bruce::cli::registry;
use bruce::config::{HashScope, ProfileChoice};
use bruce::extract::{Extracted, Kind, Placement, extract};
use common::{Fixture, fixtures};
use url::Url;

fn run(f: &Fixture) -> Extracted {
    run_as(f, &f.html, &HashScope::default())
}

/// `f` with `html` for its body, hashed over `scope`.
fn run_as(f: &Fixture, html: &str, scope: &HashScope) -> Extracted {
    extract(html, &f.url, &ProfileChoice::Auto, scope, &registry())
        .unwrap_or_else(|e| panic!("{}: {e}", f.path))
}

fn count(x: &Extracted, placement: Placement) -> usize {
    x.anchors
        .iter()
        .filter(|a| a.placement == placement)
        .count()
}

/// What a snapshot holds: counts, never every anchor.
fn summary(x: &Extracted) -> String {
    let mut tags = BTreeMap::new();
    for a in &x.anchors {
        *tags.entry(a.tag.as_str()).or_insert(0) += 1;
    }
    let tags: Vec<_> = tags.iter().map(|(t, n)| format!("{t} {n}")).collect();
    let mut kinds = BTreeMap::new();
    for l in &x.links {
        *kinds.entry(l.kind).or_insert(0) += 1;
    }
    let kinds: Vec<_> = kinds
        .iter()
        .map(|(k, n)| format!("{} {n}", k.as_str()))
        .collect();
    let facts: Vec<_> = x
        .facts
        .pairs()
        .iter()
        .map(|(k, v)| format!("  {k} = {v}"))
        .collect();
    let edit = match (&x.edit_link, &x.source) {
        (None, _) => "-".to_string(),
        (Some(url), None) => format!("{url} (no forge shape)"),
        (Some(url), Some(s)) => format!(
            "{url} (repo {}, ref {}, path {})",
            s.repo,
            s.git_ref.as_deref().unwrap_or("-"),
            s.path
        ),
    };
    format!(
        "profile: {}\ntitle: {}\nedit: {edit}\nanchors: {} (content {}, chrome {}, unknown {})\nby tag: {}\nlinks: {} ({})\nhash: {:032x}\nfacts:{}\n",
        x.profile,
        x.title.as_deref().unwrap_or("-"),
        x.anchors.len(),
        count(x, Placement::Content),
        count(x, Placement::Chrome),
        count(x, Placement::Unknown),
        tags.join(", "),
        x.links.len(),
        kinds.join(", "),
        x.hash,
        if facts.is_empty() {
            " none".to_string()
        } else {
            format!("\n{}", facts.join("\n"))
        },
    )
}

#[test]
fn every_fixture_is_summarised() {
    let pages = fixtures();
    assert_eq!(pages.len(), 40);
    for f in &pages {
        insta::with_settings!({ snapshot_path => "snapshots", prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(f.path.replace('/', "__"), summary(&run(f)));
        });
    }
}

/// Every docs.fp.o page but the 404 declares Antora (I12) and has an
/// `article.doc` (I8), so none has an `unknown` anchor (D34).
#[test]
fn docs_fp_o_pages_are_antora_with_a_region() {
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let x = run(f);
        if f.path.ends_with("404.html") {
            assert_eq!(x.profile, "generic", "{}: no generator meta", f.path);
            continue;
        }
        assert_eq!(x.profile, "antora", "{}", f.path);
        assert_eq!(count(&x, Placement::Unknown), 0, "{}", f.path);
        assert!(x.title.is_some(), "{}", f.path);
    }
}

/// I9: the template's chrome ids sit outside the article on every page, and
/// an empty `id=""` is no anchor (D12).
#[test]
fn chrome_ids_are_chrome_on_every_antora_page() {
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let x = run(f);
        if x.profile != "antora" {
            continue;
        }
        let chrome: Vec<&str> = x
            .anchors
            .iter()
            .filter(|a| a.placement == Placement::Chrome)
            .map(|a| a.value.as_str())
            .collect();
        assert!(chrome.contains(&"search-input"), "{}: {chrome:?}", f.path);
        assert!(chrome.len() >= 3, "{}: {chrome:?}", f.path);
        assert!(x.anchors.iter().all(|a| !a.value.is_empty()), "{}", f.path);
    }
}

/// I9: `@asciidoctor/tabs` ids are static, so tabs are content anchors.
#[test]
fn tab_ids_are_content_anchors() {
    let f = fixtures()
        .into_iter()
        .find(|f| f.path == "generators/antora/tabs.html")
        .unwrap();
    let x = run(&f);
    let tabs: Vec<_> = x
        .anchors
        .iter()
        .filter(|a| a.value.starts_with("_tabs_"))
        .collect();
    assert!(!tabs.is_empty());
    assert!(tabs.iter().all(|a| a.placement == Placement::Content));
}

/// D34: `generic` draws its region at `<main>` or `[role=main]`, and a page
/// with neither says `unknown` rather than guess.
#[test]
fn generic_regions_follow_the_landmarks() {
    let by_path = |p: &str| {
        let f = fixtures().into_iter().find(|f| f.path == p).unwrap();
        run(&f)
    };
    for landmark in ["generic/main-landmark", "generic/role-main"] {
        let x = by_path(&format!("generators/{landmark}.html"));
        assert_eq!(x.profile, "generic");
        assert_eq!(count(&x, Placement::Unknown), 0, "{landmark}");
        assert!(count(&x, Placement::Content) > 0, "{landmark}");
    }
    let none = by_path("generators/generic/no-landmarks.html");
    assert!(!none.anchors.is_empty());
    assert_eq!(count(&none, Placement::Unknown), none.anchors.len());
}

/// D13: the survey's markup, every docs.fp.o page: three stylesheets and a
/// canonical on each but the 404, which has no canonical, and every href
/// resolving to an absolute URL.
#[test]
fn docs_fp_o_links_carry_every_kind_the_pages_use() {
    let of = |x: &Extracted, kind: Kind| x.links.iter().filter(|l| l.kind == kind).count();
    let mut canonicals = 0;
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let x = run(f);
        assert_eq!(of(&x, Kind::Stylesheet), 3, "{}", f.path);
        canonicals += of(&x, Kind::Canonical);
        assert!(of(&x, Kind::Anchor) > 50, "{}", f.path);
        for l in &x.links {
            let url = l
                .url
                .as_ref()
                .unwrap_or_else(|| panic!("{}: {}", f.path, l.raw));
            assert!(
                matches!(
                    url.scheme(),
                    "http" | "https" | "mailto" | "irc" | "matrix" | "tel"
                ),
                "{}: {}",
                f.path,
                l.raw
            );
        }
    }
    assert_eq!(canonicals, 29);
}

/// D13: a canonical is resolved to an absolute docs.fp.o URL, which for a
/// frozen version (`fedora/f39`) is the latest one's, not the page's own.
#[test]
fn a_canonical_is_absolute_and_may_name_another_page() {
    let canonical = |path: &str| {
        let f = fixtures().into_iter().find(|f| f.path == path).unwrap();
        let x = run(&f);
        let link = x.links.iter().find(|l| l.kind == Kind::Canonical).unwrap();
        (link.raw.clone(), link.url.clone().unwrap().to_string())
    };
    let (_, own) = canonical("docs-fp-o/en-US/a11y/index.html");
    assert_eq!(own, "https://docs.fedoraproject.org/en-US/a11y/");
    let (raw, frozen) = canonical("docs-fp-o/en-US/fedora/f39/index.html");
    assert!(raw.starts_with("https://"), "{raw}");
    assert_eq!(
        frozen,
        "https://docs.fedoraproject.org/en-US/fedora/latest/"
    );
}

/// The edit link and the source file it parses to, of the fixture at `path`.
fn edit_link(path: &str) -> Option<(Url, Option<bruce::forge::Source>)> {
    let f = fixtures().into_iter().find(|f| f.path == path).unwrap();
    let x = run(&f);
    x.edit_link.map(|url| (url, x.source))
}

/// D36, I8: "Edit this Page" is picked from "Page history" and "Report an
/// issue" by its title alone, on every docs.fp.o page but the 404, which has
/// none; its forge shape names a file in a repo.
#[test]
fn docs_fp_o_edit_links_are_the_edit_this_page_title() {
    let mut forges = std::collections::BTreeSet::new();
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let x = run(f);
        if f.path.ends_with("404.html") {
            assert!(x.edit_link.is_none(), "{}", f.path);
            continue;
        }
        let edit = x
            .edit_link
            .unwrap_or_else(|| panic!("{}: no edit link", f.path));
        let source = x.source.unwrap_or_else(|| panic!("{}: {edit}", f.path));
        assert!(source.path.ends_with(".adoc"), "{}: {edit}", f.path);
        assert!(edit.as_str().ends_with(&source.path));
        forges.insert(source.repo.split('/').nth(2).unwrap().to_string());
    }
    // The survey's three hosts: Forgejo, GitHub and GitLab.
    assert_eq!(
        forges.into_iter().collect::<Vec<_>>(),
        ["forge.fedoraproject.org", "github.com", "gitlab.com"]
    );
}

/// D36: each generator's edit link is found by the signal that generator
/// gives; on the generic page `rel=edit` outranks an earlier `a.edit`.
#[test]
fn each_generators_edit_link_is_found_by_its_own_signal() {
    for (path, url, repo, git_ref, file) in [
        (
            "generators/docsy/with-landmarks.html",
            "https://github.com/example/site/edit/main/content/en/docs/get-started/_index.md",
            "https://github.com/example/site",
            "main",
            "content/en/docs/get-started/_index.md",
        ),
        (
            "generators/docusaurus/with-landmarks.html",
            "https://github.com/example/site/edit/main/website/docs/installation.mdx",
            "https://github.com/example/site",
            "main",
            "website/docs/installation.mdx",
        ),
        (
            "generators/furo/with-landmarks.html",
            "https://github.com/example/site/edit/main/docs/quickstart.md",
            "https://github.com/example/site",
            "main",
            "docs/quickstart.md",
        ),
        (
            "generators/material/with-landmarks.html",
            "https://github.com/example/site/edit/master/docs/getting-started.md",
            "https://github.com/example/site",
            "master",
            "docs/getting-started.md",
        ),
        (
            "generators/sphinx-rtd/text-only-edit.html",
            "https://github.com/example/site/blob/main/docs/user/intro/sphinx.rst",
            "https://github.com/example/site",
            "main",
            "docs/user/intro/sphinx.rst",
        ),
        (
            "generators/generic/edit-candidates.html",
            "https://forge.example.org/docs/site/src/branch/main/pages/edit-candidates.md",
            "https://forge.example.org/docs/site",
            "main",
            "pages/edit-candidates.md",
        ),
    ] {
        let (edit, source) = edit_link(path).unwrap_or_else(|| panic!("{path}: no edit link"));
        let source = source.unwrap();
        assert_eq!(edit.as_str(), url, "{path}");
        assert_eq!(
            (
                source.repo.as_str(),
                source.git_ref.as_deref().unwrap(),
                source.path.as_str()
            ),
            (repo, git_ref, file),
            "{path}"
        );
    }
}

/// A page with no edit candidate has none (D36).
#[test]
fn a_page_without_an_edit_signal_has_no_edit_link() {
    assert!(edit_link("generators/generic/main-landmark.html").is_none());
    assert!(edit_link("generators/generic/role-main.html").is_none());
}

fn fact<'x>(x: &'x Extracted, key: &str) -> Option<&'x str> {
    x.facts
        .pairs()
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// D19, I8, I13: every docs.fp.o Antora page has a component, a module and a
/// `rel_path`. A version is recorded verbatim, placeholder included, exactly
/// where the page has a nav container; `arm-sbc` has none, and takes its
/// component from its URL. On every page with both, the URL's first segment
/// under `/en-US/` agrees with the nav container.
#[test]
fn antora_coordinates_come_from_the_nav_container_url_and_edit_link() {
    let mut versions = BTreeMap::new();
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/") && !f.path.ends_with("404.html"))
    {
        let x = run(f);
        let under_root = f.url.path().strip_prefix("/en-US/").unwrap();
        let first = under_root.split('/').next().unwrap();
        assert_eq!(fact(&x, "antora.component"), Some(first), "{}", f.path);
        let edit = x.source.as_ref().unwrap();
        let module = fact(&x, "antora.module").unwrap();
        let rel_path = fact(&x, "antora.rel_path").unwrap();
        assert!(
            edit.path
                .ends_with(&format!("modules/{module}/pages/{rel_path}")),
            "{}: {}",
            f.path,
            edit.path
        );
        let has_nav = f.html.contains("data-component=");
        assert_eq!(fact(&x, "antora.version").is_some(), has_nav, "{}", f.path);
        if let Some(v) = fact(&x, "antora.version") {
            versions.insert(v.to_string(), ());
        }
    }
    // The placeholders `""` and `master` are kept as written (I8).
    assert_eq!(
        versions.into_keys().collect::<Vec<_>>(),
        ["", "f39", "f44", "master"]
    );
    let arm = fixtures()
        .into_iter()
        .find(|f| f.path == "docs-fp-o/en-US/arm-sbc/index.html")
        .unwrap();
    let x = run(&arm);
    assert_eq!(
        x.facts.pairs(),
        [
            ("antora.component".to_string(), "arm-sbc".to_string()),
            ("antora.module".to_string(), "ROOT".to_string()),
            ("antora.rel_path".to_string(), "index.adoc".to_string()),
        ]
    );
}

/// I6, D20: each `a.xref.unresolved` link is tagged with its href less the
/// leading `#`, the raw refspec, a fragment after a second `#` included; no
/// other link is tagged.
#[test]
fn unresolved_xrefs_are_tagged_with_their_refspec() {
    let mut tagged = Vec::new();
    for f in fixtures() {
        let x = run(&f);
        for l in x.links.iter().filter(|l| l.tag.is_some()) {
            let tag = l.tag.as_deref().unwrap();
            assert_eq!(l.raw.strip_prefix('#'), Some(tag), "{}", f.path);
            assert_eq!(l.kind, Kind::Anchor);
            tagged.push((f.path.clone(), tag.to_string()));
        }
    }
    // The survey's nine across five docs.fp.o pages ("Facts found while
    // grilling"), and the hand-written Antora page's one.
    let fp: Vec<_> = tagged
        .iter()
        .filter(|(p, _)| p.starts_with("docs-fp-o/"))
        .collect();
    assert_eq!(fp.len(), 9);
    let pages: std::collections::BTreeSet<_> = fp.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(pages.len(), 5);
    assert!(tagged.contains(&(
        "generators/antora/tabs.html".to_string(),
        "example::missing.adoc#frag".to_string()
    )));
    assert_eq!(tagged.len(), 10);
    assert!(tagged.contains(&(
        "docs-fp-o/en-US/fesco/Spins/index.html".to_string(),
        "program_management::changes_policy.adoc#_system_wide_changes".to_string()
    )));
}

/// The registry the deployment guide's `[[profile]]` makes: the block of
/// `docs/manual/deploy.md` that `mise run sim` extracts into the config.
fn fedora_registry() -> bruce::profile::Registry {
    let guide = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/manual/deploy.md"),
    )
    .unwrap();
    let block = guide
        .split("```toml file=bruce.toml\n")
        .skip(1)
        .map(|rest| rest.split("```").next().unwrap())
        .find(|block| block.contains("[[profile]]"))
        .expect("the guide has a profile block");
    let defined = bruce::config::Config::parse(block).unwrap().policy.profiles;
    assert_eq!(defined.len(), 1);
    bruce::cli::registry_for(&defined).unwrap()
}

fn run_fedora(f: &Fixture) -> Extracted {
    extract(
        &f.html,
        &f.url,
        &ProfileChoice::Auto,
        &HashScope::default(),
        &fedora_registry(),
    )
    .unwrap()
}

/// I25, D67: every docs.fp.o page's language menu names the page under each
/// of the 67 other locales, so each is a locale's prefix on the host (D10),
/// to the guide's `fedora` profile and to it alone: compiled `antora` reads
/// no menu. The 404 has no generator meta, and its menu is not read.
#[test]
fn a_docs_fp_o_language_menu_names_every_other_locale() {
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/") && !f.path.ends_with("404.html"))
    {
        let x = run_fedora(f);
        assert_eq!(x.profile, "fedora", "{}", f.path);
        let locales: Vec<_> = x.other_locales.iter().map(|u| u.as_str()).collect();
        assert_eq!(locales.len(), 67, "{}", f.path);
        assert_eq!(
            locales[0], "https://docs.fedoraproject.org/ar/",
            "{}",
            f.path
        );
        assert!(locales.contains(&"https://docs.fedoraproject.org/zh_TW/"));
        assert!(!locales.iter().any(|l| l.contains("en-US")), "{}", f.path);
        assert!(run(f).other_locales.is_empty(), "{}: compiled", f.path);
    }
    let generic = fixtures()
        .into_iter()
        .filter(|f| !f.path.starts_with("docs-fp-o/en-US/") || f.path.ends_with("404.html"));
    for f in generic {
        assert!(run_fedora(&f).other_locales.is_empty(), "{}", f.path);
    }
}

/// D33: through the guide's profile every fixture is extracted as compiled
/// `antora` extracts it, the profile's name and locales aside.
#[test]
fn the_guides_profile_extracts_what_antora_does() {
    for f in fixtures() {
        let (plain, fedora) = (run(&f), run_fedora(&f));
        let name = if plain.profile == "antora" {
            "fedora"
        } else {
            plain.profile
        };
        assert_eq!(fedora.profile, name, "{}", f.path);
        assert_eq!(
            summary(&fedora).replacen(&format!("profile: {name}"), "profile: ", 1),
            summary(&plain).replacen(&format!("profile: {}", plain.profile), "profile: ", 1),
            "{}",
            f.path
        );
    }
}

/// docs.antora.org, on Antora's default UI, is right under compiled `antora`
/// with no config (the spec's "The line"): component and version from the
/// nav container, the edit link and its forge shape, and a region.
#[test]
fn docs_antora_org_is_read_right_by_compiled_antora_alone() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/docs-antora-org");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    // url: component, version, repo, ref.
    let expected = BTreeMap::from([
        ("antora/latest/", ("antora", "3.2", "antora/antora", "main")),
        ("antora/3.1/", ("antora", "3.1", "antora/antora", "v3.1.x")),
        (
            "antora/latest/install/install-antora/",
            ("antora", "3.2", "antora/antora", "main"),
        ),
        (
            "antora/latest/page/xref/",
            ("antora", "3.2", "antora/antora", "main"),
        ),
        (
            "antora/latest/extend/extensions/",
            ("antora", "3.2", "antora/antora", "main"),
        ),
        (
            "antora/latest/playbook/",
            ("antora", "3.2", "antora/antora", "main"),
        ),
        (
            "antora-ui-default/",
            (
                "antora-ui-default",
                "",
                "antora/antora-ui-default",
                "master",
            ),
        ),
        (
            "antora-ui-default/add-fonts/",
            (
                "antora-ui-default",
                "",
                "antora/antora-ui-default",
                "master",
            ),
        ),
        (
            "assembler/latest/",
            ("assembler", "1.0", "antora/antora-assembler", "main"),
        ),
        (
            "collector-extension/latest/concepts/",
            (
                "collector-extension",
                "1.0",
                "antora/antora-collector",
                "main",
            ),
        ),
        (
            "gradle-plugin/latest/",
            (
                "gradle-plugin",
                "1.0",
                "antora/gradle-antora-plugin",
                "main",
            ),
        ),
        (
            "maven-plugin/latest/antora-goal/",
            ("maven-plugin", "1.0", "antora/antora-maven-plugin", "main"),
        ),
    ]);
    let pages = manifest["pages"].as_array().unwrap();
    assert_eq!(pages.len(), expected.len() + 1, "and the 404");
    for page in pages {
        let (url, path) = (
            page["url"].as_str().unwrap(),
            page["path"].as_str().unwrap(),
        );
        let html = std::fs::read_to_string(root.join(path)).unwrap();
        let x = extract(
            &html,
            &Url::parse(url).unwrap(),
            &ProfileChoice::Auto,
            &HashScope::default(),
            &registry(),
        )
        .unwrap();
        assert_eq!(x.profile, "antora", "{url}: the generator meta alone");
        assert!(x.other_locales.is_empty(), "{url}");
        let key = url.strip_prefix("https://docs.antora.org/").unwrap();
        let Some((component, version, repo, git_ref)) = expected.get(key) else {
            assert_eq!(page["status"], 404, "{url}");
            assert!(x.facts.pairs().is_empty() && x.edit_link.is_none(), "{url}");
            continue;
        };
        assert_eq!(fact(&x, "antora.component"), Some(*component), "{url}");
        assert_eq!(fact(&x, "antora.version"), Some(*version), "{url}");
        // I26: a start path's `pages/` is `ROOT`'s; the rest are under `modules/`.
        if key == "assembler/latest/" {
            assert_eq!(fact(&x, "antora.module"), Some("ROOT"), "{url}");
            assert_eq!(fact(&x, "antora.rel_path"), Some("index.adoc"), "{url}");
        }
        let source = x.source.as_ref().expect(url);
        assert_eq!(source.repo, format!("https://gitlab.com/{repo}"), "{url}");
        assert_eq!(source.git_ref.as_deref(), Some(*git_ref), "{url}");
        assert!(count(&x, Placement::Content) > 0, "{url}: the region");
        assert_eq!(count(&x, Placement::Unknown), 0, "{url}");
    }
}

/// D15: docs.fp.o's `Last build:` line changes every page's bytes at every
/// build, and neither the default scope (`article.doc`) nor
/// `hash_exclude = ["footer"]` sees it; the whole page does. A prose edit
/// changes both.
#[test]
fn a_new_build_of_a_docs_fp_o_page_keeps_its_hash() {
    let scopes = |root: &[&str], exclude: &[&str]| HashScope {
        root: root.iter().map(|s| s.to_string()).collect(),
        exclude: exclude.iter().map(|s| s.to_string()).collect(),
    };
    let (default, footer, whole) = (
        scopes(&[], &[]),
        scopes(&[], &["footer"]),
        scopes(&["html"], &[]),
    );
    let mut built = 0;
    for f in fixtures()
        .iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
    {
        let Some(at) = f.html.find("Last build: ") else {
            continue;
        };
        built += 1;
        let at = at + "Last build: ".len();
        let rebuilt = format!(
            "{}1999-01-01 00:00:00 UTC{}",
            &f.html[..at],
            &f.html[at + 23..]
        );
        assert_ne!(rebuilt, f.html, "{}", f.path);
        for scope in [&default, &footer] {
            assert_eq!(
                run_as(f, &f.html, scope).hash,
                run_as(f, &rebuilt, scope).hash,
                "{}: {scope:?}",
                f.path
            );
        }
        assert_ne!(
            run_as(f, &f.html, &whole).hash,
            run_as(f, &rebuilt, &whole).hash,
            "{}",
            f.path
        );
        let article = f.html.find("<article class=\"doc").expect(&f.path);
        let open = article + f.html[article..].find('>').unwrap() + 1;
        let edited = format!("{}<p>new</p>{}", &f.html[..open], &f.html[open..]);
        for scope in [&default, &footer] {
            assert_ne!(
                run_as(f, &f.html, scope).hash,
                run_as(f, &edited, scope).hash,
                "{}: {scope:?}",
                f.path
            );
        }
    }
    assert_eq!(built, 30);
}

/// D33: a declared profile extending `antora` with no rule of its own reads
/// every page as `antora` does, by the same facts in `antora.*`, claiming the
/// pages `antora` claims and no others, and records its own name.
#[test]
fn an_extension_with_no_rules_extracts_what_antora_does() {
    let extension =
        bruce::config::Config::parse("[[profile]]\nname = \"fedora\"\nextends = \"antora\"\n")
            .unwrap()
            .policy
            .profiles;
    let registry = bruce::cli::registry_for(&extension).unwrap();
    let mut claimed = 0;
    for f in fixtures() {
        let plain = run(&f);
        let extended = extract(
            &f.html,
            &f.url,
            &ProfileChoice::Auto,
            &HashScope::default(),
            &registry,
        )
        .unwrap();
        let expected = if plain.profile == "antora" {
            claimed += 1;
            "fedora"
        } else {
            plain.profile
        };
        assert_eq!(extended.profile, expected, "{}", f.path);
        assert_eq!(
            summary(&extended).replacen(&format!("profile: {expected}"), "profile: ", 1),
            summary(&plain).replacen(&format!("profile: {}", plain.profile), "profile: ", 1),
            "{}",
            f.path
        );
        assert_eq!(extended.other_locales, plain.other_locales, "{}", f.path);
    }
    assert!(claimed > 20, "{claimed}");
}
