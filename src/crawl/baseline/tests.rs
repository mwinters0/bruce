use std::path::Path;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::super::{Baseline, Externals};
use super::*;
use crate::config::Config;
use crate::fetch::http::host::Hosts;
use crate::fetch::http::{HttpFetcher, Settings};
use crate::fetch::local::LocalFetcher;
use crate::fetch::routed::Routed;
use crate::profile::Registry;
use crate::profile::antora::Antora;
use crate::profile::generic::Generic;
use crate::store::{BaselineIndex, Mode, RunId, RunRoot, Store, Writer};

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

#[test]
fn a_url_is_held_when_its_component_and_version_are() {
    let prefix = url("https://x.org/en-US/");
    let mut held = Held::default();
    let mut record = |at: &str, component, version| {
        let page = url(&format!("https://x.org/en-US/{at}"));
        held.record(&prefix, &page, component, version);
    };
    record("tools/", "tools", Some(""));
    record("ops/admin/", "ops", Some("master"));
    record("arm-sbc/", "arm-sbc", None);
    record("guide/1.0/install/", "guide", Some("1.0"));
    record("guide/2.0/", "guide", Some("2.0"));
    record("old/1.0/", "old", Some("1.0"));
    let holds = |at: &str| held.holds(&prefix, &url(&format!("https://x.org/en-US/{at}")));
    assert!(holds("tools/usage/"), "a bare version names no segment");
    assert!(holds("ops/admin/partitions/"));
    assert!(holds("arm-sbc/"));
    assert!(holds("guide/1.0/install/"));
    assert!(holds("guide/2.0/"));
    assert!(
        !holds("guide/3.0/install/"),
        "a version the build lacks is the baseline's"
    );
    assert!(!holds("guide/install/"), "no bare version is held");
    assert!(!holds("guide/"));
    assert!(!holds("legal/privacy/"), "a component the build lacks");
    assert!(!holds("_/css/site.css"));
    assert!(!holds(""), "the prefix itself is no component");
    assert!(
        holds("tools/1.0/"),
        "a bare version holds whatever follows its component"
    );
    assert!(
        !held.holds(&prefix, &url("https://x.org/en-GB/tools/")),
        "a URL not below the prefix"
    );
}

#[test]
fn a_component_name_is_compared_decoded() {
    let prefix = url("https://x.org/");
    let mut held = Held::default();
    held.record(&prefix, &url("https://x.org/a%20b/"), "a b", None);
    assert!(held.holds(&prefix, &url("https://x.org/a%20b/page/")));
}

#[test]
fn a_version_is_held_at_the_segment_its_pages_sit_at_and_named_by_its_own_name() {
    let prefix = url("https://x.org/en-US/");
    let mut held = Held::default();
    let mut record = |at: &str, component, version| {
        let page = url(&format!("https://x.org/en-US/{at}"));
        held.record(&prefix, &page, component, version);
    };
    // The latest served at `latest`, as a `latest_version_segment` serves it.
    record("fedora/latest/install/", "fedora", Some("f44"));
    record("fedora/f43/", "fedora", Some("f43"));
    record("fedora%20x/v%201/", "fedora x", Some("v 1"));
    // A page off the layout is held at its version's own name.
    record("elsewhere/p/", "odd", Some("1.0"));
    let holds = |at: &str| held.holds(&prefix, &url(&format!("https://x.org/en-US/{at}")));
    assert!(holds("fedora/latest/gone/"), "a page the change deleted");
    assert!(holds("fedora/f43/gone/"));
    assert!(
        holds("fedora%20x/v%201/p/"),
        "a segment is compared decoded"
    );
    assert!(
        !holds("fedora/f44/gone/"),
        "no built page sits at the latest's own name"
    );
    assert!(!holds("fedora/rawhide/"));
    assert!(holds("odd/1.0/p/"));
    assert!(!holds("odd/p/"));
    assert!(
        held.holds_coordinates("fedora", Some("f44")),
        "a refspec names f44"
    );
    assert!(
        !held.holds_coordinates("fedora", Some("latest")),
        "a segment is no version"
    );
}

fn registry() -> Registry {
    Registry::new(Box::new(Generic)).with(Box::new(Antora))
}

/// An Antora page of `component` at `version`, holding `body`.
fn page(component: &str, version: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta name="generator" content="Antora 3.1.15">
        <title>t</title></head><body>
        <div class="nav-container" data-component="{component}" data-version="{version}"></div>
        <article class="doc">{body}</article></body></html>"#
    )
}

/// A build in `dir` of `files`, each a path ending `index.html` and its text,
/// served under `prefix`: the fetcher over it, the seeds and the site.
fn build(dir: &Path, prefix: &str, files: &[(&str, String)]) -> (LocalFetcher, Vec<Url>) {
    let (mut served, mut seeds) = (Vec::new(), Vec::new());
    for (at, text) in files {
        let file = dir.join(at.replace('/', "_"));
        std::fs::write(&file, text).unwrap();
        let page = url(&format!("{prefix}{}", at.trim_end_matches("index.html")));
        seeds.push(page.clone());
        served.push((page, file));
    }
    (LocalFetcher::new(&url(prefix), served), seeds)
}

struct Checked {
    store: Store,
    run: RunId,
    summary: super::super::Summary,
}

/// One run over the build `local`, with `live` behind it, into `store`.
async fn check(
    store: Store,
    config: &str,
    local: LocalFetcher,
    seeds: &[Url],
    baseline: Baseline,
) -> Checked {
    let config = Config::parse(config).unwrap();
    let baseline_run = (baseline.run.0 > 0).then_some(baseline.run);
    let run = store
        .begin_run(Mode::Local, "test", baseline_run, &RunRoot::of(&config))
        .unwrap();
    let writer = Writer::start(store, 16);
    let live = HttpFetcher::new(Settings::default(), Hosts::unlimited()).unwrap();
    let site = local.site();
    let summary = super::super::Crawl::new(
        Routed::new(local, live),
        config,
        registry(),
        run,
        writer.handle(),
    )
    .against(site, baseline)
    .run_seeded(seeds)
    .await
    .unwrap();
    let store = writer.finish().unwrap();
    store.finish_run(run).unwrap();
    Checked {
        store,
        run,
        summary,
    }
}

async fn ok(server: &MockServer, method_: &str, at: &str) {
    Mock::given(method(method_))
        .and(path(at))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"<title>t</title><h2 id=\"x\">x</h2>".to_vec(), "text/html"),
        )
        .mount(server)
        .await;
}

/// The requests a server has had, robots.txt aside.
async fn asked(server: &MockServer) -> usize {
    let all = server.received_requests().await.unwrap();
    all.iter().filter(|r| r.url.path() != "/robots.txt").count()
}

impl Checked {
    /// Run `run`'s targets as `(path, outcome, reason, answered_by)`, the
    /// server's address cut from the URL.
    fn targets(&self, run: RunId, base: &str) -> Vec<(String, String, Option<String>, String)> {
        let mut stmt = self
            .store
            .connection()
            .prepare(
                "SELECT url, outcome, reason, answered_by FROM targets WHERE run_id = ? \
                 ORDER BY url",
            )
            .unwrap();
        stmt.query_map([run.0], |r| {
            Ok((
                r.get::<_, String>(0)?.replace(base, ""),
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
            ))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }

    /// The verdicts of the links from the page `from` (a path), as
    /// `(raw href, outcome)`.
    fn verdicts(&self, from: &str, base: &str) -> Vec<(String, String)> {
        let mut stmt = self
            .store
            .connection()
            .prepare(
                "SELECT v.raw_href, v.outcome FROM link_verdicts v \
                 JOIN targets s ON s.run_id = v.run_id AND s.url_id = v.src_url_id \
                 WHERE v.run_id = ? AND s.url = ? ORDER BY v.raw_href",
            )
            .unwrap();
        stmt.query_map(duckdb::params![self.run.0, format!("{base}{from}")], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }
}

fn t(
    path: &str,
    outcome: &str,
    reason: Option<&str>,
    by: &str,
) -> (String, String, Option<String>, String) {
    (
        path.to_owned(),
        outcome.to_owned(),
        reason.map(str::to_owned),
        by.to_owned(),
    )
}

/// The whole site as a baseline run, then a partial build of `tools` alone
/// checked against it.
#[tokio::test]
async fn a_partial_build_is_answered_by_the_build_the_baseline_and_production() {
    let production = MockServer::start().await;
    let outer = MockServer::start().await;
    let (base, port) = (production.uri(), outer.address().port());
    let config = format!("allowlist = [\"localhost\"]\n[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let outside = format!("http://localhost:{port}");
    ok(&outer, "GET", "/page").await;
    ok(&outer, "GET", "/new").await;
    ok(&production, "HEAD", "/docs/guide/1.0/new/").await;
    ok(&production, "GET", "/docs/guide/1.0/new/").await;
    let tmp = tempfile::tempdir().unwrap();

    // The baseline: the whole site, from no baseline of its own.
    let whole = [
        (
            "tools/index.html",
            page(
                "tools",
                "",
                &format!(
                    r#"<a href="usage/">u</a> <a href="reference/">r</a>
                    <a href="../guide/1.0/install/#first-boot">g</a>
                    <a href="../ops/">o</a> <a href="{outside}/page">p</a>"#
                ),
            ),
        ),
        (
            "tools/usage/index.html",
            page("tools", "", r#"<h2 id="running">r</h2>"#),
        ),
        ("tools/reference/index.html", page("tools", "", "ref")),
        (
            "guide/1.0/install/index.html",
            page("guide", "1.0", r#"<h2 id="first-boot">f</h2>"#),
        ),
        ("ops/index.html", page("ops", "master", "ops")),
    ];
    let (local, seeds) = build(tmp.path(), &prefix, &whole);
    let none = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Live,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let first = check(store, &config, local, &seeds, none).await;
    assert_eq!(first.targets(first.run, &base).len(), 6);

    // The change: `tools` alone, with `reference` deleted.
    let partial = [
        (
            "tools/index.html",
            page(
                "tools",
                "",
                &format!(
                    r#"<a href="usage/">u</a> <a href="reference/">r</a>
                    <a href="../guide/1.0/install/#first-boot">g</a>
                    <a href="../guide/1.0/install/#nope">n</a>
                    <a href="../guide/1.0/new/">new</a> <a href="../ops/">o</a>
                    <a href="{outside}/page">p</a> <a href="{outside}/new">n</a>"#
                ),
            ),
        ),
        (
            "tools/usage/index.html",
            page("tools", "", r#"<h2 id="running">r</h2>"#),
        ),
    ];
    let dir = tmp.path().join("partial");
    std::fs::create_dir(&dir).unwrap();
    let (local, seeds) = build(&dir, &prefix, &partial);
    let index = first.store.baseline_index(first.run).unwrap();
    let baseline = Baseline {
        run: first.run,
        index,
        externals: Externals::Baseline,
    };
    let requests_before = asked(&outer).await;
    let second = check(first.store, &config, local, &seeds, baseline).await;

    assert_eq!(
        second.targets(second.run, &base),
        [
            t("/docs/guide/1.0/install/", "ok", None, "baseline"),
            t("/docs/guide/1.0/new/", "ok", None, "live"),
            t("/docs/ops/", "ok", None, "baseline"),
            t("/docs/tools/", "ok", None, "local"),
            t(
                "/docs/tools/reference/",
                "broken_url",
                Some("not_built"),
                "local"
            ),
            t("/docs/tools/usage/", "ok", None, "local"),
            t(
                &format!("{outside}/new").replace(&base, ""),
                "ok",
                None,
                "live"
            ),
            t(&format!("{outside}/page"), "ok", None, "baseline"),
        ]
        .map(|(u, o, r, b)| (u.replace(&base, ""), o, r, b)),
    );
    assert_eq!(second.summary.targets, 8);
    assert_eq!(
        asked(&outer).await - requests_before,
        1,
        "only the URL the baseline lacks was asked of the outside host"
    );
    let verdicts = second.verdicts("/docs/tools/", &base);
    let of = |href: &str| {
        verdicts
            .iter()
            .find(|(h, _)| h == href)
            .unwrap_or_else(|| panic!("no link {href}"))
            .1
            .as_str()
    };
    assert_eq!(of("../guide/1.0/install/#first-boot"), "ok");
    assert_eq!(
        of("../guide/1.0/install/#nope"),
        "broken_anchor",
        "the copied page brought its anchors"
    );
    assert_eq!(of("reference/"), "broken_url");
    // The baseline run is untouched.
    assert_eq!(second.targets(first.run, &base).len(), 6);
}

#[tokio::test]
async fn externals_skip_copies_what_the_baseline_holds_and_asks_nothing_else() {
    let production = MockServer::start().await;
    let outer = MockServer::start().await;
    let (base, port) = (production.uri(), outer.address().port());
    let config = format!("allowlist = [\"localhost\"]\n[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let outside = format!("http://localhost:{port}");
    ok(&outer, "GET", "/page").await;
    ok(&outer, "GET", "/new").await;
    // A page of a component the build lacks, which production moves onto
    // the baseline's `/page`.
    Mock::given(method("GET"))
        .and(path("/docs/legal/moved/"))
        .respond_with(
            ResponseTemplate::new(301).insert_header("location", format!("{outside}/page")),
        )
        .mount(&production)
        .await;
    let tmp = tempfile::tempdir().unwrap();
    let first_files = [(
        "tools/index.html",
        page("tools", "", &format!(r#"<a href="{outside}/page">p</a>"#)),
    )];
    let files = [(
        "tools/index.html",
        page(
            "tools",
            "",
            &format!(
                r#"<a href="{outside}/page">p</a> <a href="{outside}/new">n</a>
                <a href="../legal/moved/">m</a>"#
            ),
        ),
    )];
    let (local, seeds) = build(tmp.path(), &prefix, &first_files);
    let live = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Live,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let first = check(store, &config, local, &seeds, live).await;
    let index = first.store.baseline_index(first.run).unwrap();

    let mut store = first.store;
    for (externals, page_, new) in [
        (
            Externals::Skip,
            ("ok", None, "baseline"),
            ("unchecked", Some("externals_skipped"), "live"),
        ),
        (
            Externals::Baseline,
            ("ok", None, "baseline"),
            ("ok", None, "live"),
        ),
        (Externals::Live, ("ok", None, "live"), ("ok", None, "live")),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let (local, seeds) = build(dir.path(), &prefix, &files);
        let baseline = Baseline {
            run: first.run,
            index: index.clone(),
            externals,
        };
        let before = asked(&outer).await;
        let done = check(store, &config, local, &seeds, baseline).await;
        let targets = done.targets(done.run, &base);
        let of = |url: String| {
            let t = targets.iter().find(|t| t.0 == url).unwrap();
            (t.1.as_str(), t.2.as_deref(), t.3.as_str())
        };
        assert_eq!(of(format!("{outside}/page")), page_, "{externals:?}");
        assert_eq!(of(format!("{outside}/new")), new, "{externals:?}");
        if externals == Externals::Skip {
            assert_eq!(
                of("/docs/legal/moved/".to_owned()),
                ("unchecked", Some("externals_skipped"), "live"),
                "a redirect onto an allowlisted host ends the chain under skip"
            );
            assert_eq!(asked(&outer).await, before, "skip asked the outside host");
        }
        store = done.store;
    }
}

#[tokio::test]
async fn a_build_holding_no_component_version_trusts_the_baseline_and_asks_for_what_it_lacks() {
    let production = MockServer::start().await;
    let base = production.uri();
    let config = format!("[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let tmp = tempfile::tempdir().unwrap();
    // A page with no nav container, at the prefix root, has no component
    // segment to take, so the build holds nothing and `gone/` is not
    // `not_built`.
    let files = [(
        "index.html",
        r#"<title>t</title><a href="gone/">g</a>"#.to_owned(),
    )];
    let (local, seeds) = build(tmp.path(), &prefix, &files);
    let none = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Baseline,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let done = check(store, &config, local, &seeds, none).await;
    assert_eq!(
        done.targets(done.run, &base),
        [
            t("/docs/", "ok", None, "local"),
            t("/docs/gone/", "broken_url", None, "live"),
        ]
    );
}

/// A build serving its latest at `latest` holds what sits under it, and
/// nothing under another version's segment.
#[tokio::test]
async fn a_page_the_change_deleted_under_a_renamed_latest_segment_is_not_built() {
    let production = MockServer::start().await;
    let base = production.uri();
    let config = format!("[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let tmp = tempfile::tempdir().unwrap();
    let files = [(
        "fedora/latest/index.html",
        page(
            "fedora",
            "f44",
            r#"<a href="gone/">g</a> <a href="../f43/gone/">o</a>"#,
        ),
    )];
    let (local, seeds) = build(tmp.path(), &prefix, &files);
    let none = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Baseline,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let done = check(store, &config, local, &seeds, none).await;
    assert_eq!(
        done.targets(done.run, &base),
        [
            t("/docs/fedora/f43/gone/", "broken_url", None, "live"),
            t("/docs/fedora/latest/", "ok", None, "local"),
            t(
                "/docs/fedora/latest/gone/",
                "broken_url",
                Some("not_built"),
                "local"
            ),
        ]
    );
    assert_eq!(asked(&production).await, 1, "the version the build lacks");
}

#[tokio::test]
async fn a_seeded_file_no_page_names_is_a_local_target_and_no_page() {
    let production = MockServer::start().await;
    let base = production.uri();
    let config = format!("[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let tmp = tempfile::tempdir().unwrap();
    let files = [
        ("index.html", page("tools", "", "tools")),
        ("tools/_images/lonely.png", "not a png".to_owned()),
    ];
    let (local, seeds) = build(tmp.path(), &prefix, &files);
    let none = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Baseline,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let done = check(store, &config, local, &seeds, none).await;
    assert_eq!(
        done.targets(done.run, &base),
        [
            t("/docs/", "ok", None, "local"),
            t("/docs/tools/_images/lonely.png", "ok", None, "local"),
        ]
    );
    let pages: i64 = done
        .store
        .connection()
        .query_row(
            "SELECT count(*) FROM pages WHERE run_id = ?",
            [done.run.0],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(pages, 1, "a file that is no page has no page row");
}

/// An Antora page of `component` at `version`, whose source file is
/// `modules/<module>/pages/<file>` of a repository, holding `body`.
fn sourced(component: &str, version: &str, source: (&str, &str), body: &str) -> String {
    let (module, file) = source;
    let edit = format!(
        r#"<a rel="edit" href="https://github.com/o/r/edit/main/modules/{module}/pages/{file}">e</a>"#
    );
    page(component, version, &format!("{edit}{body}"))
}

/// The whole site as a baseline run, then `tools` alone, whose xrefs into
/// the components it lacks Antora left as `#<refspec>`.
#[tokio::test]
async fn an_xref_into_a_component_the_build_lacks_resolves_through_the_baseline() {
    let production = MockServer::start().await;
    let base = production.uri();
    let config = format!("[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let tmp = tempfile::tempdir().unwrap();
    let anchors = r#"<h2 id="first-boot">f</h2>"#;
    let whole = [
        ("tools/index.html", page("tools", "", "tools")),
        (
            "guide/1.0/install/index.html",
            sourced("guide", "1.0", ("ROOT", "install.adoc"), anchors),
        ),
        (
            "guide/2.0/install/index.html",
            sourced("guide", "2.0", ("ROOT", "install.adoc"), anchors),
        ),
        (
            "ops/admin/partitions/index.html",
            sourced("ops", "master", ("admin", "partitions.adoc"), "ops"),
        ),
    ];
    let (local, seeds) = build(tmp.path(), &prefix, &whole);
    let none = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Live,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let first = check(store, &config, local, &seeds, none).await;

    let xrefs = [
        "guide::install.adoc#first-boot",
        "1.0@guide::install.adoc#nope",
        "ops:admin:partitions.adoc",
        "ops:admin:missing.adoc",
        "tools::missing.adoc",
        "nobody::install.adoc",
    ]
    .map(|refspec| format!(r##"<a class="xref unresolved" href="#{refspec}">{refspec}</a> "##));
    let partial = [("tools/index.html", page("tools", "", &xrefs.concat()))];
    let dir = tmp.path().join("partial");
    std::fs::create_dir(&dir).unwrap();
    let (local, seeds) = build(&dir, &prefix, &partial);
    let baseline = Baseline {
        run: first.run,
        index: first.store.baseline_index(first.run).unwrap(),
        externals: Externals::Baseline,
    };
    let second = check(first.store, &config, local, &seeds, baseline).await;

    assert_eq!(
        second.targets(second.run, &base),
        [
            t("/docs/guide/1.0/install/", "ok", None, "baseline"),
            t("/docs/guide/2.0/install/", "ok", None, "baseline"),
            t("/docs/ops/admin/missing/", "broken_url", None, "live"),
            t("/docs/ops/admin/partitions/", "ok", None, "baseline"),
            t("/docs/tools/", "ok", None, "local"),
        ]
        .map(|(u, o, r, b)| (u.replace(&base, ""), o, r, b)),
    );
    let verdicts = second.verdicts("/docs/tools/", &base);
    let of = |refspec: &str| {
        let raw = format!("#{refspec}");
        let (_, outcome) = verdicts.iter().find(|(h, _)| *h == raw).unwrap();
        outcome.as_str()
    };
    assert_eq!(of("guide::install.adoc#first-boot"), "ok", "the latest");
    assert_eq!(of("1.0@guide::install.adoc#nope"), "broken_anchor");
    assert_eq!(of("ops:admin:partitions.adoc"), "ok");
    // Not a page of the baseline, or a component the build holds, or the URL
    // of one (D57), which production does not serve either.
    for refspec in [
        "ops:admin:missing.adoc",
        "tools::missing.adoc",
        "nobody::install.adoc",
    ] {
        assert_eq!(of(refspec), "broken_url", "{refspec}");
    }
    assert_eq!(
        asked(&production).await,
        1,
        "the URL ops:admin:missing names"
    );
}

/// The whole site as a baseline run, holding a crawled page-alias and a page
/// whose source is `spins/index.adoc`, then `tools` alone xref-ing the alias,
/// the shared URL, an alias the crawl did not reach and a page nobody serves.
#[tokio::test]
async fn an_xref_no_page_has_the_coordinates_of_resolves_by_the_url_it_names() {
    let production = MockServer::start().await;
    let base = production.uri();
    let config = format!("[[root]]\nurl = \"{base}/docs/\"\n");
    let prefix = format!("{base}/docs/");
    let tmp = tempfile::tempdir().unwrap();
    let alias = r#"<title>r</title><meta http-equiv="refresh" content="0; url=../initiatives/">"#;
    let whole = [
        ("tools/index.html", page("tools", "", "tools")),
        (
            "project/initiatives/index.html",
            sourced("project", "", ("ROOT", "initiatives.adoc"), "p"),
        ),
        ("project/objectives/index.html", alias.to_owned()),
        (
            "releases/spins/index.html",
            sourced(
                "releases",
                "master",
                ("ROOT", "spins/index.adoc"),
                r#"<h2 id="x">x</h2>"#,
            ),
        ),
    ];
    let (local, seeds) = build(tmp.path(), &prefix, &whole);
    let none = Baseline {
        run: RunId(0),
        index: BaselineIndex::default(),
        externals: Externals::Live,
    };
    let store = Store::open_in_memory(&registry()).unwrap();
    let first = check(store, &config, local, &seeds, none).await;

    let xrefs = [
        "project::objectives.adoc",
        "releases::spins.adoc#x",
        "project::uncrawled.adoc",
        "project::nowhere.adoc",
    ]
    .map(|refspec| format!(r##"<a class="xref unresolved" href="#{refspec}">{refspec}</a> "##));
    let partial = [("tools/index.html", page("tools", "", &xrefs.concat()))];
    let dir = tmp.path().join("partial");
    std::fs::create_dir(&dir).unwrap();
    let (local, seeds) = build(&dir, &prefix, &partial);
    let baseline = Baseline {
        run: first.run,
        index: first.store.baseline_index(first.run).unwrap(),
        externals: Externals::Baseline,
    };
    ok(&production, "GET", "/docs/project/uncrawled/").await;
    let second = check(first.store, &config, local, &seeds, baseline).await;

    let verdicts = second.verdicts("/docs/tools/", &base);
    let of = |refspec: &str| {
        let raw = format!("#{refspec}");
        let (_, outcome) = verdicts.iter().find(|(h, _)| *h == raw).unwrap();
        outcome.as_str()
    };
    assert_eq!(of("project::objectives.adoc"), "ok", "a crawled alias");
    assert_eq!(of("releases::spins.adoc#x"), "ok", "a shared URL");
    assert_eq!(of("project::uncrawled.adoc"), "ok", "answered live");
    assert_eq!(of("project::nowhere.adoc"), "broken_url");
    let targets = second.targets(second.run, &base);
    let by = |path: &str| targets.iter().find(|t| t.0.ends_with(path)).unwrap();
    assert_eq!(by("/project/objectives/").3, "baseline");
    assert_eq!(by("/releases/spins/").3, "baseline");
    assert_eq!(by("/project/uncrawled/").3, "live");
    assert_eq!(
        asked(&production).await,
        2,
        "the two URLs the baseline lacks"
    );
}
