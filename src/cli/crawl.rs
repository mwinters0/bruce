//! `bruce crawl [URL] [--config f] [--db f]`: one run over HTTP, written to
//! the database (D2, D27; `roadmap-P3-crawler.md`, "The command line" and
//! "An interrupted run").
//!
//! The roots are the file's or the URL, never both: a URL beside a file
//! naming roots is refused rather than replacing them, which would drop their
//! settings silently, and a file naming none takes the URL as its only root,
//! its other settings applying (D40). A root's profile must be one the
//! registry holds, which only this layer can check (D39). The first Ctrl-C
//! stops the crawl, whose requests end within [`STOP_GRACE`] (`crawl`'s
//! doc), and flushes the writer, leaving `runs.finished_at` NULL; a second abandons the
//! requests still out, waits for the targets already being written, and
//! flushes the writer, so the run holds whole targets either way (D49). No
//! run resumes: the next starts afresh.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgMatches, Command};
use xxhash_rust::xxh3::xxh3_64;

use crate::config::{Config, ProfileChoice, Root};
use crate::crawl::{Crawl, STOP_GRACE, Stopper, Summary};
use crate::fetch::http::host::Hosts;
use crate::fetch::http::{HttpFetcher, Settings};
use crate::profile::Registry;
use crate::store::{Mode, RunId, Store, Writer};

/// The database a run writes when `--db` names none.
pub const DEFAULT_DB: &str = "bruce.duckdb";

/// Rows the writer's channel holds before a sender waits (D28).
const WRITER_CAPACITY: usize = 4096;

/// The exit status of a run stopped by Ctrl-C, the shell's for SIGINT.
const INTERRUPTED: u8 = 130;

/// What the command line asked of `crawl`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    pub url: Option<String>,
    pub config: Option<PathBuf>,
    pub db: Option<PathBuf>,
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        Args {
            url: m.get_one::<String>("url").cloned(),
            config: m.get_one::<String>("config").map(PathBuf::from),
            db: m.get_one::<String>("db").map(PathBuf::from),
        }
    }
}

pub fn command() -> Command {
    Command::new("crawl")
        .about("Crawl the roots and record every target, page, anchor and link in a new run")
        .arg(
            Arg::new("url")
                .value_name("URL")
                .help("The only root, when the file names none or there is no file"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("The configuration, in TOML"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .value_name("FILE")
                .help("The database to add the run to [default: bruce.duckdb]"),
        )
}

/// Runs `args` with the profiles `registry` holds, reporting to stdout and
/// stderr.
pub fn run(args: &Args, registry: Registry) -> ExitCode {
    match crawl(args, registry) {
        Ok((run, Some(summary))) => {
            println!("{}", report(run, &summary));
            if summary.stopped {
                ExitCode::from(INTERRUPTED)
            } else {
                ExitCode::SUCCESS
            }
        }
        Ok((run, None)) => {
            println!(
                "run {}: abandoned at the second Ctrl-C, so the run is left unfinished",
                run.0
            );
            ExitCode::from(INTERRUPTED)
        }
        Err(e) => {
            eprintln!("bruce crawl: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Crawls and writes one run, finishing it unless it was stopped. No summary
/// means the second Ctrl-C abandoned the crawl.
fn crawl(args: &Args, registry: Registry) -> Result<(RunId, Option<Summary>), String> {
    let (config, hash) = configure(args, &registry)?;
    let db = args.db.as_deref().unwrap_or(Path::new(DEFAULT_DB));
    let in_db = |e: crate::store::Error| format!("{}: {e}", db.display());
    let store = Store::open(db, &registry).map_err(in_db)?;
    let settings = Settings {
        user_agent: config.user_agent.clone(),
        ..Settings::default()
    };
    let fetcher = HttpFetcher::new(settings, Hosts::new(&config)).map_err(|e| e.to_string())?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    // Nothing after this fails before the crawl, so no run is begun idle.
    let run = store.begin_run(Mode::Crawl, &hash, None).map_err(in_db)?;
    let writer = Writer::start(store, WRITER_CAPACITY);
    let crawl = Crawl::new(fetcher, config, registry, run, writer.handle());
    let stop = crawl.stopper();
    let writes = crawl.writes();
    let crawled = runtime.block_on(async {
        tokio::select! {
            crawled = crawl.run() => Some(crawled),
            () = interrupt(stop) => None,
        }
    });
    // An abandoned crawl's tasks are not waited for, but its writes begun
    // are, the writer draining meanwhile (D49).
    runtime.shutdown_background();
    if crawled.is_none() {
        writes.close();
    }
    // The writer's own error says why a crawl lost it, so it goes first.
    let store = writer.finish().map_err(in_db)?;
    let Some(crawled) = crawled else {
        return Ok((run, None));
    };
    let summary = crawled.map_err(|e| e.to_string())?;
    if !summary.stopped {
        store.finish_run(run).map_err(in_db)?;
    }
    Ok((run, Some(summary)))
}

/// Stops the run at the first Ctrl-C, and returns at the second.
async fn interrupt(stop: Stopper) {
    if tokio::signal::ctrl_c().await.is_err() {
        return std::future::pending().await;
    }
    stop.stop();
    eprintln!(
        "bruce crawl: stopping: finishing requests in flight ({} s at most); Ctrl-C again to \
         abandon them",
        STOP_GRACE.as_secs()
    );
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending().await
    }
}

/// The run's configuration, and the `runs.config_hash` that names it: the
/// `xxh3_64` of the file's text and the URL given, so it changes with any
/// edit to either.
pub fn configure(args: &Args, registry: &Registry) -> Result<(Config, String), String> {
    let (mut config, text) = match &args.config {
        Some(path) => {
            let text =
                std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let config = Config::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            (config, text)
        }
        None => {
            let Some(url) = &args.url else {
                return Err(
                    "give a URL to crawl, or --config naming a file with a [[root]]".into(),
                );
            };
            let config = Config::for_url(url).map_err(|e| format!("{url}: {e}"))?;
            (config, String::new())
        }
    };
    if let (Some(path), Some(url)) = (&args.config, &args.url) {
        if !config.roots.is_empty() {
            return Err(format!(
                "{} names its own roots, so the URL {url} is refused: crawl the file's roots \
                 without a URL, or add a [[root]] for it",
                path.display()
            ));
        }
        config
            .roots
            .push(Root::new(url).map_err(|e| format!("{url}: {e}"))?);
        config
            .check_robots_hosts()
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if config.roots.is_empty() {
        let path = args.config.as_deref().unwrap_or(Path::new(""));
        return Err(format!(
            "{} names no [[root]]: give a URL to crawl",
            path.display()
        ));
    }
    for (i, root) in config.roots.iter().enumerate() {
        if let ProfileChoice::Named(name) = &root.profile
            && registry.get(name).is_none()
        {
            let known: Vec<&str> = registry.iter().map(|p| p.name()).collect();
            return Err(format!(
                "root[{i}].profile: no profile is named `{name}`; the profiles are `auto`, `{}`",
                known.join("`, `")
            ));
        }
    }
    let mut hashed = text.into_bytes();
    hashed.push(0);
    hashed.extend(args.url.as_deref().unwrap_or("").as_bytes());
    Ok((config, format!("{:016x}", xxh3_64(&hashed))))
}

/// The line a run ends with.
fn report(run: RunId, s: &Summary) -> String {
    let mut line = format!("run {}: {} targets, {} pages", run.0, s.targets, s.pages);
    match s.sitemaps {
        0 => line.push_str(", no sitemap"),
        1 => line.push_str(", 1 sitemap file"),
        n => line.push_str(&format!(", {n} sitemap files")),
    }
    if s.dropped > 0 {
        line.push_str(&format!(
            ", {} sitemap entries outside every root dropped",
            s.dropped
        ));
    }
    if s.cut > 0 {
        line.push_str(&format!(", {} cut", s.cut));
    }
    if s.stopped {
        line.push_str("; stopped before the end, so the run is left unfinished");
    }
    line
}

#[cfg(test)]
mod tests;
