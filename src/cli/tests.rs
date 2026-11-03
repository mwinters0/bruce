use std::process::ExitCode;

use crate::config::ProfileChoice;
use crate::store::{Mode, Store};

use super::crawl::Args;
use super::{registry, registry_for};

/// `antora_pages`' columns.
type Pivot = (
    u64,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[test]
fn a_command_line_clap_refuses_exits_2_and_help_exits_0() {
    assert_eq!(super::run_from(["bruce"]), ExitCode::from(2));
    assert_eq!(super::run_from(["bruce", "nosuch"]), ExitCode::from(2));
    assert_eq!(
        super::run_from(["bruce", "crawl", "a", "b"]),
        ExitCode::from(2)
    );
    assert_eq!(super::run_from(["bruce", "--help"]), ExitCode::SUCCESS);
    assert_eq!(
        super::run_from(["bruce", "crawl", "--help"]),
        ExitCode::SUCCESS
    );
}

#[test]
fn crawl_takes_a_url_and_two_files() {
    let m = super::command()
        .try_get_matches_from([
            "bruce",
            "crawl",
            "https://x.org/",
            "--db",
            "d",
            "--config",
            "c",
        ])
        .unwrap();
    let (_, m) = m.subcommand().unwrap();
    assert_eq!(
        Args::from(m),
        Args {
            url: Some("https://x.org/".into()),
            config: Some("c".into()),
            db: Some("d".into()),
        }
    );
}

#[test]
fn the_command_line_is_consistent() {
    super::command().debug_assert();
}

#[test]
fn auto_picks_antora_by_its_generator_and_generic_otherwise() {
    let r = registry();
    let pick = |g| r.choose(&ProfileChoice::Auto, g).unwrap().name();
    assert_eq!(pick(Some("Antora 3.1.15")), "antora");
    assert_eq!(pick(Some("Docusaurus v4.0.0")), "generic");
    assert_eq!(pick(None), "generic");
}

#[test]
fn every_view_is_named_for_its_profile_and_creates() {
    let r = registry();
    for profile in r.iter() {
        for view in profile.views() {
            assert!(view.name.starts_with(&format!("{}_", profile.name())));
        }
    }
    Store::open_in_memory(&r).unwrap();
}

#[test]
fn the_antora_view_pivots_a_pages_facts() {
    let store = Store::open_in_memory(&registry()).unwrap();
    let run = store
        .begin_run(
            Mode::Crawl,
            "abc",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
    let conn = store.connection();
    for (url_id, key, value) in [
        (1, "antora.component", "handbook"),
        (1, "antora.version", "f43"),
        (1, "antora.module", "ROOT"),
        (1, "antora.rel_path", "index.adoc"),
        (2, "antora.component", "arm-sbc"),
        (2, "antora.rel_path", "index.adoc"),
        (3, "other.component", "ignored"),
    ] {
        conn.execute(
            "INSERT INTO page_facts VALUES (?, ?, ?, ?)",
            duckdb::params![run.0, url_id, key, value],
        )
        .unwrap();
    }
    let mut stmt = conn
        .prepare(
            "SELECT url_id, component, version, module, rel_path \
             FROM antora_pages ORDER BY url_id",
        )
        .unwrap();
    let rows: Vec<Pivot> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let s = |v: &str| Some(v.to_string());
    assert_eq!(
        rows,
        [
            (1, s("handbook"), s("f43"), s("ROOT"), s("index.adoc")),
            (2, s("arm-sbc"), None, None, s("index.adoc")),
        ]
    );
}

#[test]
fn a_defined_profile_is_tried_under_auto_before_the_compiled_ones() {
    let defined = crate::config::Config::parse(
        "[[profile]]\nname = \"mine\"\ngenerator = \"^Antora\"\n[[profile]]\nname = \"other\"\ngenerator = \"^Antora\"\n",
    )
    .unwrap()
    .policy
    .profiles;
    let r = registry_for(&defined).unwrap();
    let names: Vec<&str> = r.iter().map(|p| p.name()).collect();
    assert_eq!(names, ["generic", "mine", "other", "antora"]);
    let pick = |g| r.choose(&ProfileChoice::Auto, g).unwrap().name();
    assert_eq!(pick(Some("Antora 3")), "mine");
    assert_eq!(pick(Some("Hugo")), "generic");
    for profile in r.iter() {
        for view in profile.views() {
            assert!(view.name.starts_with(&format!("{}_", profile.name())));
        }
    }
    Store::open_in_memory(&r).unwrap();
}

#[test]
fn a_profile_extends_a_compiled_one_and_nothing_else() {
    let build = |toml: &str| {
        let defined = crate::config::Config::parse(toml).unwrap().policy.profiles;
        registry_for(&defined)
    };
    let r = build("[[profile]]\nname = \"mine\"\nextends = \"antora\"\n").unwrap();
    let names: Vec<&str> = r.iter().map(|p| p.name()).collect();
    assert_eq!(names, ["generic", "mine", "antora"]);
    let pick = |g| r.choose(&ProfileChoice::Auto, g).unwrap().name();
    assert_eq!(pick(Some("Antora 3")), "mine");
    assert_eq!(pick(Some("Hugo")), "generic");
    Store::open_in_memory(&r).unwrap();
    let e = build("[[profile]]\nname = \"a\"\n[[profile]]\nname = \"b\"\nextends = \"a\"\n")
        .err()
        .unwrap();
    assert!(
        e.starts_with("profile.b: extends: `a` is not a compiled profile"),
        "{e}"
    );
}

/// A finished run in a database at `path`, recording schema version `version`.
fn database_of_version(path: &std::path::Path, version: u32) {
    let store = Store::open(path, &registry()).unwrap();
    let run = store
        .begin_run(
            Mode::Crawl,
            "abc",
            None,
            &[],
            &crate::config::Policy::default(),
        )
        .unwrap();
    store.finish_run(run).unwrap();
    store
        .connection()
        .execute(
            "UPDATE meta SET value = ? WHERE key = 'schema_version'",
            [version.to_string()],
        )
        .unwrap();
}

#[test]
fn every_command_exits_3_on_a_database_of_another_schema_version() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("old.duckdb");
    database_of_version(&db, crate::store::SCHEMA_VERSION + 1);
    let db = db.to_str().unwrap();
    let build = tmp.path().join("build");
    std::fs::create_dir(&build).unwrap();
    let build = build.to_str().unwrap();
    let out = tmp.path().join("out.duckdb");
    let out = out.to_str().unwrap();
    // A baseline of this build's schema, so only the database is wrong.
    let baseline = tmp.path().join("baseline.duckdb");
    database_of_version(&baseline, crate::store::SCHEMA_VERSION);
    let baseline = baseline.to_str().unwrap();
    let go = |args: &[&str]| super::run_from(["bruce"].into_iter().chain(args.iter().copied()));
    for (name, args) in [
        ("crawl", vec!["crawl", "http://127.0.0.1:9/", "--db", db]),
        ("prune", vec!["prune", "--keep", "1", "--db", db]),
        ("report", vec!["report", "stats", "--db", db]),
        (
            "export-baseline",
            vec!["export-baseline", "--out", out, "--db", db],
        ),
        (
            "check-dir",
            vec![
                "check-dir",
                build,
                "--prefix",
                "http://127.0.0.1:9/",
                "--baseline",
                baseline,
                "--db",
                db,
            ],
        ),
    ] {
        assert_eq!(go(&args), ExitCode::from(3), "{name}");
    }
}

#[test]
fn a_baseline_of_another_schema_version_exits_3_whoever_reads_it() {
    let tmp = tempfile::tempdir().unwrap();
    let old = tmp.path().join("old.duckdb");
    database_of_version(&old, crate::store::SCHEMA_VERSION - 1);
    let current = tmp.path().join("current.duckdb");
    database_of_version(&current, crate::store::SCHEMA_VERSION);
    let build = tmp.path().join("build");
    std::fs::create_dir(&build).unwrap();
    let (old, current, build) = (
        old.to_str().unwrap(),
        current.to_str().unwrap(),
        build.to_str().unwrap(),
    );
    let fresh = tmp.path().join("fresh.duckdb");
    let fresh = fresh.to_str().unwrap();
    let go = |args: &[&str]| super::run_from(["bruce"].into_iter().chain(args.iter().copied()));
    assert_eq!(
        go(&[
            "check-dir",
            build,
            "--prefix",
            "http://127.0.0.1:9/",
            "--baseline",
            old,
            "--db",
            fresh,
        ]),
        ExitCode::from(3)
    );
    let out = tmp.path().join("out");
    assert_eq!(
        go(&[
            "report",
            "digest",
            "--db",
            current,
            "--since",
            old,
            "--out",
            out.to_str().unwrap(),
        ]),
        ExitCode::from(3)
    );
}

/// The `bruce` command lines of the deployment guide's tagged scripts and CI
/// workflow, whose `run:` blocks are scripts too (D42): a logical line (continuations joined) that starts with `bruce`, cut at the
/// shell's `||`, `&&`, `;`, `|` or a redirection, each `$`-word a stand-in
/// number that every option's value accepts.
fn guide_command_lines() -> Vec<Vec<String>> {
    let guide = include_str!("../../docs/manual/deploy.md");
    let mut lines = Vec::new();
    let mut tagged = false;
    let mut joined = String::new();
    for line in guide.lines() {
        if let Some(info) = line.strip_prefix("```") {
            tagged = !tagged
                && info.contains("file=")
                && (info.starts_with("bash") || info.starts_with("yaml"));
            continue;
        }
        if !tagged {
            continue;
        }
        joined.push_str(line.trim_end_matches('\\'));
        if line.ends_with('\\') {
            continue;
        }
        let logical = std::mem::take(&mut joined);
        if !logical.trim_start().starts_with("bruce ") {
            continue;
        }
        let mut words = Vec::new();
        for word in logical.split_whitespace() {
            if matches!(word, "||" | "&&" | ";" | "|" | ">" | "2>" | ">&2") {
                break;
            }
            words.push(if word.contains('$') {
                "1".to_string()
            } else {
                word.trim_matches('"').to_string()
            });
        }
        lines.push(words);
    }
    lines
}

#[test]
fn every_bruce_line_of_the_guides_scripts_is_a_command_line_bruce_accepts() {
    let lines = guide_command_lines();
    // Crawl, export, two digests, prune, and the CI workflow's `check-dir`: a
    // script rewritten so that this reads none has not been checked.
    assert!(lines.len() >= 6, "found {lines:?}");
    assert!(
        lines
            .iter()
            .any(|l| l.get(1).map(String::as_str) == Some("check-dir")),
        "the CI workflow's check-dir line is not read: {lines:?}"
    );
    for line in lines {
        if let Err(e) = super::accepts(&line) {
            panic!("{}\nthe guide's script runs `{}`", e, line.join(" "));
        }
    }
}

#[test]
fn accepts_refuses_what_run_would() {
    assert!(super::accepts(["bruce", "prune", "--keep", "30"]).is_ok());
    assert!(super::accepts(["bruce", "prune"]).is_err());
    assert!(super::accepts(["bruce", "crawl", "--nosuch"]).is_err());
    assert!(
        super::accepts([
            "bruce", "report", "digest", "--out", "d", "--format", "text"
        ])
        .is_err()
    );
}
