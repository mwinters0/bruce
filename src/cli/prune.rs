//! `bruce prune (--keep N | --older-than AGE) [--db f]`: delete old runs from
//! a database (D2), as `store::Rule` and `Store::prune` pick them.
//!
//! One rule says what goes, so naming neither flag or both is a usage error;
//! `--keep` takes at least 1, since deleting every run is deleting the file.
//! The database must exist, and a crawl holding it open makes DuckDB refuse
//! this command's open rather than deleting beneath the crawl (D28).

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Arg, ArgGroup, ArgMatches, Command, value_parser};

use super::db;
use crate::profile::Registry;
use crate::store::Rule;

/// What the command line asked of `prune`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub rule: Rule,
    pub db: Option<PathBuf>,
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        let rule = match (
            m.get_one::<u64>("keep"),
            m.get_one::<Duration>("older-than"),
        ) {
            (Some(&n), _) => Rule::Keep(n as usize),
            (None, Some(&age)) => Rule::OlderThan(age),
            (None, None) => unreachable!("clap requires one"),
        };
        Args {
            rule,
            db: m.get_one::<String>("db").map(PathBuf::from),
        }
    }
}

pub fn command() -> Command {
    Command::new("prune")
        .about("Delete old runs from a database")
        .arg(
            Arg::new("keep")
                .long("keep")
                .value_name("N")
                .value_parser(value_parser!(u64).range(1..))
                .help("Keep the N latest finished runs and delete every older run"),
        )
        .arg(
            Arg::new("older-than")
                .long("older-than")
                .value_name("AGE")
                .value_parser(age)
                .help("Delete the runs started longer ago than this, such as 90d or 12h"),
        )
        .group(
            ArgGroup::new("rule")
                .args(["keep", "older-than"])
                .required(true),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .value_name("FILE")
                .help("The database to prune [default: bruce.duckdb]"),
        )
}

/// `--older-than`'s value: a whole number and a unit, `s`, `m`, `h`, `d` or
/// `w`.
fn age(s: &str) -> Result<Duration, String> {
    let wrong = || format!("expected a number and a unit (s, m, h, d or w), as in 90d: {s:?}");
    let (number, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).ok_or_else(wrong)?);
    let seconds: u64 = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        "w" => 604_800,
        _ => return Err(wrong()),
    };
    let n: u64 = number.parse().map_err(|_| wrong())?;
    n.checked_mul(seconds)
        .map(Duration::from_secs)
        .ok_or_else(|| format!("too long an age: {s:?}"))
}

/// Runs `args` with the profiles `registry` holds, printing what it deleted
/// to stdout.
pub fn run(args: &Args, registry: Registry) -> ExitCode {
    match prune(args, &registry) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bruce prune: {e}");
            ExitCode::FAILURE
        }
    }
}

fn prune(args: &Args, registry: &Registry) -> Result<(), String> {
    let db = db::path(args.db.as_deref());
    if !db.exists() {
        return Err(format!("{}: no such database", db.display()));
    }
    let in_db = |e: &dyn std::fmt::Display| format!("{}: {e}", db.display());
    let store = db::open(db, registry).map_err(|e| in_db(&e))?;
    let runs = store.prune(args.rule).map_err(|e| in_db(&e))?;
    if runs.is_empty() {
        println!("bruce prune: no run to delete");
    } else {
        let ids: Vec<String> = runs.iter().map(|r| r.0.to_string()).collect();
        let s = if runs.len() == 1 { "" } else { "s" };
        println!(
            "bruce prune: deleted {} run{s}: {}",
            runs.len(),
            ids.join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;
