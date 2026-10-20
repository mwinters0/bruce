use super::*;

fn config() -> Config {
    Config::parse("[[root]]\nurl = \"https://x.org/en-US/\"\n").unwrap()
}

/// A build holding `files`, written under a fresh directory.
fn build(files: &[&str], prefix: &str) -> (tempfile::TempDir, Result<Build, Error>) {
    let dir = tempfile::tempdir().unwrap();
    for file in files {
        let path = dir.path().join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "x").unwrap();
    }
    let built = Build::scan(dir.path(), prefix, &config());
    (dir, built)
}

fn urls(urls: &[Url]) -> Vec<&str> {
    urls.iter().map(Url::as_str).collect()
}

#[test]
fn a_file_is_the_prefix_plus_its_path_and_an_index_is_its_directory_too_d22() {
    let (dir, built) = build(
        &[
            "index.html",
            "tools/index.html",
            "tools/usage/index.html",
            "ops/disks.html",
            "_/css/site.css",
        ],
        "https://x.org/en-US",
    );
    let built = built.unwrap();
    assert_eq!(built.prefix().as_str(), "https://x.org/en-US/");
    assert_eq!(
        urls(built.seeds()),
        [
            "https://x.org/en-US/_/css/site.css",
            "https://x.org/en-US/",
            "https://x.org/en-US/ops/disks.html",
            "https://x.org/en-US/tools/",
            "https://x.org/en-US/tools/usage/",
        ],
        "every file, an index as its directory, in path order"
    );
    let served: Vec<(String, PathBuf)> = built.served().map(|(u, f)| (u.to_string(), f)).collect();
    let at = |url: &str| served.iter().filter(|(u, _)| u == url).count();
    assert_eq!(at("https://x.org/en-US/tools/"), 1);
    assert_eq!(at("https://x.org/en-US/tools/index.html"), 1);
    assert_eq!(
        at("https://x.org/en-US/_/css/site.css"),
        1,
        "an asset is answered"
    );
    assert_eq!(
        served.len(),
        2 * 3 + 2,
        "an index is two URLs, any other file one"
    );
    let (_, file) = served.iter().find(|(u, _)| u.ends_with("/tools/")).unwrap();
    assert_eq!(file, &dir.path().join("tools/index.html"));
}

#[test]
fn a_name_that_is_not_a_plain_path_segment_is_escaped() {
    let (_dir, built) = build(&["a b#c.html"], "https://x.org/en-US/");
    assert_eq!(
        urls(built.unwrap().seeds()),
        ["https://x.org/en-US/a%20b%23c.html"]
    );
}

#[test]
fn a_prefix_must_be_a_web_directory_in_a_roots_scope() {
    for (prefix, says) in [
        ("not a url", "is not a URL"),
        ("ftp://x.org/en-US/", "http or https"),
        ("https://x.org/en-US/?a=1", "query"),
        ("https://x.org/en-GB/", "outside every root"),
        ("https://y.org/en-US/", "outside every root"),
    ] {
        let (_dir, built) = build(&["index.html"], prefix);
        let e = built.unwrap_err().to_string();
        assert!(e.contains(says), "{prefix}: {e}");
    }
    let (_dir, deeper) = build(&["index.html"], "https://x.org/en-US/tools/");
    assert!(deeper.is_ok(), "a prefix may sit below a root's URL");
}

#[test]
fn a_directory_that_cannot_be_read_is_an_error_naming_it() {
    let e = Build::scan(
        Path::new("/nonexistent/build"),
        "https://x.org/en-US/",
        &config(),
    )
    .unwrap_err()
    .to_string();
    assert!(e.contains("/nonexistent/build"), "{e}");
}
