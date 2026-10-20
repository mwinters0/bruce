use std::process::ExitCode;

use tempfile::TempDir;

use super::*;
use crate::store::Store;

fn parse(args: &[&str]) -> Result<Args, clap::Error> {
    let m =
        command().try_get_matches_from(std::iter::once("report").chain(args.iter().copied()))?;
    Ok(Args::from(&m))
}

#[test]
fn a_report_takes_a_name_and_defaults_to_anchors_in_text() {
    let a = parse(&["broken"]).unwrap();
    assert_eq!(a.request, Request::new(Report::Broken));
    assert_eq!((a.db, a.format), (None, Format::Text));
    assert!(parse(&["nosuch"]).is_err());
    assert!(parse(&[]).is_err());
}

#[test]
fn every_option_reaches_the_request() {
    let a = parse(&[
        "stats", "--db", "d", "--run", "3", "--repo", "r/x", "--fact", "a.b=c", "--fact",
        "d.e=f=g", "--kind", "image", "--kind", "anchor", "--top", "5", "--format", "json",
    ])
    .unwrap();
    let r = &a.request;
    assert_eq!(r.report, Report::Stats);
    assert_eq!(r.run, Some(RunId(3)));
    assert_eq!(r.repo.as_deref(), Some("r/x"));
    assert_eq!(
        r.facts,
        [("a.b".into(), "c".into()), ("d.e".into(), "f=g".into())]
    );
    assert_eq!(r.kinds, [Kind::Anchor, Kind::Image]);
    assert_eq!(r.top, Some(5));
    assert_eq!((a.db, a.format), (Some("d".into()), Format::Json));
}

#[test]
fn json_asks_for_the_linking_pages_text_does_not() {
    assert!(!parse(&["redirects"]).unwrap().request.linking_pages);
    let json = parse(&["redirects", "--format", "json"]).unwrap();
    assert!(json.request.linking_pages);
}

#[test]
fn diff_takes_from_and_to_and_the_others_take_run() {
    let a = parse(&["diff", "--from", "2", "--to", "5"]).unwrap();
    assert_eq!(
        (a.request.from, a.request.to),
        (Some(RunId(2)), Some(RunId(5)))
    );
    assert_eq!(a.request.run, None);
    let check = |args: &[&str]| {
        let m = command()
            .try_get_matches_from(std::iter::once("report").chain(args.iter().copied()))
            .unwrap();
        check(&m)
    };
    assert!(check(&["diff", "--from", "1"]).is_ok());
    assert!(check(&["broken", "--run", "1"]).is_ok());
    assert!(check(&["diff", "--run", "1"]).is_err());
    assert!(check(&["broken", "--from", "1"]).is_err());
    assert!(check(&["stats", "--to", "1"]).is_err());
}

#[test]
fn kind_all_counts_every_kind() {
    let a = parse(&["broken", "--kind", "all"]).unwrap();
    assert_eq!(a.request.kinds, Kind::ALL);
}

#[test]
fn introduced_judges_every_kind_and_the_rest_anchors() {
    let a = parse(&["introduced", "--run", "2"]).unwrap();
    assert_eq!(a.request.kinds, Kind::ALL);
    let a = parse(&["introduced", "--kind", "image"]).unwrap();
    assert_eq!(a.request.kinds, [Kind::Image]);
    let a = parse(&["broken"]).unwrap();
    assert_eq!(a.request.kinds, [Kind::Anchor]);
}

#[test]
fn a_bad_fact_kind_top_or_format_is_a_usage_error() {
    for bad in [
        ["--fact", "nokey"],
        ["--fact", "=v"],
        ["--kind", "nosuch"],
        ["--top", "0"],
        ["--format", "xml"],
        ["--run", "x"],
    ] {
        assert!(parse(&["broken", bad[0], bad[1]]).is_err(), "{bad:?}");
    }
}

#[test]
fn the_command_line_is_consistent() {
    command().debug_assert();
}

fn args_for(db: &std::path::Path) -> Args {
    Args {
        request: Request::new(Report::Broken),
        db: Some(db.to_owned()),
        format: Format::Text,
    }
}

#[test]
fn a_database_that_is_not_there_is_refused_and_not_created() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("none.duckdb");
    let code = run(&args_for(&db), crate::cli::registry());
    assert_eq!(code, ExitCode::FAILURE);
    assert!(!db.exists());
}

#[test]
fn a_database_with_no_finished_run_fails_and_one_with_a_run_succeeds() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("a.duckdb");
    let registry = crate::cli::registry();
    let store = crate::cli::db::open(&db, &registry).unwrap();
    assert_eq!(
        run(&args_for(&db), crate::cli::registry()),
        ExitCode::FAILURE
    );
    let id = store
        .begin_run(crate::store::Mode::Crawl, "h", None, &[])
        .unwrap();
    store.finish_run(id).unwrap();
    drop(store);
    assert_eq!(run(&args_for(&db), registry), ExitCode::SUCCESS);
}

#[test]
fn opening_installs_the_reports_views_in_a_database_made_without_them() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("a.duckdb");
    let registry = crate::cli::registry();
    drop(Store::open(&db, &registry).unwrap());
    let store = crate::cli::db::open(&db, &registry).unwrap();
    for view in [
        "link_verdicts",
        "report_pages",
        "report_broken",
        "report_errors",
        "report_stats",
        "report_inbound",
        "report_roots",
        "report_sitemap",
        "report_redirects",
    ] {
        store
            .connection()
            .query_row(&format!("SELECT count(*) FROM {view}"), [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap_or_else(|e| panic!("{view}: {e}"));
    }
}
