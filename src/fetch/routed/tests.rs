use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::fetch::http::host::Hosts;
use crate::fetch::http::{HttpFetcher, Settings};

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

#[tokio::test]
async fn a_url_the_build_holds_is_read_from_it_and_any_other_is_asked_live() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.html");
    std::fs::write(&file, "<p>built</p>").unwrap();
    let local = LocalFetcher::new(
        &url("https://x.org/en-US/"),
        [(url("https://x.org/en-US/a/"), file)],
    );
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let live = HttpFetcher::new(Settings::default(), Hosts::unlimited()).unwrap();
    let routed = Routed::new(local, live);

    let built = routed
        .fetch(&url("https://x.org/en-US/a/"), Mode::Asset)
        .await
        .unwrap();
    assert_eq!(built.status, 200);
    assert!(
        server.received_requests().await.unwrap().is_empty(),
        "a built page did not leave the machine"
    );

    let ext = routed
        .fetch(&url(&format!("{}/ext", server.uri())), Mode::Page)
        .await
        .unwrap();
    assert_eq!(ext.status, 204);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    assert_eq!(
        routed.access(&url("https://x.org/en-US/a/")).await,
        Access::Allowed,
        "robots.txt is not asked for a built page"
    );
}
