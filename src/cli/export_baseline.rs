//! `bruce export-baseline --out f [--run n] [--db f] [--force]`: one finished
//! run as a database file of its own, for CI to check a build against (D21,
//! D27; `roadmap-P5-ci.md`, "`export-baseline` and `--baseline`").
//!
//! The database must exist. `--out` must not, unless `--force` replaces it;
//! it is never the database itself. The line printed gives the file's size
//! and its gzipped size, which a deployer weighs before publishing it
//! compressed.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use flate2::Compression;
use flate2::write::GzEncoder;

use super::db;
use crate::profile::Registry;
use crate::store::{Exported, RunId};

/// What the command line asked of `export-baseline`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub out: PathBuf,
    pub run: Option<RunId>,
    pub db: Option<PathBuf>,
    pub force: bool,
}

impl From<&ArgMatches> for Args {
    fn from(m: &ArgMatches) -> Args {
        Args {
            out: m
                .get_one::<String>("out")
                .map(PathBuf::from)
                .expect("clap requires it"),
            run: m.get_one::<u64>("run").map(|&n| RunId(n)),
            db: m.get_one::<String>("db").map(PathBuf::from),
            force: m.get_flag("force"),
        }
    }
}

pub fn command() -> Command {
    Command::new("export-baseline")
        .about("Write one finished run as a baseline file for CI")
        .arg(
            Arg::new("out")
                .long("out")
                .value_name("FILE")
                .required(true)
                .help("The baseline file to write; it must not exist unless --force"),
        )
        .arg(
            Arg::new("run")
                .long("run")
                .value_name("N")
                .value_parser(value_parser!(u64))
                .help("The run to export, which must have finished [default: the latest finished]"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .value_name("FILE")
                .help("The database to read [default: bruce.duckdb]"),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Replace --out if it exists"),
        )
}

/// Runs `args` with the profiles `registry` holds, printing what it wrote to
/// stdout.
pub fn run(args: &Args, registry: Registry) -> ExitCode {
    match export(args, &registry) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("bruce export-baseline: {e}");
            ExitCode::FAILURE
        }
    }
}

fn export(args: &Args, registry: &Registry) -> Result<String, String> {
    let db = db::path(args.db.as_deref());
    if !db.exists() {
        return Err(format!("{}: no such database", db.display()));
    }
    let out = args.out.as_path();
    if out.exists() {
        if same_file(db, out) {
            return Err(format!("{}: --out is the database itself", out.display()));
        }
        if !args.force {
            return Err(format!("{}: exists; --force replaces it", out.display()));
        }
    }
    let store = db::open(db, registry).map_err(|e| format!("{}: {e}", db.display()))?;
    let in_out = |e: &dyn std::fmt::Display| format!("{}: {e}", out.display());
    // Written beside `--out` and renamed onto it, so a refused run leaves the
    // file `--force` would have replaced.
    let partial = partial(out);
    remove(&partial).map_err(|e| in_out(&e))?;
    let exported = store
        .export_baseline(args.run, &partial, registry)
        .map_err(|e| format!("{}: {e}", db.display()))?;
    drop(store);
    fs::rename(&partial, out).map_err(|e| in_out(&e))?;
    let size = fs::metadata(out).map_err(|e| in_out(&e))?.len();
    let gzipped = gzipped_size(out).map_err(|e| in_out(&e))?;
    Ok(summary(&exported, out, size, gzipped))
}

/// Where `out` is written before it replaces anything.
fn partial(out: &Path) -> PathBuf {
    let mut name = out.as_os_str().to_owned();
    name.push(".partial");
    PathBuf::from(name)
}

/// Whether both paths name one existing file.
fn same_file(a: &Path, b: &Path) -> bool {
    matches!((a.canonicalize(), b.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

/// Deletes a file and its write-ahead log, if there.
fn remove(file: &Path) -> io::Result<()> {
    let mut wal = file.as_os_str().to_owned();
    wal.push(".wal");
    for f in [file.to_path_buf(), PathBuf::from(wal)] {
        match fs::remove_file(f) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

/// The bytes `path` takes gzipped at `gzip`'s default level.
fn gzipped_size(path: &Path) -> io::Result<u64> {
    struct Count(u64);
    impl Write for Count {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0 += buf.len() as u64;
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut gz = GzEncoder::new(Count(0), Compression::default());
    io::copy(&mut File::open(path)?, &mut gz)?;
    Ok(gz.finish()?.0)
}

fn summary(e: &Exported, out: &Path, size: u64, gzipped: u64) -> String {
    let mib = |bytes: u64| format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0));
    format!(
        "bruce export-baseline: run {} to {}: {} targets, {} pages, {} anchors, {} links; \
         {} bytes ({}), {} bytes ({}) gzipped",
        e.run.0,
        out.display(),
        e.targets,
        e.pages,
        e.anchors,
        e.links,
        size,
        mib(size),
        gzipped,
        mib(gzipped),
    )
}

#[cfg(test)]
mod tests;
