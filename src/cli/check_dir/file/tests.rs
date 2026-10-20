use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().unwrap()
}

fn config() -> Config {
    Config::for_url("https://x.test/docs/").unwrap()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::fast());
    enc.write_all(bytes).unwrap();
    enc.finish().unwrap()
}

#[test]
fn a_path_is_used_as_it_is_and_a_gzipped_file_is_decompressed_whatever_its_name() {
    let tmp = tempfile::tempdir().unwrap();
    let plain = tmp.path().join("b.duckdb");
    std::fs::write(&plain, b"database bytes").unwrap();
    let file = BaselineFile::open(plain.to_str().unwrap(), &config(), &runtime()).unwrap();
    assert_eq!(file.path(), plain);

    let packed = tmp.path().join("b.bin");
    std::fs::write(&packed, gzip(b"database bytes")).unwrap();
    let file = BaselineFile::open(packed.to_str().unwrap(), &config(), &runtime()).unwrap();
    assert_ne!(file.path(), packed);
    assert_eq!(std::fs::read(file.path()).unwrap(), b"database bytes");
    let made = file.path().to_owned();
    drop(file);
    assert!(!made.exists(), "the decompressed copy goes with the file");
    assert!(packed.exists(), "the file given is left");
}

#[test]
fn a_missing_file_is_an_error_naming_it() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("nope.duckdb");
    let e = BaselineFile::open(missing.to_str().unwrap(), &config(), &runtime()).unwrap_err();
    assert!(e.contains("nope.duckdb"), "{e}");
}

#[test]
fn a_url_is_downloaded_through_a_redirect_and_a_gzipped_download_is_decompressed() {
    let runtime = runtime();
    let server = runtime.block_on(MockServer::start());
    runtime.block_on(async {
        Mock::given(path("/latest"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/b.duckdb.gz"))
            .mount(&server)
            .await;
        Mock::given(path("/b.duckdb.gz"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(gzip(b"database bytes")))
            .mount(&server)
            .await;
    });
    let file =
        BaselineFile::open(&format!("{}/latest", server.uri()), &config(), &runtime).unwrap();
    assert_eq!(std::fs::read(file.path()).unwrap(), b"database bytes");
    let temporaries = file.temporary.clone();
    assert_eq!(
        temporaries.len(),
        2,
        "the download and its decompressed copy"
    );
    drop(file);
    assert!(temporaries.iter().all(|p| !p.exists()));
}

#[test]
fn a_url_that_answers_an_error_is_one() {
    let runtime = runtime();
    let server = runtime.block_on(MockServer::start());
    runtime.block_on(async {
        Mock::given(path("/b.duckdb"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
    });
    let url = format!("{}/b.duckdb", server.uri());
    let e = BaselineFile::open(&url, &config(), &runtime).unwrap_err();
    assert!(e.contains("404"), "{e}");
}
