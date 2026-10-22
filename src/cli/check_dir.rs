//! `bruce check-dir DIR --prefix URL --baseline FILE|URL [--config f] [--db f]
//! [--externals skip|baseline|live] [--inverse fail|warn] [--format text|json] [--all]`:
//! one run over a local build, written to the database and judged against the
//! baseline (D1, D9, D22, D27).
//!
//! The configuration is the one `crawl` reads, so `skip`, the allowlist and
//! the roots' scopes plan a URL as they did for the baseline; with no file
//! the prefix is the only root, as a URL is for `crawl` (D40). The baseline
//! file is imported as a run of the database ([`file`], D52) and the CI run
//! names it. The run begins from every file of `DIR`
//! ([`Build::seeds`]) and answers a URL under the prefix from the directory;
//! a URL the build lacks is `broken_url` (`not_built`) in a component-version
//! it holds, else answered from the baseline or asked live, as `crawl`'s
//! `baseline` module has it. The gate is `report`'s ([`Gate`]) over
//! the finished run, and sets the exit status: 0 where nothing is introduced,
//! 1 where a link is introduced or, under `--inverse=fail`, the change removed
//! what a page outside the build links to, 2 where the tool could not run (D16: an `error` moves
//! nothing). A SIGINT or SIGTERM stops the run as it does `crawl`'s (D58): a
//! build cut short is not judged, so no gate runs, the run is left unfinished
//! and the exit status is 128 plus the signal.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgMatches, Command};

use super::interrupt::{self, Pressed};
use super::{crawl, db};
use crate::ci::Build;
use crate::crawl::{Baseline, Crawl, Externals, Summary};
use crate::fetch::http::host::Hosts;
use crate::fetch::http::{HttpFetcher, Settings};
use crate::fetch::local::LocalFetcher;
use crate::fetch::routed::Routed;
use crate::profile::Registry;
use crate::report::{Format, Gate, InverseMode, Report, Request};
use crate::store::{Mode, RunId, Writer};

mod file;

use file::BaselineFile;

/// The exit status of a run the tool could not make, as a command line `clap`
/// refuses has it.
const COULD_NOT_RUN: u8 = 2;

/// Rows the writer's channel holds before a sender waits (D28).
const WRITER_CAPACITY: usize = 4096;

/// What the command line asked of `check-dir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub dir: PathBuf,
    pub prefix: String,
    /// A path or a URL.
    pub baseline: String,
    pub externals: Externals,
    /// Whether an inverse breakage fails the check (D9).
    pub inverse: InverseMode,
    pub config: Option<PathBuf>,
    pub db: Option<PathBuf>,
    pub format: Format,
    /// Print the breakage already in the baseline in full.
    pub all: bool,
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        let one = |name| m.get_one::<String>(name).cloned();
        Args {
            dir: PathBuf::from(one("dir").expect("clap requires DIR")),
            prefix: one("prefix").expect("clap requires --prefix"),
            baseline: one("baseline").expect("clap requires --baseline"),
            externals: match one("externals").as_deref() {
                Some("skip") => Externals::Skip,
                Some("live") => Externals::Live,
                _ => Externals::Baseline,
            },
            inverse: match one("inverse").as_deref() {
                Some("warn") => InverseMode::Warn,
                _ => InverseMode::Fail,
            },
            config: one("config").map(PathBuf::from),
            db: one("db").map(PathBuf::from),
            format: match one("format").as_deref() {
                Some("json") => Format::Json,
                _ => Format::Text,
            },
            all: m.get_flag("all"),
        }
    }
}

pub fn command() -> Command {
    Command::new("check-dir")
        .about("Check a local build directory and record every target, page, anchor and link in a new run")
        .arg(
            Arg::new("dir")
                .value_name("DIR")
                .required(true)
                .help("The build directory"),
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .value_name("URL")
                .required(true)
                .help("The production URL of the build directory's root"),
        )
        .arg(
            Arg::new("baseline")
                .long("baseline")
                .value_name("FILE|URL")
                .required(true)
                .help("The baseline file `bruce export-baseline` wrote, gzipped or not"),
        )
        .arg(
            Arg::new("externals")
                .long("externals")
                .value_name("MODE")
                .value_parser(["skip", "baseline", "live"])
                .default_value("baseline")
                .help(
                    "Allowlisted URLs: never ask (the baseline's verdict where it has one, else \
                     unchecked), take the baseline's verdict where it has one and ask \
                     otherwise, or always ask",
                ),
        )
        .arg(
            Arg::new("inverse")
                .long("inverse")
                .value_name("MODE")
                .value_parser(["fail", "warn"])
                .default_value("fail")
                .help(
                    "Whether a page outside the build linking to what the change removed fails \
                     the check, or is only listed",
                ),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("The configuration the baseline was crawled under, in TOML"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .value_name("FILE")
                .help("The database to add the run to [default: bruce.duckdb]"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .value_parser(["text", "json"])
                .default_value("text")
                .help("A text report, or one JSON object"),
        )
        .arg(
            Arg::new("all")
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Print the breakage already in the baseline in full"),
        )
}

/// Runs `args` with the profiles `registry` holds.
pub fn run(args: &Args, registry: Registry) -> ExitCode {
    match check(args, registry) {
        Ok(Checked::Judged(run, summary, gate)) => {
            let line = format!(
                "run {}: {} targets, {} pages",
                run.0, summary.targets, summary.pages
            );
            // JSON is one object on stdout, so the line goes beside it.
            let all = args.all;
            match args.format {
                Format::Text => println!("{line}\n{}", gate.text(all)),
                Format::Json => {
                    eprintln!("bruce check-dir: {line}");
                    println!("{}", gate.json(all));
                }
            }
            if gate.fails() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Ok(Checked::Unjudged(run, how, pressed)) => {
            eprintln!(
                "bruce check-dir: run {}: {how}, so the build was not judged and the run is left \
                 unfinished",
                run.0
            );
            pressed.exit()
        }
        Err(e) => {
            eprintln!("bruce check-dir: {e}");
            ExitCode::from(COULD_NOT_RUN)
        }
    }
}

/// What a check came to.
enum Checked {
    Judged(RunId, Summary, Box<Gate>),
    /// A signal ended the run first, and how, so the gate has nothing to say.
    Unjudged(RunId, &'static str, Pressed),
}

/// Checks and writes one run of the build, and judges it against its
/// baseline unless a signal came first.
fn check(args: &Args, registry: Registry) -> Result<Checked, String> {
    // With no file the prefix is the root, and a file names its own.
    let given = crawl::Args {
        url: args.config.is_none().then(|| args.prefix.clone()),
        config: args.config.clone(),
        db: None,
    };
    let (config, hash) = crawl::configure(&given, &registry)?;
    let build = Build::scan(&args.dir, &args.prefix, &config).map_err(|e| e.to_string())?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let baseline_file = BaselineFile::open(&args.baseline, &config, &runtime)?;
    let db = db::path(args.db.as_deref());
    let in_db = |e: crate::store::Error| format!("{}: {e}", db.display());
    let store = db::open(db, &registry).map_err(in_db)?;
    let imported = store
        .import_baseline(baseline_file.path())
        .map_err(|e| format!("{}: {e}", args.baseline))?;
    let index = store.baseline_index(imported.run).map_err(in_db)?;
    let settings = Settings {
        user_agent: config.user_agent.clone(),
        ..Settings::default()
    };
    let live = HttpFetcher::new(settings, Hosts::new(&config)).map_err(|e| e.to_string())?;
    let local = LocalFetcher::new(build.prefix(), build.served());
    let site = local.site();
    let fetcher = Routed::new(local, live);
    let run = store
        .begin_run(Mode::Local, &hash, Some(imported.run), &config.root_urls())
        .map_err(in_db)?;
    let writer = Writer::start(store, WRITER_CAPACITY);
    let baseline = Baseline {
        run: imported.run,
        index,
        externals: args.externals,
    };
    let crawl = Crawl::new(fetcher, config, registry, run, writer.handle()).against(site, baseline);
    let stop = crawl.stopper();
    let writes = crawl.writes();
    // An abandoned run's tasks are not waited for, but its writes begun are (D49).
    let (crawled, pressed) = interrupt::drive(
        runtime,
        "check-dir",
        stop,
        &writes,
        crawl.run_seeded(build.seeds()),
    );
    // The writer's own error says why a crawl lost it, so it goes first.
    let store = writer.finish().map_err(in_db)?;
    let Some(crawled) = crawled else {
        return Ok(Checked::Unjudged(
            run,
            "abandoned at the second signal",
            pressed,
        ));
    };
    let summary = crawled.map_err(|e| e.to_string())?;
    if summary.stopped {
        return Ok(Checked::Unjudged(run, "stopped before the end", pressed));
    }
    store.finish_run(run).map_err(in_db)?;
    let mut request = Request::new(Report::Introduced);
    request.run = Some(run);
    request.inverse = args.inverse;
    let gate = crate::report::gate(&store, &request).map_err(|e| e.to_string())?;
    Ok(Checked::Judged(run, summary, Box::new(gate)))
}

#[cfg(test)]
mod tests;
