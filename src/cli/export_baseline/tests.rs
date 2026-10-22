use std::process::ExitCode;

use super::*;
use crate::cli::registry;
use crate::store::{Mode, Store};

fn parse(args: &[&str]) -> Result<Args, clap::Error> {
    let m = command()
        .try_get_matches_from(std::iter::once("export-baseline").chain(args.iter().copied()))?;
    Ok(Args::from(&m))
}

#[test]
fn an_out_file_is_required_and_the_rest_default() {
    assert_eq!(
        parse(&["--out", "b"]).unwrap(),
        Args {
            out: "b".into(),
            run: None,
            db: None,
            force: false
        }
    );
    let all = parse(&["--out", "b", "--run", "3", "--db", "d", "--force"]).unwrap();
    assert_eq!(
        (all.run, all.db, all.force),
        (Some(RunId(3)), Some("d".into()), true)
    );
    assert!(parse(&[]).is_err());
    assert!(parse(&["--out", "b", "--run", "x"]).is_err());
}

/// A database holding a finished run 1 and an unfinished run 2, in a
/// directory the caller exports into.
fn dir_with_runs() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("bruce.duckdb");
    let store = Store::open(&db, &registry()).unwrap();
    let first = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    store
        .connection()
        .execute(
            "INSERT INTO targets (answered_by, run_id, url_id, url, outcome) \
             VALUES ('live', 1, 1, 'https://a.test/', 'ok')",
            [],
        )
        .unwrap();
    store.finish_run(first).unwrap();
    store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
    (dir, db)
}

fn go(db: &Path, args: &[&str]) -> ExitCode {
    crate::cli::run_from(
        ["bruce", "export-baseline", "--db", db.to_str().unwrap()]
            .into_iter()
            .chain(args.iter().copied()),
    )
}

#[test]
fn a_finished_run_is_written_and_an_existing_file_needs_force() {
    let (dir, db) = dir_with_runs();
    let out = dir.path().join("baseline.duckdb");
    let o = out.to_str().unwrap();
    assert_eq!(go(&db, &["--out", o]), ExitCode::SUCCESS);
    let file = Store::open(&out, &registry()).unwrap();
    let n: i64 = file
        .connection()
        .query_row("SELECT count(*) FROM targets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    drop(file);
    assert_eq!(go(&db, &["--out", o]), ExitCode::FAILURE);
    assert_eq!(go(&db, &["--out", o, "--force"]), ExitCode::SUCCESS);
}

#[test]
fn an_unfinished_run_is_refused_and_a_forced_out_survives_the_refusal() {
    let (dir, db) = dir_with_runs();
    let out = dir.path().join("baseline.duckdb");
    std::fs::write(&out, b"precious").unwrap();
    let o = out.to_str().unwrap();
    assert_eq!(
        go(&db, &["--out", o, "--force", "--run", "2"]),
        ExitCode::FAILURE
    );
    assert_eq!(std::fs::read(&out).unwrap(), b"precious");
}

#[test]
fn the_database_itself_and_a_missing_database_are_refused() {
    let (dir, db) = dir_with_runs();
    let d = db.to_str().unwrap();
    assert_eq!(go(&db, &["--out", d, "--force"]), ExitCode::FAILURE);
    assert!(Store::open(&db, &registry()).is_ok());
    let none = dir.path().join("none.duckdb");
    assert_eq!(go(&none, &["--out", "x"]), ExitCode::FAILURE);
    assert!(!none.exists());
}

#[test]
fn a_gzip_is_measured_and_the_summary_names_both_sizes() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("zeros");
    std::fs::write(&f, vec![0u8; 1 << 20]).unwrap();
    let gz = gzipped_size(&f).unwrap();
    assert!(gz > 0 && gz < 4096, "{gz}");
    let e = Exported {
        run: RunId(7),
        targets: 3,
        pages: 2,
        facts: 1,
        anchors: 4,
        links: 5,
    };
    assert_eq!(
        summary(&e, Path::new("b.duckdb"), 1 << 20, 524_288),
        "bruce export-baseline: run 7 to b.duckdb: 3 targets, 2 pages, 4 anchors, 5 links; \
         1048576 bytes (1.0 MiB), 524288 bytes (0.5 MiB) gzipped"
    );
}
