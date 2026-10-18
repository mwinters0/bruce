use std::io::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use flate2::Compression;
use flate2::write::GzEncoder;
use hickory_resolver::net::{DnsError, NoRecords};
use hickory_resolver::proto::op::{Query, ResponseCode};
use tokio::net::TcpListener;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

const LATIN1_E_ACUTE: u8 = 0xE9;

/// Settings that fail fast: retries a few milliseconds apart, a short
/// timeout.
fn quick() -> Settings {
    Settings {
        user_agent: "bruce-test/1".to_owned(),
        connect_timeout: Duration::from_secs(2),
        timeout: Duration::from_secs(2),
        backoff: vec![Duration::from_millis(5); 3],
        ..Settings::default()
    }
}

fn fetcher(settings: Settings) -> HttpFetcher {
    HttpFetcher::new(settings, Hosts::unlimited()).unwrap()
}

fn url_of(server: &MockServer, path: &str) -> Url {
    Url::parse(&format!("{}{path}", server.uri())).unwrap()
}

fn html(body: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body.as_bytes().to_vec(), "text/html")
}

async fn requests(server: &MockServer) -> usize {
    server.received_requests().await.unwrap().len()
}

#[tokio::test]
async fn an_html_page_is_read_and_decoded() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .and(header("user-agent", "bruce-test/1"))
        .respond_with(html("<h1 id=a>é</h1>"))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(got.status, 200);
    assert_eq!(got.content_type.as_deref(), Some("text/html"));
    assert_eq!(
        got.body,
        Body::Html {
            text: "<h1 id=a>é</h1>".to_owned(),
            size: "<h1 id=a>é</h1>".len() as u64
        }
    );
}

#[tokio::test]
async fn the_header_charset_decodes_the_body() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(vec![b'a', LATIN1_E_ACUTE], "text/html; charset=iso-8859-1"),
        )
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(
        got.body,
        Body::Html {
            text: "aé".to_owned(),
            size: 2
        }
    );
}

#[tokio::test]
async fn xhtml_is_read_and_other_types_are_not() {
    let server = MockServer::start().await;
    for (p, content_type) in [
        ("/x", "application/xhtml+xml; charset=utf-8"),
        ("/pdf", "application/pdf"),
        ("/img", "image/png"),
    ] {
        Mock::given(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_raw(b"<p>".to_vec(), content_type))
            .mount(&server)
            .await;
    }
    Mock::given(path("/none"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"<p>".to_vec()))
        .mount(&server)
        .await;
    let f = fetcher(quick());
    let read = |p: &'static str| {
        let (f, url) = (f.clone(), url_of(&server, p));
        async move { f.fetch(&url, Mode::Page).await.unwrap() }
    };
    assert!(matches!(read("/x").await.body, Body::Html { .. }));
    for p in ["/pdf", "/img", "/none"] {
        let got = read(p).await;
        assert_eq!((got.status, got.body), (200, Body::Unread), "{p}");
    }
}

#[tokio::test]
async fn a_non_2xx_body_is_never_read() {
    let server = MockServer::start().await;
    Mock::given(path("/gone"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(b"<h1>404".to_vec(), "text/html"))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/gone"), Mode::Page)
        .await
        .unwrap();
    assert_eq!((got.status, got.body), (404, Body::Unread));
    assert_eq!(requests(&server).await, 1, "a 4xx is not retried");
}

#[tokio::test]
async fn a_redirect_is_one_hop_with_its_location() {
    let server = MockServer::start().await;
    Mock::given(path("/old"))
        .respond_with(ResponseTemplate::new(301).insert_header("location", "/new"))
        .mount(&server)
        .await;
    Mock::given(path("/new"))
        .respond_with(html("new"))
        .mount(&server)
        .await;
    let f = fetcher(quick());
    for mode in [Mode::Page, Mode::Asset] {
        let got = f.fetch(&url_of(&server, "/old"), mode).await.unwrap();
        assert_eq!(got.status, 301);
        assert_eq!(got.location.as_deref(), Some("/new"));
        assert_eq!(got.body, Body::Unread);
    }
    let seen = server.received_requests().await.unwrap();
    assert!(seen.iter().all(|r| r.url.path() == "/old"), "{seen:?}");
}

#[tokio::test]
async fn a_gzip_body_is_decompressed_before_the_cap_is_applied() {
    let page = "<p>".to_owned() + &"a".repeat(100_000);
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(page.as_bytes()).unwrap();
    let gz = gz.finish().unwrap();
    assert!(
        gz.len() < 1_000,
        "the fixture compresses to {} bytes",
        gz.len()
    );
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-encoding", "gzip")
                .set_body_raw(gz, "text/html"),
        )
        .mount(&server)
        .await;
    let url = url_of(&server, "/p");

    let got = fetcher(quick()).fetch(&url, Mode::Page).await.unwrap();
    assert_eq!(
        got.body,
        Body::Html {
            size: page.len() as u64,
            text: page.clone()
        }
    );

    let capped = Settings {
        max_body: 50_000,
        ..quick()
    };
    let got = fetcher(capped).fetch(&url, Mode::Page).await.unwrap();
    assert_eq!((got.status, got.body), (200, Body::TooLarge));

    let exact = Settings {
        max_body: page.len(),
        ..quick()
    };
    let got = fetcher(exact).fetch(&url, Mode::Page).await.unwrap();
    assert!(
        matches!(got.body, Body::Html { .. }),
        "a body at the cap is read"
    );
}

#[tokio::test]
async fn a_server_error_is_retried_until_it_answers() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(path("/p"))
        .respond_with(html("ok"))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(got.status, 200);
    assert_eq!(requests(&server).await, 3);
}

#[tokio::test]
async fn a_server_error_that_stays_is_returned_after_three_retries() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(got.status, 500);
    assert_eq!(requests(&server).await, 4);
}

#[tokio::test]
async fn a_429_without_retry_after_backs_off() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(got.status, 429);
    assert_eq!(requests(&server).await, 4);
}

#[tokio::test]
async fn retry_after_sets_the_wait() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(path("/p"))
        .respond_with(html("ok"))
        .mount(&server)
        .await;
    let started = Instant::now();
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(got.status, 200);
    assert!(
        started.elapsed() >= Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn a_retry_after_over_the_cap_is_not_waited_for() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(ResponseTemplate::new(503).insert_header("retry-after", "61"))
        .mount(&server)
        .await;
    let started = Instant::now();
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap();
    assert_eq!(got.status, 503);
    assert_eq!(requests(&server).await, 1);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn retry_after_reads_seconds_and_dates() {
    let now = httpdate::parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
    let cases = [
        ("120", Some(120)),
        (" 7 ", Some(7)),
        ("Sun, 06 Nov 1994 08:50:37 GMT", Some(60)),
        ("Sun, 06 Nov 1994 08:00:00 GMT", Some(0)),
        ("soon", None),
        ("-1", None),
        ("", None),
    ];
    for (value, want) in cases {
        assert_eq!(
            retry_after(value, now),
            want.map(Duration::from_secs),
            "{value:?}"
        );
    }
}

#[tokio::test]
async fn a_slow_response_times_out_and_is_retried() {
    let server = MockServer::start().await;
    Mock::given(path("/p"))
        .respond_with(html("late").set_delay(Duration::from_millis(400)))
        .mount(&server)
        .await;
    let settings = Settings {
        timeout: Duration::from_millis(100),
        ..quick()
    };
    let failure = fetcher(settings)
        .fetch(&url_of(&server, "/p"), Mode::Page)
        .await
        .unwrap_err();
    assert_eq!(failure.kind, FailureKind::Timeout, "{failure}");
    assert_eq!(requests(&server).await, 4);
}

/// A listener that accepts connections and drops each at once, counting
/// them.
async fn dropping_listener() -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepted = Arc::new(AtomicUsize::new(0));
    let count = accepted.clone();
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            count.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });
    (port, accepted)
}

#[tokio::test]
async fn a_lost_connection_is_retried() {
    let (port, accepted) = dropping_listener().await;
    let url = Url::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
    let failure = fetcher(quick()).fetch(&url, Mode::Page).await.unwrap_err();
    assert_eq!(failure.kind, FailureKind::Reset, "{failure}");
    assert_eq!(accepted.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn a_refused_connection_is_not_retried() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let url = Url::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
    let started = Instant::now();
    let failure = fetcher(quick()).fetch(&url, Mode::Page).await.unwrap_err();
    assert_eq!(failure.kind, FailureKind::Connect, "{failure}");
    assert!(!failure.kind.is_transient());
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn a_server_not_speaking_tls_is_a_tls_failure_and_not_retried() {
    let server = MockServer::start().await;
    let url = Url::parse(&format!("https://{}/", server.address())).unwrap();
    let failure = fetcher(quick()).fetch(&url, Mode::Page).await.unwrap_err();
    assert_eq!(failure.kind, FailureKind::Tls, "{failure}");
    assert_eq!(requests(&server).await, 0);
}

#[tokio::test]
async fn an_asset_that_answers_head_costs_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/a.png"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/html"))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/a.png"), Mode::Asset)
        .await
        .unwrap();
    assert_eq!((got.status, got.body), (200, Body::Unread));
    assert_eq!(requests(&server).await, 1);
}

#[tokio::test]
async fn an_asset_whose_head_is_refused_is_asked_again_with_get() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .respond_with(ResponseTemplate::new(405))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"<p>".to_vec(), "text/html"))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/a"), Mode::Asset)
        .await
        .unwrap();
    assert_eq!(
        (got.status, got.body),
        (200, Body::Unread),
        "the GET's body is not read"
    );
    let methods: Vec<_> = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .map(|r| r.method.to_string())
        .collect();
    assert_eq!(methods, ["HEAD", "GET"]);
}

#[tokio::test]
async fn an_asset_missing_for_both_is_a_404() {
    let server = MockServer::start().await;
    Mock::given(path("/a"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/a"), Mode::Asset)
        .await
        .unwrap();
    assert_eq!(got.status, 404);
    assert_eq!(requests(&server).await, 2);
}

#[tokio::test]
async fn an_asset_whose_head_times_out_falls_back_to_get() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(400)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let settings = Settings {
        timeout: Duration::from_millis(100),
        ..quick()
    };
    let got = fetcher(settings)
        .fetch(&url_of(&server, "/a"), Mode::Asset)
        .await
        .unwrap();
    assert_eq!(got.status, 200);
}

#[tokio::test]
async fn an_asset_redirect_is_not_asked_again() {
    let server = MockServer::start().await;
    Mock::given(path("/a"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/b"))
        .mount(&server)
        .await;
    let got = fetcher(quick())
        .fetch(&url_of(&server, "/a"), Mode::Asset)
        .await
        .unwrap();
    assert_eq!((got.status, got.location.as_deref()), (302, Some("/b")));
    assert_eq!(requests(&server).await, 1);
}

#[test]
fn the_cause_is_found_down_the_chain() {
    let nx = NetError::from(DnsError::NoRecordsFound(NoRecords::new(
        Query::default(),
        ResponseCode::NXDomain,
    )));
    let no_data = NetError::from(DnsError::NoRecordsFound(NoRecords::new(
        Query::default(),
        ResponseCode::NoError,
    )));
    let tls = io::Error::new(
        io::ErrorKind::InvalidData,
        rustls::Error::General("bad".into()),
    );
    let reset = io::Error::from(io::ErrorKind::ConnectionReset);
    let other = io::Error::from(io::ErrorKind::PermissionDenied);
    assert_eq!(cause(&nx), Some(FailureKind::NxDomain));
    assert_eq!(
        cause(&no_data),
        None,
        "a name with no records of the type is not NXDOMAIN"
    );
    assert_eq!(cause(&tls), Some(FailureKind::Tls));
    assert_eq!(cause(&reset), Some(FailureKind::Reset));
    assert_eq!(cause(&other), None);
    let wrapped = io::Error::other(nx);
    assert_eq!(cause(&wrapped), Some(FailureKind::NxDomain));
}

#[test]
fn only_a_timeout_or_a_lost_connection_is_transient() {
    use FailureKind::*;
    let transient: Vec<_> = [NxDomain, Timeout, Tls, Connect, Reset, Other]
        .into_iter()
        .filter(|k| k.is_transient())
        .collect();
    assert_eq!(transient, [Timeout, Reset]);
}
