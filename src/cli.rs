//! Subcommands (D27), one child module each: `check_dir`, `crawl`, `export_baseline`, `prune` and `report`. The only module that names a
//! concrete profile, to build the registry (D39).

mod check_dir;
/// Public so `examples/sim_config.rs` loads a config exactly as `crawl` does.
pub mod crawl;
mod db;
mod export_baseline;
mod interrupt;
mod prune;
mod report;

use std::ffi::OsString;
use std::process::ExitCode;

use clap::Command;

use crate::profile::Registry;
use crate::profile::antora::Antora;
use crate::profile::generic::Generic;

/// The exit status of a command line `clap` refuses, as its own `exit` uses.
const USAGE: u8 = 2;

/// The profiles compiled in, `generic` the fallback (D35).
pub fn registry() -> Registry {
    Registry::new(Box::new(Generic)).with(Box::new(Antora))
}

/// Runs the process's command line.
pub fn run() -> ExitCode {
    run_from(std::env::args_os())
}

/// Runs `args`, the program's name first.
pub fn run_from<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = match command().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(e) => return refuse(&e),
    };
    match matches.subcommand() {
        Some(("check-dir", m)) => check_dir::run(&check_dir::Args::from(m), registry()),
        Some(("crawl", m)) => crawl::run(&crawl::Args::from(m), registry()),
        Some(("export-baseline", m)) => {
            export_baseline::run(&export_baseline::Args::from(m), registry())
        }
        Some(("prune", m)) => prune::run(&prune::Args::from(m), registry()),
        Some(("report", m)) => match report::check(m) {
            Ok(()) => report::run(&report::Job::from(m), registry()),
            Err(e) => refuse(&e),
        },
        _ => unreachable!("clap requires a subcommand"),
    }
}

/// Prints a command line `clap` refuses and gives its exit status.
fn refuse(e: &clap::Error) -> ExitCode {
    // `--help` and `--version` are errors to clap that print to stdout.
    let _ = e.print();
    if e.use_stderr() {
        ExitCode::from(USAGE)
    } else {
        ExitCode::SUCCESS
    }
}

/// The command line `run_from` parses.
fn command() -> Command {
    Command::new("bruce")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(check_dir::command())
        .subcommand(crawl::command())
        .subcommand(export_baseline::command())
        .subcommand(prune::command())
        .subcommand(report::command())
}

#[cfg(test)]
mod tests;
