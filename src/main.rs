//! The `bruce` binary: everything it does is `bruce::cli`'s. The crate is a
//! library too, so benches and integration tests reach its modules.

fn main() -> std::process::ExitCode {
    bruce::cli::run()
}
