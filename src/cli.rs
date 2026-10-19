//! Subcommands (D27), one child module each: `crawl` and `report`. The only module that names a
//! concrete profile, to build the registry (D39).

/// Public so `examples/sim_config.rs` loads a config exactly as `crawl` does.
pub mod crawl;
mod db;
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
        Err(e) => {
            // `--help` and `--version` are errors to clap that print to stdout.
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::from(USAGE)
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match matches.subcommand() {
        Some(("crawl", m)) => crawl::run(&crawl::Args::from(m), registry()),
        Some(("report", m)) => report::run(&report::Args::from(m), registry()),
        _ => unreachable!("clap requires a subcommand"),
    }
}

/// The command line `run_from` parses.
fn command() -> Command {
    Command::new("bruce")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(crawl::command())
        .subcommand(report::command())
}

#[cfg(test)]
mod tests;
