use std::path::Path;
use std::process::ExitCode;

use super::*;
use crate::cli::registry;
use crate::store::Store;

fn parse(args: &[&str]) -> Result<Args, clap::Error> {
    let m =
        command().try_get_matches_from(std::iter::once("check-dir").chain(args.iter().copied()))?;
    Ok(Args::from(&m))
}

#[test]
fn a_directory_and_a_prefix_are_required() {
    let args = parse(&["build", "--prefix", "https://x.test/", "--db", "d"]).unwrap();
    assert_eq!(
        args,
        Args {
            dir: "build".into(),
            prefix: "https://x.test/".into(),
            config: None,
            db: Some("d".into()),
        }
    );
    assert!(parse(&["build"]).is_err());
    assert!(parse(&["--prefix", "https://x.test/"]).is_err());
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
fn a_run_over_a_build_answers_from_the_directory_and_asks_the_network_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("build");
    build(&dir);
    let config = tmp.path().join("bruce.toml");
    std::fs::write(&config, "[[root]]\nurl = \"https://x.test/docs/\"\n").unwrap();
    let db = tmp.path().join("bruce.duckdb");
    let code = crate::cli::run_from([
        "bruce",
        "check-dir",
        dir.to_str().unwrap(),
        "--prefix",
        "https://x.test/docs/sub",
        "--config",
        config.to_str().unwrap(),
        "--db",
        db.to_str().unwrap(),
    ]);
    assert_eq!(code, ExitCode::SUCCESS);

    let store = Store::open(&db, &registry()).unwrap();
    let mut stmt = store
        .connection()
        .prepare("SELECT url, outcome, reason, answered_by FROM targets ORDER BY url")
        .unwrap();
    let targets: Vec<(String, String, Option<String>, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let t = |url: &str, outcome: &str, reason: Option<&str>| {
        (
            url.to_owned(),
            outcome.to_owned(),
            reason.map(str::to_owned),
            "live".to_owned(),
        )
    };
    assert_eq!(
        targets,
        [
            t(
                "https://elsewhere.test/",
                "unchecked",
                Some("host_not_allowlisted")
            ),
            t("https://x.test/docs/sub/", "ok", None),
            t("https://x.test/docs/sub/_/logo.png", "ok", None),
            t("https://x.test/docs/sub/a/", "ok", None),
            t("https://x.test/docs/sub/b.html", "ok", None),
            t("https://x.test/docs/sub/gone/", "broken_url", None),
        ],
        "the root's own URL is not asked about, and a root-relative href lands under the prefix"
    );
    let (mode, finished): (String, bool) = store
        .connection()
        .query_row("SELECT mode, finished_at IS NOT NULL FROM runs", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!((mode.as_str(), finished), ("local", true));
}

#[test]
fn a_prefix_outside_every_roots_scope_is_refused_before_a_database_is_opened() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("build");
    build(&dir);
    let config = tmp.path().join("bruce.toml");
    std::fs::write(&config, "[[root]]\nurl = \"https://x.test/docs/\"\n").unwrap();
    let db = tmp.path().join("bruce.duckdb");
    for prefix in ["https://y.test/docs/", "not a url"] {
        let code = crate::cli::run_from([
            "bruce",
            "check-dir",
            dir.to_str().unwrap(),
            "--prefix",
            prefix,
            "--config",
            config.to_str().unwrap(),
            "--db",
            db.to_str().unwrap(),
        ]);
        assert_eq!(code, ExitCode::FAILURE, "{prefix}");
    }
    assert!(!db.exists());
}
