//! `bruce report <kind> [--db f] [--run n | --from n --to n] [--repo r]
//! [--fact k=v] [--kind k] [--top n] [--format f]`: one report over one run,
//! or `diff` over two (D6, D27): by default the latest finished run, and for
//! `diff` the two latest. `bruce report digest --out dir [--name s] [--db f]
//! [--run n] [--format json|html|both]` writes a crawl run's dated digest
//! instead (D64).
//!
//! A report reads a database a crawl wrote and never creates one; opening it
//! installs the views, so a database from before them gains them here.
//! Finding rows is not a failure: the exit status is 1 only where the report
//! could not run.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};

use super::db;
use crate::extract::Kind;
use crate::profile::Registry;
use crate::report::{self, Digest, Format, Report, Request};
use crate::store::{RunId, Store};

/// The value of `REPORT` that writes the digest, which is no `Report`: it
/// makes files and not a table.
const DIGEST: &str = "digest";

/// The digest's file stem when `--name` gives none.
const STEM: &str = "bruce";

/// The value of `--kind` that stands for every kind.
const ALL_KINDS: &str = "all";

/// What the command line asked of `report`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub request: Request,
    pub db: Option<PathBuf>,
    pub format: Format,
}

/// What the command line asked of `report`: a report, or the digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    Report(Args),
    Digest(DigestArgs),
}

impl From<&ArgMatches> for Job {
    fn from(m: &ArgMatches) -> Job {
        match m.get_one::<String>("report").map(String::as_str) {
            Some(DIGEST) => Job::Digest(DigestArgs::from(m)),
            _ => Job::Report(Args::from(m)),
        }
    }
}

/// Which of the digest's files `report digest` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestFormat {
    Json,
    Html,
    Both,
}

impl DigestFormat {
    fn json(self) -> bool {
        self != DigestFormat::Html
    }

    fn html(self) -> bool {
        self != DigestFormat::Json
    }
}

/// What the command line asked of `report digest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestArgs {
    pub db: Option<PathBuf>,
    pub run: Option<RunId>,
    pub out: PathBuf,
    pub name: String,
    pub format: DigestFormat,
}

impl From<&ArgMatches> for DigestArgs {
    fn from(m: &ArgMatches) -> DigestArgs {
        DigestArgs {
            db: m.get_one::<String>("db").map(PathBuf::from),
            run: m.get_one::<u64>("run").map(|&n| RunId(n)),
            out: m
                .get_one::<String>("out")
                .map(PathBuf::from)
                .expect("`check` requires it"),
            name: m
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_else(|| STEM.to_owned()),
            format: match m.get_one::<String>("format").map(String::as_str) {
                Some("json") => DigestFormat::Json,
                Some("html") => DigestFormat::Html,
                _ => DigestFormat::Both,
            },
        }
    }
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        let name = m.get_one::<String>("report").expect("clap requires it");
        let report = Report::ALL
            .into_iter()
            .find(|r| r.name() == name)
            .expect("`Job` sends a report here, and clap checked the name");
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
        if let Some(kinds) = kinds(m) {
            request.kinds = kinds;
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

/// The `--kind` option.
fn kind_arg() -> Arg {
    let mut kinds: Vec<&str> = Kind::ALL.iter().map(|k| k.as_str()).collect();
    kinds.push(ALL_KINDS);
    Arg::new("kind")
        .long("kind")
        .value_name("KIND")
        .action(ArgAction::Append)
        .value_parser(PossibleValuesParser::new(kinds))
        .help(
            "Count links of this kind; repeat to add, `all` for every kind \
             [default: anchor, and every kind for `introduced` and `inverse`]",
        )
}

/// The kinds `--kind` names, or none where it was not given.
fn kinds(m: &ArgMatches) -> Option<Vec<Kind>> {
    let named: Vec<&str> = m
        .get_many::<String>("kind")
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect();
    if named.contains(&ALL_KINDS) {
        Some(Kind::ALL.to_vec())
    } else if named.is_empty() {
        None
    } else {
        Some(
            Kind::ALL
                .into_iter()
                .filter(|k| named.contains(&k.as_str()))
                .collect(),
        )
    }
}

pub fn command() -> Command {
    let mut reports: Vec<&str> = Report::ALL.iter().map(|r| r.name()).collect();
    reports.push(DIGEST);
    Command::new("report")
        .about("Print a report over one run of a database, or write its digest")
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
            Arg::new("out")
                .long("out")
                .value_name("DIR")
                .help("`digest` only: the directory the files are written to"),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .value_name("STEM")
                .value_parser(stem)
                .help("`digest` only: the file name's first part [default: bruce]"),
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
        .arg(kind_arg())
        .arg(
            Arg::new("top")
                .long("top")
                .value_name("N")
                .value_parser(value_parser!(u64).range(1..))
                .help("Print only the first N rows, in the report's order"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .value_parser(PossibleValuesParser::new(["text", "json", "html", "both"]))
                .help(
                    "A text table, or one JSON array of objects [default: text]; \
                     `digest` writes `json`, `html` or `both` [default: both]",
                ),
        )
}

/// The options a report or the digest does not take, and the ones that name
/// a run, which `diff` and the other reports take differently: `diff` reads
/// `--from` and `--to`, the rest `--run`.
pub fn check(m: &ArgMatches) -> Result<(), clap::Error> {
    let name = m.get_one::<String>("report").map(String::as_str);
    let given = |name| m.contains_id(name);
    let refuse = |arg: &str, why: &str| {
        Err(command().error(
            clap::error::ErrorKind::ArgumentConflict,
            format!("{arg}: {why}"),
        ))
    };
    if name == Some(DIGEST) {
        let whole =
            "the digest is of a whole run; it takes --run, --db, --out, --name and --format";
        for (id, flag) in [
            ("from", "--from"),
            ("to", "--to"),
            ("repo", "--repo"),
            ("fact", "--fact"),
            ("kind", "--kind"),
            ("top", "--top"),
        ] {
            if given(id) {
                return refuse(flag, whole);
            }
        }
        if m.get_one::<String>("format").map(String::as_str) == Some("text") {
            return refuse("--format", "the digest is written as json, html or both");
        }
        return if given("out") {
            Ok(())
        } else {
            Err(command().error(
                clap::error::ErrorKind::MissingRequiredArgument,
                "digest needs --out DIR, where its files are written",
            ))
        };
    }
    for (id, flag) in [("out", "--out"), ("name", "--name")] {
        if given(id) {
            return refuse(flag, "only digest writes files");
        }
    }
    if let Some(format @ ("html" | "both")) = m.get_one::<String>("format").map(String::as_str) {
        return refuse(
            "--format",
            &format!("`{format}` is the digest's; a report is text or json"),
        );
    }
    if name == Some("diff") {
        if given("run") {
            return refuse(
                "--run",
                "diff compares two runs; name them with --from and --to",
            );
        }
    } else if let Some(flag) = [("from", "--from"), ("to", "--to")]
        .into_iter()
        .find_map(|(id, flag)| given(id).then_some(flag))
    {
        return refuse(
            flag,
            "only diff compares two runs; the other reports read --run",
        );
    }
    Ok(())
}

/// `--name`'s value: a file name's first part, which holds no separator.
fn stem(s: &str) -> Result<String, String> {
    if s.is_empty() || s.contains(['/', '\\']) {
        Err("expected a file name without a `/`, as in en-us".to_owned())
    } else {
        Ok(s.to_owned())
    }
}

/// `--fact`'s value: a key and a value around the first `=`.
fn fact(s: &str) -> Result<String, String> {
    match s.split_once('=') {
        Some((key, _)) if !key.is_empty() => Ok(s.to_owned()),
        _ => Err("expected KEY=VALUE, as in antora.component=fedora-docs".to_owned()),
    }
}

/// Runs `job` with the profiles `registry` holds, printing to stdout and
/// stderr.
pub fn run(job: &Job, registry: Registry) -> ExitCode {
    let done = match job {
        Job::Report(args) => report(args, &registry),
        Job::Digest(args) => digest(args, &registry),
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bruce report: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The database `db` names, which must exist, and its path as the messages
/// name it.
fn open(db: Option<&Path>, registry: &Registry) -> Result<(Store, PathBuf), String> {
    let db = db::path(db);
    if !db.exists() {
        return Err(format!("{}: no such database", db.display()));
    }
    let store = db::open(db, registry).map_err(|e| format!("{}: {e}", db.display()))?;
    Ok((store, db.to_owned()))
}

/// Says on stderr that a later run stopped before the end, and which of
/// `runs` never finished, so what was read is incomplete.
fn warn_incomplete(
    reading: RunId,
    later_unfinished: Option<RunId>,
    runs: impl IntoIterator<Item = (RunId, bool)>,
) {
    if let Some(later) = later_unfinished {
        eprintln!(
            "bruce report: run {} stopped before the end; reporting run {}",
            later.0, reading.0
        );
    }
    for (run, unfinished) in runs {
        if unfinished {
            eprintln!(
                "bruce report: run {} never finished, so this report is incomplete",
                run.0
            );
        }
    }
}

fn report(args: &Args, registry: &Registry) -> Result<(), String> {
    let (store, db) = open(args.db.as_deref(), registry)?;
    let output =
        report::run(&store, &args.request).map_err(|e| format!("{}: {e}", db.display()))?;
    warn_incomplete(
        output.run,
        output.later_unfinished,
        output
            .from
            .into_iter()
            .chain([(output.run, output.unfinished)]),
    );
    let mut text = output.table.render(args.format);
    if args.format == Format::Text {
        for note in &output.notes {
            text.push_str("\n\n");
            text.push_str(note);
        }
    }
    say(&text)
}

/// Writes the digest of the run named, as `DIR/STEM-YYYY-MM-DD.json` and
/// `.html`, and prints the path of each (D64).
fn digest(args: &DigestArgs, registry: &Registry) -> Result<(), String> {
    let (store, db) = open(args.db.as_deref(), registry)?;
    let digest: Digest =
        report::digest(&store, args.run).map_err(|e| format!("{}: {e}", db.display()))?;
    warn_incomplete(
        digest.run.id,
        digest.later_unfinished,
        [(digest.run.id, digest.unfinished)],
    );
    let io_error = |path: &Path, e: io::Error| format!("{}: {e}", path.display());
    fs::create_dir_all(&args.out).map_err(|e| io_error(&args.out, e))?;
    let file = |extension| {
        args.out
            .join(format!("{}-{}.{extension}", args.name, digest.date()))
    };
    // The files are rendered before either is written, so a failing one
    // leaves no half of the pair.
    let mut files = Vec::new();
    if args.format.json() {
        files.push((file("json"), digest.json()));
    }
    if args.format.html() {
        files.push((file("html"), digest.html()));
    }
    for (path, text) in files {
        fs::write(&path, text).map_err(|e| io_error(&path, e))?;
        say(&path.display().to_string())?;
    }
    Ok(())
}

/// Prints `text` and a newline. A reader that closes the pipe, as `head`
/// does, has what it wanted.
fn say(text: &str) -> Result<(), String> {
    match writeln!(io::stdout().lock(), "{text}") {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => Err(e.to_string()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
