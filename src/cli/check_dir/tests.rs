use std::path::Path;
use std::process::ExitCode;

use wiremock::MockServer;

use super::*;
use crate::cli::registry;
use crate::report::Format;
use crate::store::Store;

fn parse(args: &[&str]) -> Result<Args, clap::Error> {
    let m =
        command().try_get_matches_from(std::iter::once("check-dir").chain(args.iter().copied()))?;
    Ok(Args::from(&m))
}

#[test]
fn a_directory_and_a_prefix_are_required() {
    let args = parse(&[
        "build",
        "--prefix",
        "https://x.test/",
        "--baseline",
        "b.duckdb.gz",
        "--db",
        "d",
    ])
    .unwrap();
    assert_eq!(
        args,
        Args {
            dir: "build".into(),
            prefix: "https://x.test/".into(),
            baseline: "b.duckdb.gz".into(),
            externals: Externals::Baseline,
            config: None,
            db: Some("d".into()),
            format: Format::Text,
            all: false,
        }
    );
    let wide = parse(&[
        "build",
        "--prefix",
        "p",
        "--baseline",
        "b",
        "--format",
        "json",
        "--all",
    ])
    .unwrap();
    assert_eq!((wide.format, wide.all), (Format::Json, true));
    let live = parse(&[
        "build",
        "--prefix",
        "https://x.test/",
        "--baseline",
        "b",
        "--externals",
        "live",
    ])
    .unwrap();
    assert_eq!(live.externals, Externals::Live);
    // The gate judges every kind, so there is no way to narrow it.
    assert!(
        parse(&[
            "build",
            "--prefix",
            "p",
            "--baseline",
            "b",
            "--kind",
            "image"
        ])
        .is_err()
    );
    assert!(parse(&["build", "--prefix", "https://x.test/"]).is_err());
    assert!(parse(&["build", "--baseline", "b"]).is_err());
    assert!(parse(&["--prefix", "https://x.test/", "--baseline", "b"]).is_err());
    let bad = [
        "build",
        "--prefix",
        "p",
        "--baseline",
        "b",
        "--externals",
        "x",
    ];
    assert!(parse(&bad).is_err());
}

/// A baseline file holding one finished run and nothing else.
fn empty_baseline(path: &Path) {
    let store = Store::open(path, &registry()).unwrap();
    let run = store
        .begin_run(crate::store::Mode::Crawl, "base", None, &[])
        .unwrap();
    store.finish_run(run).unwrap();
}

/// A build of three pages under `https://x.test/docs/sub/`, whose root in
/// the configuration is `https://x.test/docs/`.
fn build(dir: &Path) {
    let page = |at: &str, body: &str| {
        let path = dir.join(at);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    };
    page(
        "index.html",
        r#"<a href="a/">a</a> <a href="/gone/">root-relative</a>"#,
    );
    page(
        "a/index.html",
        r##"<a href="../b.html#x">b</a> <a href="#nope">self</a> <a href="https://elsewhere.test/">e</a>
           <img src="../_/logo.png">"##,
    );
    page("b.html", r#"<h2 id="x">x</h2>"#);
    page("_/logo.png", "png");
}

#[test]
fn a_run_over_a_build_answers_from_the_directory_and_asks_production_for_the_rest() {
    let server_runtime = tokio::runtime::Runtime::new().unwrap();
    let server = server_runtime.block_on(MockServer::start());
    let base = server.uri();
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("build");
    build(&dir);
    let config = tmp.path().join("bruce.toml");
    std::fs::write(&config, format!("[[root]]\nurl = \"{base}/docs/\"\n")).unwrap();
    let baseline = tmp.path().join("baseline.duckdb");
    empty_baseline(&baseline);
    let db = tmp.path().join("bruce.duckdb");
    let code = crate::cli::run_from([
        "bruce",
        "check-dir",
        dir.to_str().unwrap(),
        "--prefix",
        &format!("{base}/docs/sub"),
        "--baseline",
        baseline.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--db",
        db.to_str().unwrap(),
    ]);
    // The links to `gone/` and `#nope` are broken and the empty baseline has
    // no word of either, so the gate fails.
    assert_eq!(code, ExitCode::FAILURE);

    let store = Store::open(&db, &registry()).unwrap();
    let mut stmt = store
        .connection()
        .prepare(
            "SELECT url, outcome, reason, answered_by FROM targets WHERE run_id = 2 ORDER BY url",
        )
        .unwrap();
    let targets: Vec<(String, String, Option<String>, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let t = |url: &str, outcome: &str, answered_by: &str| {
        (
            url.replace("{base}", &base),
            outcome.to_owned(),
            None,
            answered_by.to_owned(),
        )
    };
    assert_eq!(
        targets,
        [
            t("{base}/docs/sub/", "ok", "local"),
            t("{base}/docs/sub/_/logo.png", "ok", "local"),
            t("{base}/docs/sub/a/", "ok", "local"),
            t("{base}/docs/sub/b.html", "ok", "local"),
            t("{base}/docs/sub/gone/", "broken_url", "live"),
            (
                "https://elsewhere.test/".to_owned(),
                "unchecked".to_owned(),
                Some("host_not_allowlisted".to_owned()),
                "live".to_owned()
            ),
        ],
        "a built page is read from the directory, and the root-relative href lands under the \
         prefix, where the build lacks it and the baseline has no word of it, so production is asked"
    );
    let (mode, finished, baseline_run): (String, bool, Option<u64>) = store
        .connection()
        .query_row(
            "SELECT mode, finished_at IS NOT NULL, baseline_run FROM runs WHERE run_id = 2",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (mode.as_str(), finished, baseline_run),
        ("local", true, Some(1))
    );
    drop(server_runtime);
}

#[test]
fn a_prefix_outside_every_roots_scope_is_refused_before_a_database_is_opened() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("build");
    build(&dir);
    let config = tmp.path().join("bruce.toml");
    std::fs::write(&config, "[[root]]\nurl = \"https://x.test/docs/\"\n").unwrap();
    let baseline = tmp.path().join("baseline.duckdb");
    empty_baseline(&baseline);
    let db = tmp.path().join("bruce.duckdb");
    for prefix in ["https://y.test/docs/", "not a url"] {
        let code = crate::cli::run_from([
            "bruce",
            "check-dir",
            dir.to_str().unwrap(),
            "--prefix",
            prefix,
            "--baseline",
            baseline.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--db",
            db.to_str().unwrap(),
        ]);
        assert_eq!(code, ExitCode::from(2), "{prefix}");
    }
    assert!(!db.exists());
}

#[test]
fn a_baseline_that_is_not_one_is_refused_and_the_run_is_not_begun() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("build");
    build(&dir);
    let config = tmp.path().join("bruce.toml");
    std::fs::write(&config, "[[root]]\nurl = \"https://x.test/docs/\"\n").unwrap();
    let not_a_database = tmp.path().join("notes.txt");
    std::fs::write(&not_a_database, "not a database").unwrap();
    let db = tmp.path().join("bruce.duckdb");
    for baseline in [not_a_database, tmp.path().join("missing.duckdb")] {
        let code = crate::cli::run_from([
            "bruce",
            "check-dir",
            dir.to_str().unwrap(),
            "--prefix",
            "https://x.test/docs/",
            "--baseline",
            baseline.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--db",
            db.to_str().unwrap(),
        ]);
        assert_eq!(code, ExitCode::from(2), "{}", baseline.display());
    }
    let store = Store::open(&db, &registry()).unwrap();
    let runs: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM runs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(runs, 0);
}

#[test]
fn a_run_checked_against_its_own_export_introduces_nothing_and_a_clean_build_passes() {
    let server_runtime = tokio::runtime::Runtime::new().unwrap();
    let server = server_runtime.block_on(MockServer::start());
    let base = server.uri();
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("build");
    build(&dir);
    let config = tmp.path().join("bruce.toml");
    std::fs::write(&config, format!("[[root]]\nurl = \"{base}/docs/\"\n")).unwrap();
    let first = tmp.path().join("first.duckdb");
    empty_baseline(&first);
    let db = tmp.path().join("bruce.duckdb");
    let check = |baseline: &Path, dir: &Path, format: &str| {
        crate::cli::run_from([
            "bruce",
            "check-dir",
            dir.to_str().unwrap(),
            "--prefix",
            &format!("{base}/docs/sub"),
            "--baseline",
            baseline.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--db",
            db.to_str().unwrap(),
            "--format",
            format,
        ])
    };
    assert_eq!(check(&first, &dir, "text"), ExitCode::FAILURE);
    // The breakage is now the baseline's, whichever way it is printed.
    let exported = tmp.path().join("second.duckdb");
    let code = crate::cli::run_from([
        "bruce",
        "export-baseline",
        "--db",
        db.to_str().unwrap(),
        "--out",
        exported.to_str().unwrap(),
    ]);
    assert_eq!(code, ExitCode::SUCCESS);
    assert_eq!(check(&exported, &dir, "json"), ExitCode::SUCCESS);

    let clean = tmp.path().join("clean");
    std::fs::create_dir_all(&clean).unwrap();
    std::fs::write(
        clean.join("index.html"),
        r##"<h2 id="x">x</h2><a href="#x">x</a>"##,
    )
    .unwrap();
    assert_eq!(check(&first, &clean, "text"), ExitCode::SUCCESS);
    drop(server_runtime);
}
