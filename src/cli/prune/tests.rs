use std::process::ExitCode;

use super::*;
use crate::cli::registry;
use crate::store::{Mode, Store};

fn parse(args: &[&str]) -> Result<Args, clap::Error> {
    let m = command().try_get_matches_from(std::iter::once("prune").chain(args.iter().copied()))?;
    Ok(Args::from(&m))
}

#[test]
fn keep_or_older_than_is_required_and_only_one_is_allowed() {
    assert_eq!(parse(&["--keep", "3"]).unwrap().rule, Rule::Keep(3));
    assert_eq!(
        parse(&["--older-than", "2w", "--db", "d"]).unwrap(),
        Args {
            rule: Rule::OlderThan(Duration::from_secs(2 * 604_800)),
            db: Some("d".into())
        }
    );
    assert!(parse(&[]).is_err());
    assert!(parse(&["--db", "d"]).is_err());
    assert!(parse(&["--keep", "3", "--older-than", "1d"]).is_err());
    assert!(parse(&["--keep", "0"]).is_err());
    assert!(parse(&["--keep", "x"]).is_err());
}

#[test]
fn an_age_is_a_number_and_a_unit() {
    let secs = |s| age(s).map(|d| d.as_secs());
    assert_eq!(secs("30s"), Ok(30));
    assert_eq!(secs("5m"), Ok(300));
    assert_eq!(secs("12h"), Ok(43_200));
    assert_eq!(secs("90d"), Ok(90 * 86_400));
    assert_eq!(secs("0d"), Ok(0));
    for bad in ["", "d", "90", "9 d", "1.5d", "-1d", "1y", "1D", "1dd"] {
        assert!(age(bad).is_err(), "{bad:?}");
    }
    assert!(age("99999999999999999999d").is_err());
    assert!(age(&format!("{}w", u64::MAX)).is_err());
}

#[test]
fn prune_deletes_through_the_command_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bruce.duckdb");
    {
        let store = Store::open(&path, &registry()).unwrap();
        for _ in 0..3 {
            let run = store.begin_run(Mode::Crawl, "abc", None, &[]).unwrap();
            store.finish_run(run).unwrap();
        }
    }
    let db = path.to_str().unwrap();
    let go = |args: &[&str]| {
        crate::cli::run_from(
            ["bruce", "prune", "--db", db]
                .into_iter()
                .chain(args.iter().copied()),
        )
    };
    assert_eq!(go(&["--keep", "2"]), ExitCode::SUCCESS);
    assert_eq!(go(&["--keep", "2"]), ExitCode::SUCCESS);
    let store = Store::open(&path, &registry()).unwrap();
    let runs: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM runs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(runs, 2);
}

#[test]
fn a_missing_database_is_an_error_and_is_not_created() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("none.duckdb");
    let args = Args {
        rule: Rule::Keep(1),
        db: Some(path.clone()),
    };
    assert_eq!(run(&args, registry()), ExitCode::FAILURE);
    assert!(!path.exists());
}

#[test]
fn a_command_line_without_a_rule_exits_2() {
    assert_eq!(crate::cli::run_from(["bruce", "prune"]), ExitCode::from(2));
}
