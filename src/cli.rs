//! Subcommands (D27). The only module that names a concrete profile, to build
//! the registry (D39).

use std::process::ExitCode;

/// Runs the command line. No subcommand exists yet, so it says so and fails.
pub fn run() -> ExitCode {
    eprintln!(
        "bruce {}: no subcommand is implemented yet",
        env!("CARGO_PKG_VERSION")
    );
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    #[test]
    fn a_run_with_no_subcommand_fails() {
        assert_eq!(super::run(), ExitCode::FAILURE);
    }
}
