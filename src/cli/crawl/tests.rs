use std::io::Write;
use std::process::ExitCode;

use tempfile::{NamedTempFile, TempDir};
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::config::ProfileChoice;
use crate::profile::antora::Antora;
use crate::profile::generic::Generic;
use crate::store::Store;

use super::*;

fn registry() -> Registry {
    Registry::new(Box::new(Generic)).with(Box::new(Antora))
}

const URL: &str = "https://docs.example.org/en-US/";

fn file(toml: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(toml.as_bytes()).unwrap();
    f
}

fn args(url: Option<&str>, config: Option<&NamedTempFile>) -> Args {
    Args {
        url: url.map(str::to_owned),
        config: config.map(|f| f.path().to_owned()),
        db: None,
    }
}

fn configured(url: Option<&str>, config: Option<&NamedTempFile>) -> Result<Config, String> {
    configure(&args(url, config), &registry()).map(|(c, _)| c)
}

fn root_urls(config: &Config) -> Vec<String> {
    config.roots.iter().map(|r| r.url.to_string()).collect()
}

#[test]
fn a_url_alone_is_the_only_root_at_every_default() {
    let config = configured(Some(URL), None).unwrap();
    assert_eq!(root_urls(&config), [URL]);
    assert_eq!(config.roots[0].profile, ProfileChoice::Auto);
}

#[test]
fn a_file_naming_no_root_takes_the_url_and_keeps_its_settings() {
    let f = file("allowlist = [\"gitlab.com\"]\n");
    let config = configured(Some(URL), Some(&f)).unwrap();
    assert_eq!(root_urls(&config), [URL]);
    assert_eq!(config.allowlist.entries(), ["gitlab.com"]);
}

#[test]
fn a_url_beside_a_file_naming_roots_is_refused() {
    let f = file("[[root]]\nurl = \"https://other.example.org/\"\n");
    let e = configured(Some(URL), Some(&f)).unwrap_err();
    assert!(e.contains("names its own roots"), "{e}");
    let config = configured(None, Some(&f)).unwrap();
    assert_eq!(root_urls(&config), ["https://other.example.org/"]);
}

#[test]
fn no_root_from_either_is_refused() {
    let e = configured(None, None).unwrap_err();
    assert!(e.contains("give a URL"), "{e}");
    let f = file("allowlist = [\"gitlab.com\"]\n");
    let e = configured(None, Some(&f)).unwrap_err();
    assert!(e.contains("names no [[root]]"), "{e}");
}

#[test]
fn the_urls_host_is_checked_for_robots_false() {
    let f = file("[host.\"docs.example.org\"]\nrobots = false\n");
    configured(Some(URL), Some(&f)).unwrap();
    let e = configured(Some("https://other.example.org/"), Some(&f)).unwrap_err();
    assert!(e.contains("no root starts on docs.example.org"), "{e}");
}

#[test]
fn the_urls_host_may_not_be_allowlisted_outright() {
    let f = file("allowlist = [\"docs.example.org\", \"*.example.org\"]\n");
    let e = configured(Some(URL), Some(&f)).unwrap_err();
    assert!(
        e.contains("docs.example.org is a host a root starts on"),
        "{e}"
    );
    let config = configured(Some("https://other.example.org/"), Some(&f)).unwrap();
    assert!(!config.is_allowlisted("other.example.org"));
    assert!(config.is_allowlisted("docs.example.org"));
}

#[test]
fn a_profile_the_registry_lacks_is_refused() {
    let f = file("[[root]]\nurl = \"https://docs.example.org/\"\nprofile = \"hugo\"\n");
    let e = configured(None, Some(&f)).unwrap_err();
    assert_eq!(
        e,
        "root[0].profile: no profile is named `hugo`; the profiles are `auto`, `generic`, `antora`"
    );
    let f = file("[[root]]\nurl = \"https://docs.example.org/\"\nprofile = \"antora\"\n");
    configured(None, Some(&f)).unwrap();
}

#[test]
fn the_config_hash_follows_the_file_and_the_url() {
    let hash = |url: Option<&str>, f: Option<&NamedTempFile>| {
        configure(&args(url, f), &registry()).unwrap().1
    };
    let (a, b) = (
        file("user_agent = \"x/1\"\n"),
        file("user_agent = \"y/1\"\n"),
    );
    assert_eq!(hash(Some(URL), None), hash(Some(URL), None));
    assert_ne!(
        hash(Some(URL), None),
        hash(Some("https://docs.example.org/"), None)
    );
    assert_ne!(hash(Some(URL), Some(&a)), hash(Some(URL), Some(&b)));
    assert_ne!(hash(Some(URL), Some(&a)), hash(Some(URL), None));
    assert_eq!(hash(Some(URL), None).len(), 16);
}

#[test]
fn the_report_says_what_the_run_wrote_and_whether_it_finished() {
    let s = Summary {
        targets: 12,
        pages: 3,
        sitemaps: 0,
        dropped: 0,
        stopped: false,
        cut: 0,
    };
    assert_eq!(
        report(RunId(4), &s),
        "run 4: 12 targets, 3 pages, no sitemap"
    );
    let s = Summary {
        sitemaps: 2,
        dropped: 5,
        stopped: true,
        cut: 7,
        ..s
    };
    assert_eq!(
        report(RunId(4), &s),
        "run 4: 12 targets, 3 pages, 2 sitemap files, 5 sitemap entries outside every root \
         dropped, 7 cut; stopped before the end, so the run is left unfinished"
    );
}

/// `run` from parsed arguments, end to end: a run is added to the file `--db` names,
/// finished, and a second run after it is numbered on.
#[test]
fn a_crawl_writes_a_finished_run_to_the_db() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(path("/docs/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(r#"<title>Docs</title><a href="a">a</a>"#, "text/html"),
            )
            .mount(&server)
            .await;
        Mock::given(path("/docs/a"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("<title>A</title>", "text/html"))
            .mount(&server)
            .await;
        server
    });
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("runs.duckdb");
    let url = format!("{}/docs/", server.uri());
    let args = Args {
        url: Some(url),
        config: None,
        db: Some(db.clone()),
    };
    assert_eq!(run(&args, registry()), ExitCode::SUCCESS);
    assert_eq!(run(&args, registry()), ExitCode::SUCCESS);
    let store = Store::open(&db, &registry()).unwrap();
    let conn = store.connection();
    let (runs, finished): (i64, i64) = conn
        .query_row(
            "SELECT count(*), count(finished_at) FROM runs WHERE mode = 'crawl'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((runs, finished), (2, 2));
    let pages: i64 = conn
        .query_row("SELECT count(*) FROM pages WHERE run_id = 2", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(pages, 2);
}

#[test]
fn a_db_that_cannot_be_opened_fails_before_any_request() {
    let dir = TempDir::new().unwrap();
    let args = Args {
        url: Some("http://127.0.0.1:9/".into()),
        config: None,
        db: Some(dir.path().to_owned()),
    };
    assert_eq!(run(&args, registry()), ExitCode::FAILURE);
}
