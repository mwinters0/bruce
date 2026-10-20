//! `bruce check-dir DIR --prefix URL [--config f] [--db f]`: one run over a
//! local build, written to the database (D1, D22, D27; `roadmap-P5-ci.md`,
//! "A `check-dir` run is a crawl").
//!
//! The configuration is the one `crawl` reads, so `skip`, the allowlist and
//! the roots' scopes plan a URL as they did for the baseline; with no file
//! the prefix is the only root, as a URL is for `crawl` (D40). The run
//! begins from the `.html` files of `DIR` alone ([`Build::seeds`]) and
//! answers a URL under the prefix from the directory. Nothing else is
//! answered yet: a URL outside it is an `error`, and no baseline, gate or
//! exit status other than 0 is in place.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, ArgMatches, Command};

use super::{crawl, db};
use crate::ci::Build;
use crate::crawl::{Crawl, Summary};
use crate::fetch::local::LocalFetcher;
use crate::profile::Registry;
use crate::store::{Mode, RunId, Writer};

/// Rows the writer's channel holds before a sender waits (D28).
const WRITER_CAPACITY: usize = 4096;

/// What the command line asked of `check-dir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub dir: PathBuf,
    pub prefix: String,
    pub config: Option<PathBuf>,
    pub db: Option<PathBuf>,
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        let one = |name| m.get_one::<String>(name).cloned();
        Args {
            dir: PathBuf::from(one("dir").expect("clap requires DIR")),
            prefix: one("prefix").expect("clap requires --prefix"),
            config: one("config").map(PathBuf::from),
            db: one("db").map(PathBuf::from),
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
}

/// Runs `args` with the profiles `registry` holds.
pub fn run(args: &Args, registry: Registry) -> ExitCode {
    match check(args, registry) {
        Ok((run, summary)) => {
            println!(
                "run {}: {} targets, {} pages",
                run.0, summary.targets, summary.pages
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("bruce check-dir: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Checks and writes one run of the build.
fn check(args: &Args, registry: Registry) -> Result<(RunId, Summary), String> {
    // With no file the prefix is the root, and a file names its own.
    let given = crawl::Args {
        url: args.config.is_none().then(|| args.prefix.clone()),
        config: args.config.clone(),
        db: None,
    };
    let (config, hash) = crawl::configure(&given, &registry)?;
    let build = Build::scan(&args.dir, &args.prefix, &config).map_err(|e| e.to_string())?;
    let db = db::path(args.db.as_deref());
    let in_db = |e: crate::store::Error| format!("{}: {e}", db.display());
    let store = db::open(db, &registry).map_err(in_db)?;
    let fetcher = LocalFetcher::new(build.prefix(), build.served());
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let run = store
        .begin_run(Mode::Local, &hash, None, &config.root_urls())
        .map_err(in_db)?;
    let writer = Writer::start(store, WRITER_CAPACITY);
    let crawl = Crawl::new(fetcher, config, registry, run, writer.handle())
        .local_site(build.prefix().clone());
    let crawled = runtime.block_on(crawl.run_seeded(build.seeds()));
    // The writer's own error says why a crawl lost it, so it goes first.
    let store = writer.finish().map_err(in_db)?;
    let summary = crawled.map_err(|e| e.to_string())?;
    store.finish_run(run).map_err(in_db)?;
    Ok((run, summary))
}

#[cfg(test)]
mod tests;
