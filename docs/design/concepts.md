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

Every concept below is, so far, one top-level module, laid out by the layering D39 records. `cli`, `config`, `store`, `profile`, `extract`, `forge`, `url`, `fetch`, `crawl`, `ci` and `report` hold code. The
decisions each carries are cited in its module's doc.

## Subcommands

- **bruce** — `cli/` and the binary's `main.rs`, which only calls it: the
  command line and the registry in the parent file, a child module for
  each subcommand, `check_dir` (its build, the run over it and the line it prints, and the gate's exit status), `crawl` (its roots from the file or the URL and the run), `export_baseline` (its files and the line it prints), `file` (the baseline file a path or URL names, downloaded and decompressed, which `check_dir` and `report digest --since` both read), `interrupt` (what SIGINT and SIGTERM do to the run `crawl` and `check_dir` make, D58), `prune` (its rule and what it prints) and `report` (its options and
  printing the output, or writing the digest's file), and the `db`
  child, which all five open the database through, installing the reports' views
  that only this layer can hand `store` (D6, D39). It is the top of the
  layering, and the one module naming a concrete profile (D39).

## The crawl

- **bruce** — `crawl/`: the run, its tasks, the rows a target and its page
  write, and the landing of a redirect in the parent file; the frontier (a
  URL's class, its canonical form, the claim that dedups, and the other
  locales the pages' profiles found with the URLs waiting on them, D67) in the `frontier`
  child; a target's redirect chain, its limit and where a hop leads in the
  `hops` child; D16's classification in the `outcome` child; which target starts next, a build's reads in a budget of their own and each live host holding its share of
  the run's other slots, in the `schedule` child (D61); reading one
  sitemap file, gzip and index included, in the `sitemap` child, discovery
  staying in the parent, and asking the URL a 404's final slash toggled to, in the `slash` child (D72). A run over a local build begins from its seeds in the
  parent, which holds the build's site root for `extract` to resolve an href beginning `/` against (D53).
  What a run over a build leaves to the baseline, the component-versions its
  pages hold and the copies the parent sends last is in the `baseline` child (D54); the
  xrefs of a built page, resolved through the baseline's pages once the held
  set is final, in the `xref` child (D55). It sits above
  `fetch`, `extract` and `store`, driving each (D39).

## CI mode

- **bruce** — `ci.rs`: a build directory read as the URLs it will have in
  production, the prefix mapping, which files are seeds and the refusal of a
  prefix outside every root's scope, handing `fetch/local` a finished map. It
  sits beside `crawl` and `report`, none calling another, and above `config`
  (D39).

## Reports and pruning

- **bruce** — `report/`: running one report in the parent file, the request
  and the output in the `request` child, choosing the run in `run`, the
  statement and the filters every report shares in `sql`, `broken`'s and
  `errors`' query in `links`, `stats`' and the choice among all the reports in
  `query`, how a page is reached (`inbound`, `orphans`, `sitemap`,
  `unlisted`) in `reach`, `redirects`, `diff`, `changed` and `anchors_removed` (two runs each), the gate
  (`introduced`: a CI run against its baseline, D9) and the `inverse` check
  (what the change removed that pages outside the build link to, D56) each in their own child,
  the crawl run's digest (D64) in `digest`, its document in the `document` child, its sections' queries in `sections`,
  the run it is compared with, and the anchors removed since (in the JSON alone), in `compare` (D73), the breakage new and fixed since, each new link with its cause, in `cause` (D74), what is new since in each section in `novelty` (D75), its JSON in `json` and its HTML, with the stylesheet it holds, the severity dots in its `severity` child, the markup helpers they and the document share in `markup` and the comparison's sub-title, figure changes, the legend's line for what is new and sections (the breakage new, the breakage fixed) in its `compared` child, in `html`,
  the views they read in `views`, the table's
  text and JSON in `table` and the failures in `error`.
  It sits above `store`, whose `link_verdicts` its views join, and below
  `cli`, which installs its views at every open (D39). Pruning (D2) is not
  a report's: its deletes are `store`'s, in the `prune` child below, and its
  command line `cli`'s.

## Fetchers

- **bruce** — `fetch/`: the trait (a hop, and whether robots.txt lets it
  be asked), one hop's response, the failure causes and the stop a crawl
  hands its fetcher in the parent file, the reqwest fetcher in the `http` child, each
  host's pace, in-flight cap and robots.txt in `http`'s `host` child, and
  the body-to-text decoding in the `charset` child, and the build-directory
  fetcher in the `local` child, which holds a map from URL to file and no
  directory logic, that being `ci`'s, and reuses `charset`; the `routed` child
  sends a URL the build holds to it and any other to the live fetcher. It sits above `config` (it reads the rate, the hosts, the roots and
  the robots token; `cli` passes the user agent in) and `url`; the crawler above it follows the hops (D39).

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
  row types in the `rows` child, the views over them (`link_verdicts` and the `doubled_slash` targets it joins) in the
  `views` child, deleting old runs (D2) in the `prune` child, writing one run as a baseline file
  (D21), importing one as a run, listing the targets a run holds with its pages' facts, which a profile resolves through (D55), and copying a target's rows between runs (D52) in the `baseline` child and the single
  writer thread, which also runs a requested copy, in the `writer` child. It sits above `profile`
  because it asks each profile for its views and answers its lookups (D39).

## Profiles

- **bruce** — `profile/`: the trait, the registry and the fact and lookup
  types in the parent file, and a child module for each profile compiled in
  (D33), `generic` and `antora`, whose Resolve is its `resolve` child (D55) and whose hint for an unresolved xref is its `hints` child (D65). `cli` alone builds the registry (D39).

**Not in one place.** I16's URL layout, a component-version's prefix in a
page's URL, is read in `crawl/baseline.rs` (`Held::record`, `Held::holds`), `report/inverse.rs`
(`prefix`) and `profile/antora/resolve.rs` (`prefix`); a shared reader would sit
below `profile` (D39).

## Forge edit-URL shapes

- **bruce** — `forge.rs`: the shape parse and the edit link it parses, below
  `profile` so the finish step can read it (D39).

## Canonical URLs and IDs

- **bruce** — `url.rs`: the canonical form and `url_id`, below `config`, which reads it to match a root's scope on the form (D39, D44).

## Configuration

- **bruce** — `config.rs`: the loader, its defaults and its validation, and
  the allowlist's matching of a host (D68), at
  the bottom of the layering but for `url`, so every module may read a setting (D39).
