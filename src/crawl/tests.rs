use std::time::Duration;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::fetch::http::host::Hosts;
use crate::fetch::http::{HttpFetcher, Settings};
use crate::profile::antora::Antora;
use crate::profile::generic::Generic;
use crate::store::{Mode, Store, Writer};

use super::*;

fn registry() -> Registry {
    Registry::new(Box::new(Generic)).with(Box::new(Antora))
}

/// Settings that fail fast: retries a few milliseconds apart, a short
/// timeout.
fn quick() -> Settings {
    Settings {
        connect_timeout: Duration::from_secs(2),
        timeout: Duration::from_secs(2),
        backoff: vec![Duration::from_millis(5); 3],
        ..Settings::default()
    }
}

/// A finished run: its database, and the server's address to strip from URLs.
struct Done {
    store: Store,
    summary: Summary,
    base: String,
}

/// Crawls the root `/docs/` of `server`, `extra` being top-level keys of the
/// configuration.
async fn crawl(server: &MockServer, extra: &str, settings: Settings) -> Done {
    let config = Config::parse(&format!(
        "{extra}\n[[root]]\nurl = \"{}/docs/\"\n",
        server.uri()
    ))
    .unwrap();
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "test", None).unwrap();
    let writer = Writer::start(store, 16);
    let fetcher = HttpFetcher::new(settings, Hosts::unlimited()).unwrap();
    let summary = Crawl::new(fetcher, config, registry(), run, writer.handle())
        .run()
        .await
        .unwrap();
    Done {
        store: writer.finish().unwrap(),
        summary,
        base: server.uri(),
    }
}

fn page(server: &MockServer, at: &str, body: &str) -> impl Future<Output = ()> {
    let at = at.to_owned();
    let body = body.to_owned();
    async move {
        Mock::given(path(at))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body.into_bytes(), "text/html"))
            .mount(server)
            .await;
    }
}

async fn status(server: &MockServer, at: &str, code: u16) {
    Mock::given(path(at))
        .respond_with(ResponseTemplate::new(code))
        .mount(server)
        .await;
}

async fn redirect(server: &MockServer, from: &str, code: u16, to: &str) {
    Mock::given(path(from))
        .respond_with(ResponseTemplate::new(code).insert_header("location", to))
        .mount(server)
        .await;
}

fn rows<T>(store: &Store, sql: &str, map: impl Fn(&duckdb::Row) -> duckdb::Result<T>) -> Vec<T> {
    let mut stmt = store.connection().prepare(sql).unwrap();
    stmt.query_map([], |r| map(r))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn count(store: &Store, sql: &str) -> i64 {
    store.connection().query_row(sql, [], |r| r.get(0)).unwrap()
}

/// One target: its URL without the server's address, outcome, reason and
/// status.
/// A page's URL, title, content type, size, profile, `via_links` and hash length.
type PageFacts = (
    String,
    Option<String>,
    Option<String>,
    Option<u64>,
    String,
    bool,
    usize,
);

type Target = (String, String, Option<String>, Option<u16>);

impl Done {
    fn targets(&self) -> Vec<Target> {
        let mut found: Vec<Target> = rows(
            &self.store,
            "SELECT url, outcome, reason, status FROM targets ORDER BY url",
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        );
        for t in &mut found {
            t.0 = t.0.replace(&self.base, "");
        }
        found
    }

    fn target(&self, url: &str) -> Target {
        self.targets()
            .into_iter()
            .find(|t| t.0 == url)
            .unwrap_or_else(|| panic!("no target {url}"))
    }

    /// A target's redirect chain as `(url, status, refresh)`, empty where it
    /// has none.
    fn chain(&self, url: &str) -> Vec<(String, u16, bool)> {
        let conn = self.store.connection();
        let len: Option<usize> = conn
            .query_row(
                "SELECT len(redirect_chain) FROM targets WHERE url = ?",
                [url],
                |r| r.get(0),
            )
            .unwrap();
        (1..=len.unwrap_or(0))
            .map(|i| {
                conn.query_row(
                    &format!(
                        "SELECT redirect_chain[{i}].url, redirect_chain[{i}].status, \
                         redirect_chain[{i}].refresh FROM targets WHERE url = ?"
                    ),
                    [url],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .unwrap()
            })
            .collect()
    }

    /// The pages' URLs, without the server's address.
    fn pages(&self) -> Vec<String> {
        rows(
            &self.store,
            "SELECT t.url FROM pages p JOIN targets t USING (run_id, url_id) ORDER BY t.url",
            |r| r.get::<_, String>(0),
        )
        .into_iter()
        .map(|u| u.replace(&self.base, ""))
        .collect()
    }
}

fn t(url: &str, outcome: &str, reason: Option<&str>, status: Option<u16>) -> Target {
    (
        url.to_owned(),
        outcome.to_owned(),
        reason.map(str::to_owned),
        status,
    )
}

async fn requests(server: &MockServer) -> Vec<(String, String)> {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .map(|r| (r.method.to_string(), r.url.path().to_owned()))
        .collect()
}

#[tokio::test]
async fn every_target_gets_its_outcome_and_every_link_a_row() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        r##"<title>Home</title>
        <a href="/docs/ok">ok</a><a href="/docs/ok#sec">sec</a><a href="ok">again</a>
        <a href="/docs/missing">m</a><a href="/docs/denied">d</a><a href="/docs/broken">b</a>
        <a href="/fr/docs/">off-locale</a><a href="mailto:me@example.org">mail</a>
        <a href="http://other.invalid/x">elsewhere</a><a href="http://[bad">bad</a>"##,
    )
    .await;
    page(
        &server,
        "/docs/ok",
        r#"<title>Ok</title><h2 id="sec">s</h2>"#,
    )
    .await;
    status(&server, "/docs/missing", 404).await;
    status(&server, "/docs/denied", 403).await;
    status(&server, "/docs/broken", 500).await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(
        done.targets(),
        [
            t("/docs/", "ok", None, Some(200)),
            t("/docs/broken", "error", None, Some(500)),
            t("/docs/denied", "error", None, Some(403)),
            t("/docs/missing", "broken_url", None, Some(404)),
            t("/docs/ok", "ok", None, Some(200)),
            t("/fr/docs/", "unchecked", Some("out_of_scope"), None),
            t("http://[bad", "broken_url", Some("invalid_url"), None),
            t(
                "http://other.invalid/x",
                "unchecked",
                Some("host_not_allowlisted"),
                None
            ),
            t("mailto:me@example.org", "unchecked", Some("scheme"), None),
        ]
    );
    assert_eq!(
        done.summary,
        Summary {
            targets: 9,
            pages: 2
        }
    );
    assert_eq!(done.pages(), ["/docs/", "/docs/ok"]);
    assert_eq!(
        count(&done.store, "SELECT count(*) FROM links"),
        10,
        "one row per occurrence, the repeats included"
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM links WHERE fragment = 'sec' AND raw_href = '/docs/ok#sec' \
             AND kind = 'anchor' AND outcome IS NULL AND NOT redirected"
        ),
        1
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM anchors a JOIN targets t USING (run_id, url_id) \
             WHERE t.url LIKE '%/docs/ok' AND a.anchor = 'sec' AND a.placement = 'unknown'"
        ),
        1
    );
    // Checked once, whatever links to it.
    let hits = requests(&server).await;
    assert_eq!(hits.iter().filter(|(_, p)| p == "/docs/ok").count(), 1);
    assert!(hits.iter().all(|(_, p)| p != "/fr/docs/"));
}

#[tokio::test]
async fn a_page_row_holds_what_was_read_and_which_way_it_was_found() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        r#"<title>Home</title><a href="/docs/two">two</a>"#,
    )
    .await;
    page(&server, "/docs/two", "<title>  Two\n pages </title>").await;

    let done = crawl(&server, "", quick()).await;

    let got: Vec<PageFacts> = rows(
        &done.store,
        "SELECT t.url, p.title, p.content_type, p.size, p.profile, p.via_links, \
             octet_length(p.content_hash) \
             FROM pages p JOIN targets t USING (run_id, url_id) ORDER BY t.url",
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        },
    );
    let base = &done.base;
    assert_eq!(
        got,
        [
            (
                format!("{base}/docs/"),
                Some("Home".into()),
                Some("text/html".into()),
                Some(46),
                "generic".into(),
                false,
                16
            ),
            (
                format!("{base}/docs/two"),
                Some("Two pages".into()),
                Some("text/html".into()),
                Some(28),
                "generic".into(),
                true,
                16
            ),
        ]
    );
}

#[tokio::test]
async fn an_allowlisted_host_is_one_hop_deep() {
    let server = MockServer::start().await;
    let outer = MockServer::start().await;
    let port = outer.address().port();
    page(
        &server,
        "/docs/",
        &format!(
            r#"<a href="http://localhost:{port}/page#x">p</a>
            <img src="http://localhost:{port}/pic.png">
            <img src="http://localhost:{port}/denied">"#
        ),
    )
    .await;
    page(
        &outer,
        "/page",
        r#"<title>Outer</title><h2 id="x">x</h2><a href="/deeper">more</a>"#,
    )
    .await;
    Mock::given(method("HEAD"))
        .and(path("/pic.png"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "image/png"))
        .mount(&outer)
        .await;
    Mock::given(method("HEAD"))
        .and(path("/denied"))
        .respond_with(ResponseTemplate::new(405))
        .mount(&outer)
        .await;
    status(&outer, "/denied", 200).await;

    let done = crawl(&server, "allowlist = [\"localhost\"]", quick()).await;

    let outer_targets: Vec<Target> = done
        .targets()
        .into_iter()
        .filter(|t| t.0.contains("localhost"))
        .map(|mut t| {
            t.0 = t.0.replace(&format!("http://localhost:{port}"), "");
            t
        })
        .collect();
    assert_eq!(
        outer_targets,
        [
            t("/denied", "ok", None, Some(200)),
            t("/page", "ok", None, Some(200)),
            t("/pic.png", "ok", None, Some(200)),
        ],
        "a refused HEAD is asked again as a GET (D13)"
    );
    let mut asked: Vec<_> = requests(&outer)
        .await
        .into_iter()
        .filter(|(_, p)| p != "/robots.txt")
        .collect();
    asked.sort();
    let get = |p: &str| ("GET".to_owned(), p.to_owned());
    let head = |p: &str| ("HEAD".to_owned(), p.to_owned());
    assert_eq!(
        asked,
        [
            get("/denied"),
            get("/page"),
            head("/denied"),
            head("/pic.png")
        ],
        "the page is read once and /deeper never"
    );

    // The page's title, hash and anchors are kept; its links and facts are not.
    assert!(
        done.pages()
            .contains(&format!("http://localhost:{port}/page"))
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM anchors a JOIN targets t USING (run_id, url_id) \
             WHERE t.url LIKE '%/page' AND a.anchor = 'x'"
        ),
        1
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM links l JOIN targets t ON t.url_id = l.src_url_id \
             WHERE t.url LIKE '%/page'"
        ),
        0
    );
}

#[tokio::test]
async fn a_redirect_is_a_chain_and_its_landing_page_is_recorded_once() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        r#"<a href="/docs/old">o</a><a href="/docs/lone">l</a><a href="/docs/new">n</a>
        <a href="/docs/leave">x</a>"#,
    )
    .await;
    redirect(&server, "/docs/old", 301, "/docs/new").await;
    page(&server, "/docs/new", "<title>New</title><h2 id=a>a</h2>").await;
    redirect(&server, "/docs/lone", 302, "/docs/landed#frag").await;
    page(&server, "/docs/landed", "<title>Landed</title>").await;
    redirect(&server, "/docs/leave", 307, "/fr/away").await;
    page(&server, "/fr/away", "<title>Away</title>").await;

    let done = crawl(&server, "", quick()).await;

    let base = &done.base;
    for (url, final_url, hops) in [
        ("/docs/old", "/docs/new", vec![("/docs/old", 301)]),
        ("/docs/lone", "/docs/landed", vec![("/docs/lone", 302)]),
        ("/docs/leave", "/fr/away", vec![("/docs/leave", 307)]),
    ] {
        let (status, final_got): (Option<u16>, Option<String>) = done
            .store
            .connection()
            .query_row(
                "SELECT status, final_url FROM targets WHERE url = ?",
                [format!("{base}{url}")],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        let chain = done.chain(&format!("{base}{url}"));
        assert_eq!(status, Some(200), "{url}");
        assert_eq!(final_got, Some(format!("{base}{final_url}")), "{url}");
        assert_eq!(
            chain,
            hops.iter()
                .map(|(u, s)| (format!("{base}{u}"), *s, false))
                .collect::<Vec<_>>(),
            "{url}"
        );
    }
    // Each landing page is a target and a page once, wherever it was met first.
    for landed in ["/docs/new", "/docs/landed", "/fr/away"] {
        assert_eq!(
            count(
                &done.store,
                &format!("SELECT count(*) FROM targets WHERE url = '{base}{landed}'")
            ),
            1,
            "{landed}"
        );
    }
    assert_eq!(
        done.pages(),
        ["/docs/", "/docs/landed", "/docs/new"],
        "a hop out of every scope is asked for its status and read no further"
    );
    assert_eq!(
        done.target("/docs/new"),
        t("/docs/new", "ok", None, Some(200))
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE redirect_chain IS NULL"
        ),
        7 - 3
    );
}

#[tokio::test]
async fn a_refresh_of_delay_zero_is_a_hop_and_one_that_waits_is_a_page() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        r#"<a href="/docs/alias#frag">a</a><a href="/docs/slow">s</a>"#,
    )
    .await;
    page(
        &server,
        "/docs/alias",
        r#"<title>Redirect Notice</title><meta http-equiv="refresh" content="0; url=real"><a href="/docs/only-here">x</a>"#,
    )
    .await;
    page(
        &server,
        "/docs/real",
        r#"<title>Real</title><h2 id="frag">f</h2>"#,
    )
    .await;
    page(
        &server,
        "/docs/slow",
        r#"<title>Slow</title><meta http-equiv="refresh" content="5; url=/docs/real">"#,
    )
    .await;

    let done = crawl(&server, "", quick()).await;

    let base = &done.base;
    assert_eq!(
        done.chain(&format!("{base}/docs/alias")),
        [(format!("{base}/docs/alias"), 200, true)]
    );
    assert_eq!(done.chain(&format!("{base}/docs/slow")), []);
    assert_eq!(
        done.pages(),
        ["/docs/", "/docs/real", "/docs/slow"],
        "an alias is a hop, not a page, and its own links are not followed"
    );
    // The link names the alias; its fragment is the final page's to answer.
    let (target, fragment): (String, Option<String>) = done
        .store
        .connection()
        .query_row(
            "SELECT t.url, l.fragment FROM links l \
             JOIN targets t ON t.url_id = l.target_url_id WHERE l.raw_href = '/docs/alias#frag'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(target, format!("{base}/docs/alias"));
    assert_eq!(fragment.as_deref(), Some("frag"));
    assert_eq!(
        count(
            &done.store,
            &format!(
                "SELECT count(*) FROM targets WHERE url = '{base}/docs/alias' \
                 AND final_url = '{base}/docs/real' AND status = 200 AND outcome = 'ok'"
            )
        ),
        1
    );
    assert!(
        requests(&server)
            .await
            .iter()
            .all(|(_, p)| p != "/docs/only-here")
    );
}

#[tokio::test]
async fn a_loop_and_a_chain_past_ten_hops_are_broken() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        r#"<a href="/docs/a">a</a><a href="/docs/h0">h</a>"#,
    )
    .await;
    redirect(&server, "/docs/a", 302, "/docs/b").await;
    redirect(&server, "/docs/b", 302, "/docs/a").await;
    for i in 0..12 {
        redirect(
            &server,
            &format!("/docs/h{i}"),
            301,
            &format!("/docs/h{}", i + 1),
        )
        .await;
    }

    let done = crawl(&server, "", quick()).await;

    let base = &done.base;
    assert_eq!(
        done.target("/docs/a"),
        t("/docs/a", "broken_url", Some("redirect_loop"), None)
    );
    assert_eq!(done.chain(&format!("{base}/docs/a")).len(), 2);
    assert_eq!(
        done.target("/docs/h0"),
        t("/docs/h0", "broken_url", Some("too_many_redirects"), None)
    );
    assert_eq!(done.chain(&format!("{base}/docs/h0")).len(), 10);
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE final_url IS NOT NULL"
        ),
        0,
        "a chain that never lands names no final URL"
    );
}

#[tokio::test]
async fn a_page_over_the_cap_is_ok_and_is_not_extracted() {
    let server = MockServer::start().await;
    page(&server, "/docs/", r#"<a href="/docs/big#x">b</a>"#).await;
    page(
        &server,
        "/docs/big",
        &format!("<h2 id=x>x</h2>{}", "y".repeat(2000)),
    )
    .await;
    let settings = Settings {
        max_body: 1000,
        ..quick()
    };

    let done = crawl(&server, "", settings).await;

    assert_eq!(
        done.target("/docs/big"),
        t("/docs/big", "ok", Some("too_large"), Some(200))
    );
    assert_eq!(done.pages(), ["/docs/"]);
}

#[tokio::test]
async fn robots_txt_keeps_a_request_from_being_made() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        r#"<a href="/docs/private/x">p</a><a href="/docs/ok">o</a>"#,
    )
    .await;
    page(&server, "/docs/ok", "<title>ok</title>").await;
    page(
        &server,
        "/robots.txt",
        "User-agent: *\nDisallow: /docs/private\n",
    )
    .await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(
        done.target("/docs/private/x"),
        t("/docs/private/x", "unchecked", Some("robots"), None)
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE reason = 'robots' AND checked_at IS NULL \
             AND duration_ms IS NULL"
        ),
        1
    );
    assert!(
        requests(&server)
            .await
            .iter()
            .all(|(_, p)| p != "/docs/private/x")
    );
    assert_eq!(done.pages(), ["/docs/", "/docs/ok"]);
}

#[tokio::test]
async fn a_robots_txt_that_cannot_be_read_closes_the_host() {
    let server = MockServer::start().await;
    page(&server, "/docs/", r#"<a href="/docs/ok">o</a>"#).await;
    status(&server, "/robots.txt", 503).await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(
        done.targets(),
        [t("/docs/", "unchecked", Some("robots_unavailable"), None)]
    );
    assert_eq!(done.pages(), Vec::<String>::new());
}

#[tokio::test]
async fn a_failure_with_no_response_is_classified_by_its_cause() {
    // A port nobody answers on: bind one and let it go.
    let dead = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let server = MockServer::start().await;
    let outer = MockServer::start().await;
    let port = outer.address().port();
    page(
        &server,
        "/docs/",
        &format!(r#"<img src="http://localhost:{port}/x.png">"#),
    )
    .await;
    // The hop leaves the port the host's robots.txt was read from, and not
    // the host, so it is allowed and then refused.
    redirect(
        &outer,
        "/x.png",
        301,
        &format!("http://localhost:{dead}/y.png"),
    )
    .await;

    let done = crawl(&server, "allowlist = [\"localhost\"]", quick()).await;

    let from = format!("http://localhost:{port}/x.png");
    let to = format!("http://localhost:{dead}/y.png");
    for url in [&from, &to] {
        let got = done.target(url);
        assert_eq!(
            (got.1.as_str(), got.2.as_deref(), got.3),
            ("error", Some("connect"), None),
            "{url}"
        );
    }
    assert_eq!(done.chain(&from), [(from.clone(), 301, false)]);
    assert_eq!(
        count(
            &done.store,
            &format!("SELECT count(*) FROM targets WHERE url = '{from}' AND final_url = '{to}'")
        ),
        1,
        "the landing URL has a row of its own"
    );
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE checked_at IS NOT NULL"
        ),
        3,
        "a request was made for each, so the time is kept"
    );
}

#[tokio::test]
async fn a_saved_antora_page_yields_its_profile_facts_source_and_links() {
    let server = MockServer::start().await;
    page(
        &server,
        "/docs/",
        include_str!("../../fixtures/docs-fp-o/en-US/a11y/index.html"),
    )
    .await;

    let done = crawl(&server, "", quick()).await;

    let (profile, repo, source_ref, source_path, source_url): (
        String,
        String,
        String,
        String,
        String,
    ) = done
        .store
        .connection()
        .query_row(
            "SELECT profile, source_repo, source_ref, source_path, source_url FROM pages",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap();
    assert_eq!(profile, "antora");
    assert_eq!(
        repo,
        "https://forge.fedoraproject.org/dei/fedora-accessibility"
    );
    assert_eq!(source_ref, "main");
    assert_eq!(source_path, "docs/modules/ROOT/pages/index.adoc");
    assert!(source_url.ends_with("/src/branch/main/docs/modules/ROOT/pages/index.adoc"));
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM page_facts WHERE key = 'antora.component' AND value = 'a11y'"
        ),
        1
    );
    assert!(count(&done.store, "SELECT count(*) FROM links") > 100);
    assert!(
        count(
            &done.store,
            "SELECT count(*) FROM anchors WHERE placement = 'content'"
        ) > 0,
        "the profile's region decides placement (D34)"
    );
    assert!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE reason = 'host_not_allowlisted'"
        ) > 10
    );
    assert_eq!(
        requests(&server)
            .await
            .iter()
            .filter(|(_, p)| p != "/robots.txt")
            .count(),
        1,
        "everything else the page names is out of scope or unlisted"
    );
}
