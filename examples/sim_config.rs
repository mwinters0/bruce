//! `cargo run --example sim_config -- FILE`: loads FILE as `bruce crawl
//! --config FILE` would, by the same function, and prints what it holds.
//!
//! `mise run sim` runs it on the deployment guide's config (D42): the check
//! that the guide's example is a config this build accepts, with no request
//! sent. Exit status 1 and the message on stderr when it is refused.

use std::path::PathBuf;
use std::process::ExitCode;

use bruce::cli::crawl::{Args, configure};

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: sim_config FILE");
        return ExitCode::from(2);
    };
    let args = Args {
        config: Some(PathBuf::from(path)),
        ..Args::default()
    };
    match configure(&args, &bruce::cli::registry()) {
        Ok((config, _)) => {
            println!(
                "{} root(s), {} allowlisted host(s) or domain(s), {} skip glob(s)",
                config.roots.len(),
                config.allowlist.entries().len(),
                config.skip.globs().len()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("sim_config: {e}");
            ExitCode::FAILURE
        }
    }
}
