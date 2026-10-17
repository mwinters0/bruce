# P1.3 Scaffold: notes

What landed, and what the next slices inherit. The row is `P1.3` in
[`../status/STATUS.md`](../status/STATUS.md), its intent in
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md).

## What landed

- **`bruce` is the root package**, a library with a binary beside it:
  `src/main.rs` only calls `cli::run`, so the benches and integration tests
  D24 asks for can reach the modules. The manifest has no `[workspace]`
  table, so `spikes/tokenizer/` stays its own project, as P1.2 left it.
- **Rust 1.99.1** is pinned in `mise.toml` with clippy, rustfmt and
  rust-analyzer as components; `rustup which rust-analyzer` resolves through
  mise's `RUSTUP_TOOLCHAIN`, so `module_graph.py` needs no
  `rust-toolchain.toml`. `mise ls-remote rust` lists 1.99.1 now, so the
  spec's fallback to 1.99.0 was not needed.
- **The mise tasks** `build`, `test`, `clippy`, `fmt` and `bench`, beside
  `check`, which already runs fmt, clippy and the suite with its own flags.
- **The eleven top-level modules of D39**, each holding only its doc, with
  `profile::generic` and `profile::antora` as children of `profile`, and the
  concept map's first entries. `cli::run` fails with "no subcommand is
  implemented yet"; its one test keeps nextest from failing on an empty
  suite (`error: no tests to run`).
- **The concrete-profile check** is `module_graph.py`'s `CONFINED` table:
  an edge into `bruce::profile::generic` or `bruce::profile::antora` from
  any module but `bruce::cli` (or the profile's own descendants) fails the
  graph. Proven on the tree by a throwaway `crawl` → `generic` reference,
  which reported `!! confined: bruce::crawl -> bruce::profile::generic`.
- **The checks skip the spike.** `file_limits.py` (and so `concepts.py` and
  `citations.py`) and `repoint.py` skip a package nested below a member that
  is not one; the spike's provenance comments had turned `repoint.py` red.

## What the next slices inherit

- **P1.4** fills `config.rs`; **P1.5** fills `store.rs` and `profile.rs`.
  A profile's types reach the rest of the crate only through the trait, or
  the graph fails (D39).
- **No bench exists**, so `mise run bench` runs nothing and `mise run check`
  runs no bench; the slice that moves the parser bench in (P2) joins it to
  the check in test mode (`../design/roadmap-P2-extractor-inbox.md`).
- **`check.py --affected` treats every `src/` change as outside a crate**:
  a root package's directory is `.`, which no path starts with, so any Rust
  change runs every check. For a one-crate repo that is the same set.
- **The spike now builds under 1.99.1** too (mise applies below the root);
  its numbers were taken under 1.98.0.

## Negative results

- **A workspace table with `exclude = ["spikes"]`** was not needed: cargo
  finds no workspace above the spike while the root manifest has none.
