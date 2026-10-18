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

Every concept below is, so far, one top-level module, placed by P1's spec (`roadmap-P1-scaffold.md`, "Where the concepts
live"). `cli`, `config`, `store`, `profile`, `extract`, `forge`, `url` and `fetch` hold code; the rest hold only their doc. The
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

- **bruce** — `fetch/`: the trait, one hop's response and the failure
  causes in the parent file, the reqwest fetcher in the `http` child, each
  host's pace, in-flight cap and robots.txt in `http`'s `host` child, and
  the body-to-text decoding in the `charset` child, which P5's local fetcher
  reuses. It sits above `config` (it reads the rate and `user_agent`) and
  `url`; the crawler above it follows the hops (D39).

## The extraction pass and its core heuristics

- **bruce** — `extract/`: the pass, the page's result, the generator pre-pass
  and the title in the parent file, anchors (validity and placement) in the
  `anchors` child, links (kinds, `srcset`, `<base>`, refresh) in the `links`
  child, the edit-link ranking in the `edit_link` child, and the scoped
  content hash in the `hash` child. It sits above `profile` because it drives the pass the
  profile's Extract hook adds to, and reads no concrete profile (D39).

## Storage

- **bruce** — `store/`: the connection, the schema's version check and a
  profile's views in the parent file, the tables in the `schema` child, the
  row types in the `rows` child and the single writer thread in the `writer`
  child. It sits above `profile`
  because it asks each profile for its views and answers its lookups (D39).

## Profiles

- **bruce** — `profile/`: the trait, the registry and the fact and lookup
  types in the parent file, and a child module for each profile compiled in
  (D33), `generic` and `antora`. `cli` alone builds the registry (D39).

## Forge edit-URL shapes

- **bruce** — `forge.rs`: the shape parse and the edit link it parses, below
  `profile` so the finish step can read it (D39).

## Canonical URLs and IDs

- **bruce** — `url.rs`: the canonical form and `url_id`, below `config`, which reads it to match a root's scope on the form (D39, D44).

## Configuration

- **bruce** — `config.rs`: the loader, its defaults and its validation, at
  the bottom of the layering but for `url`, so every module may read a setting (D39).
