use std::time::Instant as StdInstant;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::super::{HttpFetcher, Settings};
use super::*;
use crate::fetch::{Fetcher, Mode};

fn settings(per_second: Option<f64>) -> HostSettings {
    HostSettings {
        per_second,
        ..HostSettings::default()
    }
}

fn secs(n: f64) -> Duration {
    Duration::from_secs_f64(n)
}

#[test]
fn with_nothing_written_the_default_rate_applies() {
    let got = interval(&settings(None), 2.0, true, None);
    assert_eq!(got, secs(0.5));
}

#[test]
fn a_crawl_delay_governs_and_the_default_rate_never_beats_it() {
    // Slower than the default, and faster than it: both are the delay.
    assert_eq!(interval(&settings(None), 2.0, false, Some(5.0)), secs(5.0));
    assert_eq!(
        interval(&settings(None), 2.0, true, Some(0.125)),
        secs(0.125)
    );
}

#[test]
fn a_written_rate_beats_the_crawl_delay_on_a_root_host_only() {
    let fast = settings(Some(10.0));
    assert_eq!(interval(&fast, 2.0, true, Some(5.0)), secs(0.1));
    // Elsewhere the slower of the two wins, whichever it is.
    assert_eq!(interval(&fast, 2.0, false, Some(5.0)), secs(5.0));
    let slow = settings(Some(0.25));
    assert_eq!(interval(&slow, 2.0, false, Some(1.0)), secs(4.0));
    assert_eq!(interval(&slow, 2.0, true, Some(1.0)), secs(4.0));
}

#[test]
fn a_crawl_delay_that_is_not_a_delay_is_ignored_and_a_huge_one_is_capped() {
    for bad in [f32::NAN, f32::INFINITY, -1.0] {
        assert_eq!(interval(&settings(None), 2.0, false, Some(bad)), secs(0.5));
    }
    assert_eq!(
        interval(&settings(None), 2.0, false, Some(f32::MAX)),
        MAX_INTERVAL
    );
}

fn config(server: &MockServer, extra: &str) -> Config {
    Config::parse(&format!(
        "user_agent = \"Probe/9\"\n{extra}\n[[root]]\nurl = \"{}/\"\n",
        server.uri()
    ))
    .unwrap()
}

fn fetcher(config: &Config) -> HttpFetcher {
    let settings = Settings {
        user_agent: config.user_agent.clone(),
        backoff: vec![Duration::from_millis(5); 3],
        ..Settings::default()
    };
    HttpFetcher::new(settings, Hosts::new(config)).unwrap()
}

fn url_of(server: &MockServer, path: &str) -> Url {
    Url::parse(&format!("{}{path}", server.uri())).unwrap()
}

async fn robots(server: &MockServer, status: u16, body: &str) {
    Mock::given(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(status).set_body_string(body))
        .mount(server)
        .await;
}

async fn count(server: &MockServer, at: &str) -> usize {
    let all = server.received_requests().await.unwrap();
    all.iter().filter(|r| r.url.path() == at).count()
}

#[tokio::test]
async fn disallowed_urls_are_told_apart_from_allowed_ones() {
    let server = MockServer::start().await;
    robots(
        &server,
        200,
        "User-agent: *\nDisallow: /everything\n\nUser-agent: probe\nDisallow: /private/\n",
    )
    .await;
    let f = fetcher(&config(&server, "[rate]\nper_second = 1000"));
    // The group naming the token replaces `*`'s, whatever the UA's version.
    assert_eq!(
        f.access(&url_of(&server, "/private/x")).await,
        Access::Disallowed
    );
    assert_eq!(
        f.access(&url_of(&server, "/everything")).await,
        Access::Allowed
    );
    assert_eq!(f.access(&url_of(&server, "/public")).await, Access::Allowed);
    assert_eq!(count(&server, "/robots.txt").await, 1, "read once");
}

#[tokio::test]
async fn a_4xx_robots_txt_allows_everything() {
    let server = MockServer::start().await;
    robots(&server, 404, "").await;
    let f = fetcher(&config(&server, "[rate]\nper_second = 1000"));
    assert_eq!(f.access(&url_of(&server, "/a")).await, Access::Allowed);
    assert_eq!(
        count(&server, "/robots.txt").await,
        1,
        "a 404 is not retried"
    );
}

#[tokio::test]
async fn a_5xx_robots_txt_closes_the_host_after_the_retries() {
    let server = MockServer::start().await;
    robots(&server, 503, "").await;
    let f = fetcher(&config(&server, "[rate]\nper_second = 1000"));
    assert_eq!(f.access(&url_of(&server, "/a")).await, Access::Unavailable);
    assert_eq!(f.access(&url_of(&server, "/b")).await, Access::Unavailable);
    assert_eq!(
        count(&server, "/robots.txt").await,
        4,
        "one try and three retries, once"
    );
}

#[tokio::test]
async fn a_429_robots_txt_does_not_open_the_host() {
    let server = MockServer::start().await;
    robots(&server, 429, "").await;
    let f = fetcher(&config(&server, "[rate]\nper_second = 1000"));
    assert_eq!(f.access(&url_of(&server, "/a")).await, Access::Unavailable);
}

#[tokio::test]
async fn no_answer_closes_the_host() {
    // A port nothing listens on: the listener is dropped at once.
    let port = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let config = Config::parse(&format!(
        "[rate]\nper_second = 1000\n[[root]]\nurl = \"http://127.0.0.1:{port}/\"\n"
    ))
    .unwrap();
    let url = Url::parse(&format!("http://127.0.0.1:{port}/a")).unwrap();
    assert_eq!(fetcher(&config).access(&url).await, Access::Unavailable);
}

#[tokio::test]
async fn robots_txt_redirects_are_followed_to_a_limit() {
    let server = MockServer::start().await;
    Mock::given(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(301).insert_header("location", "/moved.txt"))
        .mount(&server)
        .await;
    Mock::given(path("/moved.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string("User-agent: *\nDisallow: /x\n"))
        .mount(&server)
        .await;
    let f = fetcher(&config(&server, "[rate]\nper_second = 1000"));
    assert_eq!(f.access(&url_of(&server, "/x")).await, Access::Disallowed);

    let looping = MockServer::start().await;
    Mock::given(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/robots.txt"))
        .mount(&looping)
        .await;
    let f = fetcher(&config(&looping, "[rate]\nper_second = 1000"));
    assert_eq!(f.access(&url_of(&looping, "/x")).await, Access::Unavailable);
    assert_eq!(
        count(&looping, "/robots.txt").await,
        6,
        "the first and five more"
    );
}

#[tokio::test]
async fn robots_false_reads_no_robots_txt() {
    let server = MockServer::start().await;
    robots(
        &server,
        200,
        "User-agent: *\nDisallow: /\nCrawl-delay: 30\n",
    )
    .await;
    let config = config(
        &server,
        "[host.\"127.0.0.1\"]\nrobots = false\nper_second = 1000",
    );
    let f = fetcher(&config);
    let started = StdInstant::now();
    assert_eq!(f.access(&url_of(&server, "/a")).await, Access::Allowed);
    f.fetch(&url_of(&server, "/a"), Mode::Asset).await.unwrap();
    f.fetch(&url_of(&server, "/b"), Mode::Asset).await.unwrap();
    assert_eq!(count(&server, "/robots.txt").await, 0);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "its Crawl-delay was not read"
    );
}

#[tokio::test]
async fn requests_to_a_host_are_spaced_by_its_rate() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    // 10 a second: five requests, four gaps of 100 ms.
    let f = fetcher(&config(&server, "[rate]\nper_second = 10"));
    let started = StdInstant::now();
    for i in 0..5 {
        f.fetch(&url_of(&server, &format!("/p{i}")), Mode::Page)
            .await
            .unwrap();
    }
    assert!(
        started.elapsed() >= Duration::from_millis(400),
        "{:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn a_crawl_delay_sets_the_pace_once_robots_txt_is_in() {
    let server = MockServer::start().await;
    robots(&server, 200, "User-agent: *\nCrawl-delay: 1\n").await;
    Mock::given(path("/a"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    // The robots.txt fetch is paced at the default rate (the 1000 a second
    // here); the page after it waits the Crawl-delay.
    let f = fetcher(&config(&server, "[rate]\nper_second = 1000"));
    let url = url_of(&server, "/a");
    assert_eq!(f.access(&url).await, Access::Allowed);
    let started = StdInstant::now();
    f.fetch(&url, Mode::Page).await.unwrap();
    f.fetch(&url, Mode::Page).await.unwrap();
    assert!(
        started.elapsed() >= Duration::from_millis(900),
        "{:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn concurrency_caps_the_requests_in_flight_to_a_host() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(200)))
        .mount(&server)
        .await;
    let timed = |extra: &'static str| {
        let server = &server;
        async move {
            let f = fetcher(&config(server, extra));
            let started = StdInstant::now();
            let mut tasks = Vec::new();
            for i in 0..4 {
                let (f, url) = (f.clone(), url_of(server, &format!("/p{i}")));
                tasks.push(tokio::spawn(async move { f.fetch(&url, Mode::Page).await }));
            }
            for t in tasks {
                t.await.unwrap().unwrap();
            }
            started.elapsed()
        }
    };
    let one = timed("[rate]\nper_second = 1000\nconcurrency = 1").await;
    let four = timed("[rate]\nper_second = 1000\nconcurrency = 4").await;
    assert!(one >= Duration::from_millis(800), "{one:?}");
    assert!(four < Duration::from_millis(600), "{four:?}");
    // A `[host]` number beats the default one.
    let host =
        timed("[rate]\nper_second = 1000\nconcurrency = 4\n[host.\"127.0.0.1\"]\nconcurrency = 1")
            .await;
    assert!(host >= Duration::from_millis(800), "{host:?}");
}
