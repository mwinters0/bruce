use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

/// A fetcher serving `files` under `https://x.org/en-US/`, each at the URL
/// given and holding the text beside it.
fn serving(dir: &tempfile::TempDir, files: &[(&str, &str, &[u8])]) -> LocalFetcher {
    let served = files.iter().map(|(at, name, bytes)| {
        let file = dir.path().join(name);
        std::fs::write(&file, bytes).unwrap();
        (url(at), file)
    });
    LocalFetcher::new(&url("https://x.org/en-US/"), served)
}

async fn ask(f: &LocalFetcher, at: &str, mode: Mode) -> Result<Response, Failure> {
    f.fetch(&url(at), mode).await
}

#[tokio::test]
async fn a_file_answers_200_with_its_decoded_page_and_a_status_alone_reads_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let f = serving(
        &dir,
        &[(
            "https://x.org/en-US/a/",
            "a.html",
            "<p>caf\u{e9}</p>".as_bytes(),
        )],
    );
    let page = ask(&f, "https://x.org/en-US/a/?q=1#frag", Mode::Page)
        .await
        .unwrap();
    assert_eq!(page.status, 200);
    assert_eq!(page.content_type.as_deref(), Some("text/html"));
    assert_eq!(
        page.body,
        Body::Html {
            text: "<p>caf\u{e9}</p>".into(),
            size: 12
        },
        "the query and the fragment are not part of the name"
    );
    let head = ask(&f, "https://x.org/en-US/a/", Mode::Asset)
        .await
        .unwrap();
    assert_eq!((head.status, head.body), (200, Body::Unread));
}

#[tokio::test]
async fn a_file_that_is_not_html_is_not_read_as_a_page() {
    let dir = tempfile::tempdir().unwrap();
    let f = serving(
        &dir,
        &[("https://x.org/en-US/_/site.css", "site.css", b"a{}")],
    );
    let css = ask(&f, "https://x.org/en-US/_/site.css", Mode::Page)
        .await
        .unwrap();
    assert_eq!(css.content_type.as_deref(), Some("text/css"));
    assert_eq!(css.body, Body::Unread);
    let map = ask(&f, "https://x.org/en-US/_/site.css", Mode::Sitemap)
        .await
        .unwrap();
    assert_eq!(map.body, Body::Raw(b"a{}".to_vec()));
}

#[tokio::test]
async fn a_name_without_its_slash_moves_to_the_slashed_one_and_a_missing_page_is_a_404() {
    let dir = tempfile::tempdir().unwrap();
    let f = serving(&dir, &[("https://x.org/en-US/a/b/", "b.html", b"x")]);
    let moved = ask(&f, "https://x.org/en-US/a/b", Mode::Page)
        .await
        .unwrap();
    assert_eq!((moved.status, moved.location.as_deref()), (301, Some("b/")));
    let missing = ask(&f, "https://x.org/en-US/a/c/", Mode::Page)
        .await
        .unwrap();
    assert_eq!((missing.status, missing.body), (404, Body::Unread));
}

#[tokio::test]
async fn a_url_outside_the_prefix_is_a_failure_not_an_answer() {
    let dir = tempfile::tempdir().unwrap();
    let f = serving(&dir, &[("https://x.org/en-US/a/", "a.html", b"x")]);
    for outside in ["https://x.org/en-GB/a/", "https://y.org/en-US/a/"] {
        let failure = ask(&f, outside, Mode::Page).await.unwrap_err();
        assert_eq!(failure.kind, FailureKind::Other, "{outside}");
    }
}

#[tokio::test]
async fn a_file_over_the_cap_is_too_large_and_an_unreadable_one_is_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    let big = vec![b' '; MAX_BODY + 1];
    let f = serving(&dir, &[("https://x.org/en-US/big/", "big.html", &big)]);
    let page = ask(&f, "https://x.org/en-US/big/", Mode::Page)
        .await
        .unwrap();
    assert_eq!((page.status, page.body), (200, Body::TooLarge));
    std::fs::remove_file(dir.path().join("big.html")).unwrap();
    let gone = ask(&f, "https://x.org/en-US/big/", Mode::Page)
        .await
        .unwrap_err();
    assert_eq!(gone.kind, FailureKind::Other);
}
