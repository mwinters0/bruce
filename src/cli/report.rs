//! `bruce report <kind> [--db f] [--run n | --from n --to n] [--repo r]
//! [--fact k=v] [--kind k] [--top n] [--format f]`: one report over one run,
//! or `diff` over two (D6, D27; `roadmap-P4-reports.md`, "Which reports" and
//! "Which run a report reads").
//!
//! A report reads a database a crawl wrote and never creates one; opening it
//! installs the views, so a database from before them gains them here.
//! Finding rows is not a failure: the exit status is 1 only where the report
//! could not run.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};

use super::db;
use crate::extract::Kind;
use crate::profile::Registry;
use crate::report::{self, Format, Report, Request};
use crate::store::RunId;

/// The value of `--kind` that stands for every kind.
const ALL_KINDS: &str = "all";

/// What the command line asked of `report`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub request: Request,
    pub db: Option<PathBuf>,
    pub format: Format,
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        let name = m.get_one::<String>("report").expect("clap requires it");
        let report = Report::ALL
            .into_iter()
            .find(|r| r.name() == name)
            .expect("clap checked the name");
        let mut request = Request::new(report);
        let run = |name| m.get_one::<u64>(name).map(|&n| RunId(n));
        request.run = run("run");
        request.from = run("from");
        request.to = run("to");
        request.repo = m.get_one::<String>("repo").cloned();
        request.facts = m
            .get_many::<String>("fact")
            .into_iter()
            .flatten()
            .filter_map(|f| f.split_once('='))
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        let named: Vec<&str> = m
            .get_many::<String>("kind")
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect();
        if named.contains(&ALL_KINDS) {
            request.kinds = Kind::ALL.to_vec();
        } else if !named.is_empty() {
            request.kinds = Kind::ALL
                .into_iter()
                .filter(|k| named.contains(&k.as_str()))
                .collect();
        }
        request.top = m.get_one::<u64>("top").map(|&n| n as usize);
        let format = match m.get_one::<String>("format").map(String::as_str) {
            Some("json") => Format::Json,
            _ => Format::Text,
        };
        request.linking_pages = format == Format::Json;
        Args {
            request,
            db: m.get_one::<String>("db").map(PathBuf::from),
            format,
        }
    }
}

pub fn command() -> Command {
    let reports: Vec<&str> = Report::ALL.iter().map(|r| r.name()).collect();
    let mut kinds: Vec<&str> = Kind::ALL.iter().map(|k| k.as_str()).collect();
    kinds.push(ALL_KINDS);
    Command::new("report")
        .about("Print a report over one run of a database")
        .arg(
            Arg::new("report")
                .value_name("REPORT")
                .required(true)
                .value_parser(PossibleValuesParser::new(reports))
                .help("Which report to print"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .value_name("FILE")
                .help("The database to read [default: bruce.duckdb]"),
        )
        .arg(
            Arg::new("run")
                .long("run")
                .value_name("N")
                .value_parser(value_parser!(u64))
                .help("The run to read, finished or not [default: the latest finished]"),
        )
        .arg(
            Arg::new("from")
                .long("from")
                .value_name("N")
                .value_parser(value_parser!(u64))
                .help("`diff` only: the run to compare against [default: the finished run before --to]"),
        )
        .arg(
            Arg::new("to")
                .long("to")
                .value_name("N")
                .value_parser(value_parser!(u64))
                .help("`diff` only: the run to compare [default: the latest finished]"),
        )
        .arg(
            Arg::new("repo")
                .long("repo")
                .value_name("REPO")
                .help("Only pages whose source file is in this repo, or links on them"),
        )
        .arg(
            Arg::new("fact")
                .long("fact")
                .value_name("KEY=VALUE")
                .action(ArgAction::Append)
                .value_parser(fact)
                .help("Only pages with this fact, or links on them; repeat to require several"),
        )
        .arg(
            Arg::new("kind")
                .long("kind")
                .value_name("KIND")
                .action(ArgAction::Append)
                .value_parser(PossibleValuesParser::new(kinds))
                .help("Count links of this kind; repeat to add, `all` for every kind [default: anchor]"),
        )
        .arg(
            Arg::new("top")
                .long("top")
                .value_name("N")
                .value_parser(value_parser!(u64).range(1..))
                .help("Print only the first N rows, the worst first"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .value_parser(PossibleValuesParser::new(["text", "json"]))
                .default_value("text")
                .help("A text table, or one JSON array of objects"),
        )
}

/// The options that name a run, which `diff` and the other reports take
/// differently: `diff` reads `--from` and `--to`, the rest `--run`.
pub fn check(m: &ArgMatches) -> Result<(), clap::Error> {
    let diff = m.get_one::<String>("report").map(String::as_str) == Some("diff");
    let given = |name| m.contains_id(name);
    let misplaced = if diff {
        given("run").then_some((
            "--run",
            "diff compares two runs; name them with --from and --to",
        ))
    } else {
        ["from", "to"].into_iter().find(|&n| given(n)).map(|n| {
            (
                if n == "from" { "--from" } else { "--to" },
                "only diff compares two runs; the other reports read --run",
            )
        })
    };
    match misplaced {
        Some((arg, why)) => Err(command().error(
            clap::error::ErrorKind::ArgumentConflict,
            format!("{arg}: {why}"),
        )),
        None => Ok(()),
    }
}

/// `--fact`'s value: a key and a value around the first `=`.
fn fact(s: &str) -> Result<String, String> {
    match s.split_once('=') {
        Some((key, _)) if !key.is_empty() => Ok(s.to_owned()),
        _ => Err("expected KEY=VALUE, as in antora.component=fedora-docs".to_owned()),
    }
}

/// Runs `args` with the profiles `registry` holds, printing to stdout and
/// stderr.
pub fn run(args: &Args, registry: Registry) -> ExitCode {
    match report(args, &registry) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bruce report: {e}");
            ExitCode::FAILURE
        }
    }
}

fn report(args: &Args, registry: &Registry) -> Result<(), String> {
    let db = db::path(args.db.as_deref());
    if !db.exists() {
        return Err(format!("{}: no such database", db.display()));
    }
    let in_db = |e: &dyn std::fmt::Display| format!("{}: {e}", db.display());
    let store = db::open(db, registry).map_err(|e| in_db(&e))?;
    let output = report::run(&store, &args.request).map_err(|e| in_db(&e))?;
    let runs = output
        .from
        .into_iter()
        .chain([(output.run, output.unfinished)]);
    for (run, unfinished) in runs {
        if unfinished {
            eprintln!(
                "bruce report: run {} never finished, so this report is incomplete",
                run.0
            );
        }
    }
    let mut text = output.table.render(args.format);
    if args.format == Format::Text {
        for note in &output.notes {
            text.push_str("\n\n");
            text.push_str(note);
        }
    }
    // A reader that closes the pipe, as `head` does, has what it wanted.
    match writeln!(io::stdout().lock(), "{text}") {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => Err(e.to_string()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
