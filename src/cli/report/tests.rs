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
fn the_two_run_reports_take_from_and_to_and_the_others_take_run() {
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
    for report in ["changed", "anchors-removed"] {
        assert!(check(&[report, "--to", "2"]).is_ok());
        let err = check(&[report, "--run", "1"]).unwrap_err().to_string();
        assert!(
            err.contains(&format!("{report} compares two runs")),
            "{err}"
        );
    }
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
    let code = run(&Job::Report(args_for(&db)), crate::cli::registry());
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
        run(&Job::Report(args_for(&db)), crate::cli::registry()),
        ExitCode::FAILURE
    );
    let id = store
        .begin_run(crate::store::Mode::Crawl, "h", None, &[])
        .unwrap();
    store.finish_run(id).unwrap();
    drop(store);
    assert_eq!(
        run(&Job::Report(args_for(&db)), registry),
        ExitCode::SUCCESS
    );
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

fn job(args: &[&str]) -> Result<Job, clap::Error> {
    let m =
        command().try_get_matches_from(std::iter::once("report").chain(args.iter().copied()))?;
    check(&m)?;
    Ok(Job::from(&m))
}

#[test]
fn the_digest_takes_a_directory_a_name_and_a_run() {
    let Job::Digest(d) = job(&["digest", "--out", "o", "--db", "d", "--run", "4"]).unwrap() else {
        panic!("a digest")
    };
    assert_eq!(
        d,
        DigestArgs {
            db: Some("d".into()),
            run: Some(RunId(4)),
            against: Compare::Previous,
            out: "o".into(),
            name: "bruce".into(),
            format: DigestFormat::Both,
        }
    );
    let Job::Digest(d) = job(&[
        "digest", "--out", "o", "--name", "en-us", "--format", "json",
    ])
    .unwrap() else {
        panic!("a digest")
    };
    assert_eq!((d.name.as_str(), d.run), ("en-us", None));
    assert_eq!(d.format, DigestFormat::Json);
    let Job::Digest(d) = job(&["digest", "--out", "o", "--format", "html"]).unwrap() else {
        panic!("a digest")
    };
    assert_eq!(d.format, DigestFormat::Html);
    assert!(matches!(job(&["broken"]).unwrap(), Job::Report(_)));
}

#[test]
fn the_digest_refuses_what_is_not_its_own() {
    assert!(job(&["digest"]).is_err(), "it needs --out");
    for bad in [
        ["--to", "1"],
        ["--repo", "r"],
        ["--fact", "a=b"],
        ["--kind", "image"],
        ["--top", "3"],
        ["--format", "text"],
        ["--name", ""],
        ["--name", "a/b"],
    ] {
        assert!(
            job(&["digest", "--out", "o", bad[0], bad[1]]).is_err(),
            "{bad:?}"
        );
    }
    for bad in [
        ["--out", "o"],
        ["--name", "n"],
        ["--format", "html"],
        ["--format", "both"],
        ["--since", "b.duckdb"],
    ] {
        assert!(job(&["broken", bad[0], bad[1]]).is_err(), "{bad:?}");
    }
}

#[test]
fn the_digest_compares_with_a_run_or_a_baseline_file_and_not_both() {
    let against = |extra: &[&str]| {
        let mut args = vec!["digest", "--out", "o"];
        args.extend(extra);
        let Job::Digest(d) = job(&args).unwrap() else {
            panic!("a digest")
        };
        d.against
    };
    assert_eq!(against(&[]), Compare::Previous);
    assert_eq!(against(&["--from", "3"]), Compare::Run(RunId(3)));
    assert_eq!(
        against(&["--since", "https://x.test/b.duckdb.gz"]),
        Compare::Since("https://x.test/b.duckdb.gz".into())
    );
    assert!(job(&["digest", "--out", "o", "--from", "3", "--since", "b"]).is_err());
}

#[test]
fn a_digest_since_a_baseline_file_compares_with_its_run_and_leaves_the_database_whole() {
    let dir = TempDir::new().unwrap();
    let registry = crate::cli::registry();
    let crawl = |name: &str| {
        let db = dir.path().join(name);
        let store = crate::cli::db::open(&db, &registry).unwrap();
        let id = store
            .begin_run(crate::store::Mode::Crawl, name, None, &[])
            .unwrap();
        store.finish_run(id).unwrap();
        (db, store, id)
    };
    let (yesterday, store, id) = crawl("yesterday.duckdb");
    let baseline = dir.path().join("baseline.duckdb");
    store
        .export_baseline(Some(id), &baseline, &registry)
        .unwrap();
    drop(store);
    let (today, store, _) = crawl("today.duckdb");
    drop(store);
    let args = DigestArgs {
        db: Some(today.clone()),
        run: None,
        against: Compare::Since(baseline.display().to_string()),
        out: dir.path().join("out"),
        name: "d".into(),
        format: DigestFormat::Json,
    };
    assert_eq!(
        run(&Job::Digest(args), crate::cli::registry()),
        ExitCode::SUCCESS
    );
    let written = std::fs::read_dir(dir.path().join("out"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    let json = std::fs::read_to_string(written.path()).unwrap();
    assert!(json.contains("\"from_file\":true"), "{json}");
    let store = crate::cli::db::open(&today, &registry).unwrap();
    let runs: i64 = store
        .connection()
        .query_row("SELECT count(*) FROM runs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(runs, 1, "the baseline is not left in the database");
    let _ = yesterday;
}

#[test]
fn a_digest_is_written_under_the_day_the_run_started() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("a.duckdb");
    let out = dir.path().join("archive").join("digests");
    let registry = crate::cli::registry();
    let store = crate::cli::db::open(&db, &registry).unwrap();
    let id = store
        .begin_run(crate::store::Mode::Crawl, "h", None, &[])
        .unwrap();
    store.finish_run(id).unwrap();
    drop(store);
    let args = DigestArgs {
        db: Some(db),
        run: None,
        against: Compare::Previous,
        out: out.clone(),
        name: "en-us".into(),
        format: DigestFormat::Both,
    };
    assert_eq!(
        run(&Job::Digest(args.clone()), crate::cli::registry()),
        ExitCode::SUCCESS
    );
    let mut files: Vec<_> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    files.sort();
    assert_eq!(files.len(), 2);
    let (html, json) = (&files[0], &files[1]);
    assert!(
        html.starts_with("en-us-20") && html.ends_with(".html"),
        "{files:?}"
    );
    assert_eq!(html.trim_end_matches("html"), json.trim_end_matches("json"));
    let text = std::fs::read_to_string(out.join(json)).unwrap();
    assert!(text.contains("\"format_version\": 1"));
    let page = std::fs::read_to_string(out.join(html)).unwrap();
    assert!(page.starts_with("<!doctype html>") && page.contains("<title>Link check digest 20"));
    // A second digest of the day replaces the first.
    assert_eq!(run(&Job::Digest(args.clone()), registry), ExitCode::SUCCESS);
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 2);
    // One format writes one file.
    let only = DigestArgs {
        out: dir.path().join("html"),
        format: DigestFormat::Html,
        ..args
    };
    assert_eq!(
        run(&Job::Digest(only.clone()), crate::cli::registry()),
        ExitCode::SUCCESS
    );
    let written: Vec<_> = std::fs::read_dir(&only.out).unwrap().collect();
    assert_eq!(written.len(), 1);
}

#[test]
fn a_digest_of_a_database_that_is_not_there_is_refused() {
    let dir = TempDir::new().unwrap();
    let args = DigestArgs {
        db: Some(dir.path().join("none.duckdb")),
        run: None,
        against: Compare::Previous,
        out: dir.path().join("out"),
        name: "bruce".into(),
        format: DigestFormat::Both,
    };
    assert_eq!(
        run(&Job::Digest(args), crate::cli::registry()),
        ExitCode::FAILURE
    );
    assert!(!dir.path().join("out").exists());
}
