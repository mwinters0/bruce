# The concept map

Each concept the code holds, the crates and modules holding it, and why each
crate holds its part. **Read it before deciding where new code goes.** The rule
it serves is [`../process.md`](../process.md), "Seams". `scripts/concepts.py`
holds every production file in the workspace to exactly one concept here.

A line `- **<crate>** — …` claims the backticked paths in it, relative to that
crate's `src/`. A path is a file, or a directory written with a trailing `/`,
which claims everything under it and the `<dir>.rs` beside it. A crate root
(`lib.rs`, `main.rs`) holding only its crate's doc, `mod` lines and re-exports
is claimed by none. `bruce` is one crate (D31), so a concept lives in one top-level module and
spans others only along the layering [`decisions.md`](decisions.md), "D39"
records, each part saying why its module holds it. **Not in one place** names what today's tree scatters and the layering
does not explain.

Every concept below is, so far, one top-level module, placed by P1's spec (`../wip/roadmap-P1-scaffold.md`, "Where the concepts
live"). `cli`, `config`, `store` and `profile` hold code; the rest hold only their doc. The
decisions each carries are cited in its module's doc.

## Subcommands

- **bruce** — `cli.rs` and the binary's `main.rs`, which only calls it: the
  top of the layering, and the one module naming a concrete profile (D39).

## The crawl

- **bruce** — `crawl.rs`.

## CI mode

- **bruce** — `ci.rs`.

## Reports and pruning

- **bruce** — `report.rs`.

## Fetchers

- **bruce** — `fetch.rs`.

## The extraction pass and its core heuristics

- **bruce** — `extract.rs`.

## Storage

- **bruce** — `store/`: the connection, the schema's version check and a
  profile's views in the parent file, the tables in the `schema` child and
  the single writer thread in the `writer` child. It sits above `profile`
  because it asks each profile for its views and answers its lookups (D39).

## Profiles

- **bruce** — `profile/`: the trait, the registry and the fact and lookup
  types in the parent file, and a child module for each profile compiled in
  (D33), `generic` and `antora`. `cli` alone builds the registry (D39).

## Forge edit-URL shapes

- **bruce** — `forge.rs`.

## Canonical URLs and IDs

- **bruce** — `url.rs`.

## Configuration

- **bruce** — `config.rs`: the loader, its defaults and its validation, at
  the bottom of the layering so every module may read a setting (D39).
