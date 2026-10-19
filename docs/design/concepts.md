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
live"). `cli`, `config`, `store`, `profile`, `extract`, `forge`, `url`, `fetch`, `crawl` and `report` hold code; the rest hold only their doc. The
decisions each carries are cited in its module's doc.

## Subcommands

- **bruce** — `cli/` and the binary's `main.rs`, which only calls it: the
  command line and the registry in the parent file, a child module for
  each subcommand, `crawl` (its roots from the file or the URL, the run and
  Ctrl-C) and `report` (its options and printing the output), and the `db`
  child, which both open the database through, installing the reports' views
  that only this layer can hand `store` (D6, D39). It is the top of the
  layering, and the one module naming a concrete profile (D39).

## The crawl

- **bruce** — `crawl/`: the run, its tasks, the rows a target and its page
  write, and the landing of a redirect in the parent file; the frontier (a
  URL's class, its canonical form and the claim that dedups) in the `frontier`
  child; a target's redirect chain, its limit and where a hop leads in the
  `hops` child; D16's classification in the `outcome` child; and reading one
  sitemap file, gzip and index included, in the `sitemap` child, discovery
  staying in the parent. It sits above
  `fetch`, `extract` and `store`, driving each (D39).

## CI mode

- **bruce** — `ci.rs`.

## Reports and pruning

- **bruce** — `report/`: running one report in the parent file, the request
  and the output in the `request` child, choosing the run in `run`, the
  statement and the filters every report shares in `sql`, the link reports'
  and `stats`' queries and the choice among all of them in `query`, how a
  page is reached (`inbound`, `orphans`, `sitemap`, `unlisted`) in `reach`,
  `redirects` in its own child, the views they read in `views`, the table's
  text and JSON in `table` and the failures in `error`.
  It sits above `store`, whose `link_verdicts` its views join, and below
  `cli`, which installs its views at every open (D39). Pruning (D2) is not
  built.

## Fetchers

- **bruce** — `fetch/`: the trait (a hop, and whether robots.txt lets it
  be asked), one hop's response, the failure causes and the stop a crawl
  hands its fetcher in the parent file, the reqwest fetcher in the `http` child, each
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
  row types in the `rows` child, the views over them (`link_verdicts`) in the
  `views` child and the single writer thread in the `writer`
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
