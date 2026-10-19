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
    let config = format!("{extra}\n[[root]]\nurl = \"{}/docs/\"\n", server.uri());
    crawl_config(server, &config, settings).await
}

/// Crawls `server` under the configuration `config`.
async fn crawl_config(server: &MockServer, config: &str, settings: Settings) -> Done {
    let config = Config::parse(config).unwrap();
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store
        .begin_run(Mode::Crawl, "test", None, &config.root_urls())
        .unwrap();
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

/// One target: its URL without the server's address, outcome, reason and
/// status.
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
            pages: 2,
            sitemaps: 0,
            dropped: 0,
            stopped: false,
            cut: 0,
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
             AND kind = 'anchor'"
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
async fn a_skipped_url_is_never_asked_and_its_link_is_kept() {
    let server = MockServer::start().await;
    let uri = server.uri();
    page(
        &server,
        "/docs/",
        r#"<a href="old/">old</a><a href="kept/">kept</a>
        <a href="/elsewhere/issues/new?title=x">new issue</a>"#,
    )
    .await;
    page(&server, "/docs/kept/", "<title>Kept</title>").await;
    page(&server, "/docs/old/", "<title>Old</title>").await;
    let skip = format!("skip = [\"{uri}/docs/old/**\", \"{uri}/elsewhere/**\"]");
    let done = crawl(&server, &skip, quick()).await;

    assert_eq!(
        done.targets(),
        [
            t("/docs/", "ok", None, Some(200)),
            t("/docs/kept/", "ok", None, Some(200)),
            t("/docs/old/", "unchecked", Some("skipped"), None),
            t(
                "/elsewhere/issues/new?title=x",
                "unchecked",
                Some("skipped"),
                None
            ),
        ],
        "in a root's scope or outside it, a skipped URL is a target and no request"
    );
    let asked: Vec<_> = requests(&server)
        .await
        .into_iter()
        .map(|(_, p)| p)
        .collect();
    assert!(
        !asked
            .iter()
            .any(|p| p.starts_with("/docs/old") || p.starts_with("/elsewhere")),
        "{asked:?}"
    );
    assert_eq!(
        count(&done.store, "SELECT count(*) FROM links"),
        3,
        "the links to them are kept"
    );
}

#[tokio::test]
async fn a_redirect_onto_a_skipped_url_ends_the_chain() {
    let server = MockServer::start().await;
    let base = server.uri();
    page(
        &server,
        "/docs/",
        r#"<a href="/docs/go">go</a><a href="/docs/twice">twice</a><a href="/docs/ok">ok</a>"#,
    )
    .await;
    redirect(&server, "/docs/go", 303, "/user/login?next=a").await;
    redirect(&server, "/docs/twice", 301, "/docs/go").await;
    page(&server, "/docs/ok", "<title>Ok</title>").await;
    page(&server, "/user/login", "<title>Login</title>").await;
    // A sitemap moved onto a skipped URL lists nothing, whatever is there.
    redirect(&server, "/docs/set.xml", 301, "/user/login/map.xml").await;
    xml(
        &server,
        "/user/login/map.xml",
        &urlset(&base, &["/docs/in-map"]),
    )
    .await;
    page(&server, "/docs/in-map", "<title>In map</title>").await;
    let config = format!(
        "skip = [\"{base}/user/login*\", \"{base}/user/login/**\"]\n\
         [[root]]\nurl = \"{base}/docs/\"\nsitemap = \"{base}/docs/set.xml\"\n"
    );

    let done = crawl_config(&server, &config, quick()).await;

    let login = format!("{base}/user/login?next=a");
    let ended = |url: &str| t(url, "unchecked", Some("skipped"), None);
    assert_eq!(done.target("/docs/go"), ended("/docs/go"));
    assert_eq!(done.target("/docs/twice"), ended("/docs/twice"));
    assert_eq!(done.target("/docs/set.xml"), ended("/docs/set.xml"));
    assert_eq!(
        done.target("/user/login/map.xml"),
        ended("/user/login/map.xml")
    );
    assert_eq!(
        done.target("/user/login?next=a"),
        ended("/user/login?next=a")
    );
    assert_eq!(
        done.target("/docs/ok"),
        t("/docs/ok", "ok", None, Some(200))
    );
    assert_eq!(
        done.chain(&format!("{base}/docs/twice")),
        [
            (format!("{base}/docs/twice"), 301, false),
            (format!("{base}/docs/go"), 303, false)
        ]
    );
    assert_eq!(
        count(
            &done.store,
            &format!("SELECT count(*) FROM targets WHERE url = '{login}' OR final_url = '{login}'")
        ),
        3,
        "the landing URL has a row of its own, once, and /docs/go and /docs/twice end there"
    );
    assert_eq!(done.pages(), ["/docs/", "/docs/ok"]);
    let asked: Vec<_> = requests(&server)
        .await
        .into_iter()
        .map(|(_, p)| p)
        .collect();
    assert!(
        asked.iter().all(|p| !p.starts_with("/user/login")),
        "{asked:?}"
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
    let root = format!("{}/docs/", server.uri());
    let roots = |sql: &str| count(&done.store, sql);
    assert_eq!(
        roots(&format!(
            "SELECT count(*) FROM pages WHERE root = '{root}' AND list_contains((SELECT roots FROM runs), '{root}')"
        )),
        1,
        "the root's page names its root, the run lists it"
    );
    assert_eq!(
        roots(
            "SELECT count(*) FROM pages p JOIN targets t USING (run_id, url_id) WHERE t.url LIKE '%/page' AND p.root IS NULL"
        ),
        1,
        "an allowlisted page has no root"
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
            .filter(|(_, p)| p != "/robots.txt" && !p.ends_with("/sitemap.xml"))
            .count(),
        1,
        "everything else the page names is out of scope or unlisted"
    );
}

async fn xml(server: &MockServer, at: &str, body: &str) {
    Mock::given(path(at.to_owned()))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.as_bytes().to_vec(), "text/xml"))
        .mount(server)
        .await;
}

fn urlset(base: &str, paths: &[&str]) -> String {
    let urls: String = paths
        .iter()
        .map(|p| {
            let loc = if p.contains(':') {
                p.to_string()
            } else {
                format!("{base}{p}")
            };
            format!("<url><loc>{loc}</loc><lastmod>2026-10-18</lastmod></url>")
        })
        .collect();
    format!(
        r#"<?xml version="1.0"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{urls}</urlset>"#
    )
}

fn index(base: &str, paths: &[&str]) -> String {
    let maps: String = paths
        .iter()
        .map(|p| format!("<sitemap><loc>{base}{p}</loc></sitemap>"))
        .collect();
    format!(
        r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{maps}</sitemapindex>"#
    )
}

impl Done {
    /// The pages' URLs with `in_sitemap` and `via_links`.
    fn found(&self) -> Vec<(String, bool, bool)> {
        rows(
            &self.store,
            "SELECT t.url, p.in_sitemap, p.via_links FROM pages p \
             JOIN targets t USING (run_id, url_id) ORDER BY t.url",
            |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?)),
        )
        .into_iter()
        .map(|(u, s, l)| (u.replace(&self.base, ""), s, l))
        .collect()
    }

    /// The `sitemap` links as (source URL, raw href), the server's address
    /// stripped from both.
    fn sitemap_links(&self) -> Vec<(String, String)> {
        self.links_of("sitemap")
    }

    /// The `sitemap_file` links as (source URL, raw href, target URL), the
    /// server's address stripped from each.
    fn file_links(&self) -> Vec<(String, String, String)> {
        let mut found: Vec<_> = rows(
            &self.store,
            "SELECT s.url, l.raw_href, t.url FROM links l \
             JOIN targets s ON s.run_id = l.run_id AND s.url_id = l.src_url_id \
             JOIN targets t ON t.run_id = l.run_id AND t.url_id = l.target_url_id \
             WHERE l.kind = 'sitemap_file'",
            |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get::<_, String>(2)?)),
        )
        .into_iter()
        .map(|(s, h, t)| {
            let strip = |u: String| u.replace(&self.base, "");
            (strip(s), strip(h), strip(t))
        })
        .collect();
        found.sort();
        found
    }

    fn links_of(&self, kind: &str) -> Vec<(String, String)> {
        let mut found: Vec<_> = rows(
            &self.store,
            &format!(
                "SELECT t.url, l.raw_href FROM links l JOIN targets t \
                 ON t.run_id = l.run_id AND t.url_id = l.src_url_id \
                 WHERE l.kind = '{kind}' ORDER BY ALL"
            ),
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .into_iter()
        .map(|(s, h)| (s.replace(&self.base, ""), h.replace(&self.base, "")))
        .collect();
        found.sort();
        found
    }
}

#[tokio::test]
async fn a_sitemap_index_seeds_its_pages_and_each_entry_is_a_link() {
    let server = MockServer::start().await;
    let base = server.uri();
    Mock::given(path("/robots.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(format!("User-agent: *\nSitemap: {base}/docs/sitemap.xml\n")),
        )
        .mount(&server)
        .await;
    xml(
        &server,
        "/docs/sitemap.xml",
        &index(&base, &["/docs/a.xml", "/docs/b.xml.gz"]),
    )
    .await;
    xml(
        &server,
        "/docs/a.xml",
        &urlset(&base, &["/docs/", "/docs/listed", "/fr/docs/x", "::bad"]),
    )
    .await;
    let mut packed = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(
        &mut packed,
        urlset(&base, &["/docs/gone", "/docs/listed"]).as_bytes(),
    )
    .unwrap();
    Mock::given(path("/docs/b.xml.gz"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(packed.finish().unwrap(), "application/gzip"),
        )
        .mount(&server)
        .await;
    page(
        &server,
        "/docs/",
        r#"<a href="/docs/linked">l</a><a href="/docs/listed">l</a>"#,
    )
    .await;
    page(&server, "/docs/listed", "<title>Listed</title>").await;
    page(&server, "/docs/linked", "<title>Linked</title>").await;
    status(&server, "/docs/gone", 404).await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(
        done.targets(),
        [
            t("::bad", "broken_url", Some("invalid_url"), None),
            t("/docs/", "ok", None, Some(200)),
            t("/docs/a.xml", "ok", None, Some(200)),
            t("/docs/b.xml.gz", "ok", None, Some(200)),
            t("/docs/gone", "broken_url", None, Some(404)),
            t("/docs/linked", "ok", None, Some(200)),
            t("/docs/listed", "ok", None, Some(200)),
            t("/docs/sitemap.xml", "ok", None, Some(200)),
        ],
        "the off-scope entry has no target"
    );
    assert_eq!(
        done.found(),
        [
            ("/docs/".to_owned(), true, false),
            ("/docs/linked".to_owned(), false, true),
            ("/docs/listed".to_owned(), true, false),
        ],
        "a listed page is claimed by its sitemap before any link names it"
    );
    let s = |src: &str, href: &str| (src.to_owned(), href.to_owned());
    assert_eq!(
        done.sitemap_links(),
        [
            s("/docs/a.xml", "/docs/"),
            s("/docs/a.xml", "/docs/listed"),
            s("/docs/a.xml", "::bad"),
            s("/docs/b.xml.gz", "/docs/gone"),
            s("/docs/b.xml.gz", "/docs/listed"),
        ],
        "an index's children are files, not entries"
    );
    let f = |src: &str, href: &str, to: &str| (src.to_owned(), href.to_owned(), to.to_owned());
    assert_eq!(
        done.file_links(),
        [
            f("/docs/", "/docs/sitemap.xml", "/docs/sitemap.xml"),
            f("/docs/sitemap.xml", "/docs/a.xml", "/docs/a.xml"),
            f("/docs/sitemap.xml", "/docs/b.xml.gz", "/docs/b.xml.gz"),
        ],
        "the root links to what robots.txt named, its line the raw href"
    );
    assert_eq!((done.summary.sitemaps, done.summary.dropped), (3, 1));
}

#[tokio::test]
async fn discovery_falls_back_to_the_host_and_a_miss_writes_nothing() {
    let server = MockServer::start().await;
    let base = server.uri();
    // A soft 404: the page answers, and is no sitemap.
    page(&server, "/docs/sitemap.xml", "<title>Not found</title>").await;
    xml(
        &server,
        "/sitemap.xml",
        &urlset(&base, &["/docs/only-listed"]),
    )
    .await;
    page(&server, "/docs/", "<title>Home</title>").await;
    page(&server, "/docs/only-listed", "<title>Orphan</title>").await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(
        done.targets(),
        [
            t("/docs/", "ok", None, Some(200)),
            t("/docs/only-listed", "ok", None, Some(200)),
            t("/sitemap.xml", "ok", None, Some(200)),
        ]
    );
    assert_eq!(
        done.file_links(),
        [(
            "/docs/".to_owned(),
            "/sitemap.xml".to_owned(),
            "/sitemap.xml".to_owned()
        )],
        "the probe that found it is the raw href; the one that missed has no link"
    );
    assert_eq!(done.summary.sitemaps, 1);
}

#[tokio::test]
async fn a_run_with_no_sitemap_follows_links_alone() {
    let server = MockServer::start().await;
    page(&server, "/docs/", r#"<a href="/docs/two">two</a>"#).await;
    page(&server, "/docs/two", "<title>Two</title>").await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(done.pages(), ["/docs/", "/docs/two"]);
    assert_eq!(done.summary.sitemaps, 0);
    assert!(done.sitemap_links().is_empty());
    assert!(done.file_links().is_empty(), "the root has no sitemap");
    let probed: Vec<_> = requests(&server)
        .await
        .into_iter()
        .map(|(_, p)| p)
        .filter(|p| p.ends_with("sitemap.xml"))
        .collect();
    assert_eq!(probed, ["/docs/sitemap.xml", "/sitemap.xml"]);
}

#[tokio::test]
async fn a_set_sitemap_is_a_target_whatever_it_answers() {
    let server = MockServer::start().await;
    page(&server, "/docs/", "<title>Home</title>").await;
    xml(&server, "/docs/sitemap.xml", "<urlset/>").await;
    let config = format!(
        "[[root]]\nurl = \"{0}/docs/\"\nsitemap = \"{0}/docs/missing.xml\"\n",
        server.uri()
    );

    let done = crawl_config(&server, &config, quick()).await;

    assert_eq!(
        done.targets(),
        [
            t("/docs/", "ok", None, Some(200)),
            t("/docs/missing.xml", "broken_url", None, Some(404)),
        ],
        "the root's own sitemap.xml is not probed"
    );
    assert_eq!(
        done.file_links(),
        [(
            "/docs/".to_owned(),
            "/docs/missing.xml".to_owned(),
            "/docs/missing.xml".to_owned()
        )],
        "a broken set sitemap shows as a broken link from its root"
    );
    assert_eq!(done.summary.sitemaps, 0);
}

#[tokio::test]
async fn an_index_is_followed_through_a_redirect_and_three_files_deep() {
    let server = MockServer::start().await;
    let base = server.uri();
    xml(
        &server,
        "/docs/sitemap.xml",
        &index(&base, &["/docs/old.xml"]),
    )
    .await;
    redirect(&server, "/docs/old.xml", 301, "/docs/two.xml").await;
    xml(
        &server,
        "/docs/two.xml",
        &index(&base, &["/docs/three.xml"]),
    )
    .await;
    xml(
        &server,
        "/docs/three.xml",
        &index(&base, &["/docs/four.xml"]),
    )
    .await;
    xml(&server, "/docs/four.xml", &urlset(&base, &["/docs/deep"])).await;
    page(&server, "/docs/", "<title>Home</title>").await;

    let done = crawl(&server, "", quick()).await;

    assert_eq!(
        done.targets(),
        [
            t("/docs/", "ok", None, Some(200)),
            t("/docs/four.xml", "unchecked", Some("too_deep"), None),
            t("/docs/old.xml", "ok", None, Some(200)),
            t("/docs/sitemap.xml", "ok", None, Some(200)),
            t("/docs/three.xml", "ok", None, Some(200)),
            t("/docs/two.xml", "ok", None, Some(200)),
        ]
    );
    assert_eq!(
        done.chain(&format!("{base}/docs/old.xml")),
        [(format!("{base}/docs/old.xml"), 301, false)]
    );
    let f = |src: &str, href: &str, to: &str| (src.to_owned(), href.to_owned(), to.to_owned());
    assert_eq!(
        done.file_links(),
        [
            f("/docs/", "sitemap.xml", "/docs/sitemap.xml"),
            f("/docs/sitemap.xml", "/docs/old.xml", "/docs/old.xml"),
            f("/docs/three.xml", "/docs/four.xml", "/docs/four.xml"),
            f("/docs/two.xml", "/docs/three.xml", "/docs/three.xml"),
        ],
        "a moved index links from where it landed"
    );
    assert!(done.sitemap_links().is_empty());
    assert_eq!(done.summary.sitemaps, 3);
    assert!(
        requests(&server)
            .await
            .iter()
            .all(|(_, p)| p != "/docs/four.xml")
    );
}

#[tokio::test]
async fn each_root_links_once_to_each_sitemap_it_read() {
    let server = MockServer::start().await;
    let base = server.uri();
    // Two roots on one host read the same robots.txt, which names one file
    // twice.
    Mock::given(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "User-agent: *\nSitemap: /map.xml\nSitemap: /map.xml\nSitemap: ftp://x/m.xml\n",
        ))
        .mount(&server)
        .await;
    xml(&server, "/map.xml", &urlset(&base, &["/a/", "/b/"])).await;
    page(&server, "/a/", "<title>A</title>").await;
    page(&server, "/b/", "<title>B</title>").await;
    let config = format!("[[root]]\nurl = \"{base}/a/\"\n[[root]]\nurl = \"{base}/b/\"\n");

    let done = crawl_config(&server, &config, quick()).await;

    let f = |src: &str| (src.to_owned(), "/map.xml".to_owned(), "/map.xml".to_owned());
    assert_eq!(
        done.file_links(),
        [f("/a/"), f("/b/")],
        "a shared file is read once and linked from each root"
    );
    assert_eq!(done.summary.sitemaps, 1);
    assert_eq!(
        requests(&server)
            .await
            .iter()
            .filter(|(_, p)| p == "/map.xml")
            .count(),
        1
    );
}

#[tokio::test]
async fn a_probe_another_root_found_is_linked_from_both() {
    let server = MockServer::start().await;
    let base = server.uri();
    xml(&server, "/sitemap.xml", &urlset(&base, &["/a/"])).await;
    page(&server, "/a/", "<title>A</title>").await;
    page(&server, "/b/", "<title>B</title>").await;
    let config = format!("[[root]]\nurl = \"{base}/a/\"\n[[root]]\nurl = \"{base}/b/\"\n");

    let done = crawl_config(&server, &config, quick()).await;

    let f = |src: &str| {
        (
            src.to_owned(),
            "/sitemap.xml".to_owned(),
            "/sitemap.xml".to_owned(),
        )
    };
    assert_eq!(done.file_links(), [f("/a/"), f("/b/")]);
    assert_eq!(
        requests(&server)
            .await
            .iter()
            .filter(|(_, p)| p == "/sitemap.xml")
            .count(),
        1,
        "the second root does not ask again"
    );
}

/// Crawls the root `/docs/` of `server` with `grace` after a stop, stops
/// once `at` has been requested, and returns the run and how long it went
/// on after the stop.
async fn stop_at(server: &MockServer, at: &str, grace: Duration) -> (Done, Duration) {
    let config = format!("[[root]]\nurl = \"{}/docs/\"\n", server.uri());
    let config = Config::parse(&config).unwrap();
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store
        .begin_run(Mode::Crawl, "test", None, &config.root_urls())
        .unwrap();
    let writer = Writer::start(store, 16);
    let fetcher = HttpFetcher::new(quick(), Hosts::unlimited()).unwrap();
    let crawl = Crawl::new(fetcher, config, registry(), run, writer.handle()).with_grace(grace);
    let stop = crawl.stopper();
    let running = tokio::spawn(crawl.run());
    while !requests(server).await.iter().any(|(_, p)| p == at) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    stop.stop();
    let stopped = Instant::now();
    let summary = running.await.unwrap().unwrap();
    let after = stopped.elapsed();
    assert!(summary.stopped);
    let done = Done {
        store: writer.finish().unwrap(),
        summary,
        base: server.uri(),
    };
    (done, after)
}

fn has_requested(requests: &[(String, String)], at: &str) -> bool {
    requests.iter().any(|(_, p)| p == at)
}

/// A stop lets the request in flight finish and starts nothing after it,
/// counting what it left unwritten as cut.
#[tokio::test]
async fn a_stop_finishes_what_started_and_starts_nothing_more() {
    let server = MockServer::start().await;
    page(&server, "/docs/", r#"<a href="slow">slow</a>"#).await;
    Mock::given(path("/docs/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"<a href="next">next</a>"#, "text/html")
                .set_delay(Duration::from_millis(600)),
        )
        .mount(&server)
        .await;
    page(&server, "/docs/next", "").await;
    let (done, _) = stop_at(&server, "/docs/slow", STOP_GRACE).await;
    assert_eq!(done.pages(), ["/docs/", "/docs/slow"]);
    assert!(!has_requested(&requests(&server).await, "/docs/next"));
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE url LIKE '%/next'"
        ),
        0
    );
    assert_eq!(done.summary.cut, 1, "the link to /next was claimed");
}

/// A redirect answered after the stop is not followed, and its target is
/// cut rather than written half done.
#[tokio::test]
async fn a_stop_takes_no_further_hop() {
    let server = MockServer::start().await;
    page(&server, "/docs/", r#"<a href="hop">hop</a>"#).await;
    Mock::given(path("/docs/hop"))
        .respond_with(
            ResponseTemplate::new(301)
                .insert_header("location", "/docs/there")
                .set_delay(Duration::from_millis(600)),
        )
        .mount(&server)
        .await;
    page(&server, "/docs/there", "").await;
    let (done, _) = stop_at(&server, "/docs/hop", STOP_GRACE).await;
    assert!(!has_requested(&requests(&server).await, "/docs/there"));
    assert_eq!(done.pages(), ["/docs/"]);
    assert_eq!(
        count(
            &done.store,
            "SELECT count(*) FROM targets WHERE url LIKE '%/hop'"
        ),
        0
    );
    assert_eq!(done.summary.cut, 1);
}

/// A target still unanswered when the grace runs out is dropped there, and
/// the run ends then rather than when the answer comes.
#[tokio::test]
async fn a_target_unfinished_at_the_grace_is_cut() {
    let server = MockServer::start().await;
    page(&server, "/docs/", r#"<a href="slow">slow</a>"#).await;
    Mock::given(path("/docs/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw("", "text/html")
                .set_delay(Duration::from_millis(1500)),
        )
        .mount(&server)
        .await;
    let (done, after) = stop_at(&server, "/docs/slow", Duration::from_millis(200)).await;
    assert!(after < Duration::from_millis(1000), "waited {after:?}");
    assert_eq!(done.pages(), ["/docs/"]);
    assert_eq!(
        (done.summary.targets, done.summary.cut),
        (1, 1),
        "/docs/ written, /docs/slow cut"
    );
}

/// A stop before the run asks about nothing, the sitemaps included, and
/// says the run is not whole.
#[tokio::test]
async fn a_stop_before_the_run_asks_about_nothing() {
    let server = MockServer::start().await;
    page(&server, "/docs/", "").await;
    let config = format!("[[root]]\nurl = \"{}/docs/\"\n", server.uri());
    let config = Config::parse(&config).unwrap();
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store
        .begin_run(Mode::Crawl, "test", None, &config.root_urls())
        .unwrap();
    let writer = Writer::start(store, 16);
    let fetcher = HttpFetcher::new(quick(), Hosts::unlimited()).unwrap();
    let crawl = Crawl::new(fetcher, config, registry(), run, writer.handle());
    crawl.stopper().stop();
    let summary = crawl.run().await.unwrap();
    assert!(summary.stopped);
    assert_eq!((summary.targets, summary.pages), (0, 0));
    assert!(requests(&server).await.is_empty());
    writer.finish().unwrap();
}

/// Closing the writes waits for one begun, and begins none after.
#[test]
fn closed_writes_wait_for_those_begun_and_begin_none() {
    let writes = Writes::default();
    let writing = writes.begin().unwrap();
    let closer = std::thread::spawn({
        let writes = writes.clone();
        move || writes.close()
    });
    std::thread::sleep(Duration::from_millis(50));
    assert!(!closer.is_finished(), "close returned with a write begun");
    drop(writing);
    closer.join().unwrap();
    assert!(writes.begin().is_none());
    assert_eq!(writes.writing(), 0);
}

/// A run abandoned while a page is being written, its writes then closed,
/// holds that page whole: every link, not the first few.
#[tokio::test]
async fn an_abandoned_run_holds_the_page_it_was_writing_whole() {
    const LINKS: usize = 20_000;
    let server = MockServer::start().await;
    let body: String = (0..LINKS)
        .map(|i| format!(r#"<a href="p{i}">{i}</a>"#))
        .collect();
    page(&server, "/docs/", &body).await;
    let config = format!("[[root]]\nurl = \"{}/docs/\"\n", server.uri());
    let config = Config::parse(&config).unwrap();
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store
        .begin_run(Mode::Crawl, "test", None, &config.root_urls())
        .unwrap();
    let writer = Writer::start(store, 1);
    let fetcher = HttpFetcher::new(quick(), Hosts::unlimited()).unwrap();
    let crawl = Crawl::new(fetcher, config, registry(), run, writer.handle());
    let writes = crawl.writes();
    // The sitemap probes' links are written before /docs/ is asked about,
    // so a write begun after that request is the page's or a later one's.
    let mid_write = async {
        loop {
            if writes.writing() > 0 && has_requested(&requests(&server).await, "/docs/") {
                return;
            }
            tokio::task::yield_now().await;
        }
    };
    tokio::select! {
        _ = crawl.run() => panic!("the run ended before it was abandoned"),
        () = mid_write => {}
    }
    writes.close();
    let store = writer.finish().unwrap();
    let links = count(
        &store,
        "SELECT count(*) FROM links l JOIN targets t ON t.url_id = l.src_url_id \
         WHERE t.url LIKE '%/docs/'",
    );
    assert_eq!(links, LINKS as i64);
}
