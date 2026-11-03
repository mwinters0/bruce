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
  command line in the parent file, the registry of the compiled profiles and a config's in its `registry` child, a child module for
  each subcommand, `check_dir` (its build, the run over it and the line it prints, and the gate's exit status), `crawl` (its roots from the file or the URL and the run), `export_baseline` (its files and the line it prints), `file` (the baseline file a path or URL names, downloaded and decompressed, which `check_dir` and `report digest --since` both read), `interrupt` (what SIGINT and SIGTERM do to the run `crawl` and `check_dir` make, D58), `prune` (its rule and what it prints) and `report` (its options and
  printing the output, or writing the digest's file), and the `db`
  child, which all five open the database through, installing the reports' views
  that only this layer can hand `store` (D6, D39). It is the top of the
  layering, and the one module naming a concrete profile (D39).

## The crawl

- **bruce** — `crawl/`: the run, its tasks, the rows a target and its page
  write, and the landing of a redirect in the parent file; the frontier (a
  URL's class, its canonical form, the claim that dedups, and the other
  locales the pages' profiles name, with the URLs waiting on them, D67) in the `frontier`
  child; a target's redirect chain, its limit and where a hop leads in the
  `hops` child; D16's classification in the `outcome` child; which target starts next, a build's reads in a budget of their own and each live host holding its share of
  the run's other slots, in the `schedule` child (D61); reading one
  sitemap file, gzip and index included, in the `sitemap` child, discovery
  staying in the parent, and asking the URL a broken 4xx's final slash toggled to, in the `slash` child (D72). A run over a local build begins from its seeds in the
  parent, which holds the build's site root for `extract` to resolve an href beginning `/` against (D53).
  What a run over a build leaves to the baseline, the component-versions its
  pages hold and the copies the parent sends last is in the `baseline` child (D54); the
  xrefs of a built page, resolved through the baseline's pages once the held
  set is final, in the `xref` child (D55). It sits above
  `fetch`, `extract` and `store`, driving each (D39).

## CI mode

- **bruce** — `ci.rs`: a build directory read as the URLs it will have in
  production, the prefix mapping, which files are seeds, a top `index.html`
  that is a meta refresh served at its own path and no prefix (D101), and the refusal of a
  prefix outside every root's scope, handing `fetch/local` a finished map. It
  sits beside `crawl` and `report`, none calling another, and above `config`
  (D39).

## Reports and pruning

- **bruce** — `report/`: running one report in the parent file, the request
  and the output in the `request` child, choosing the run in `run`, the
  statement and the filters every report shares in `sql`, `broken`'s and
  `errors`' query in `links`, `stats`' and the choice among all the reports in
  `query`, how a page is reached (`inbound`, `orphans`, `sitemap`,
  `unlisted`) in `reach`, `redirects`, and `diff`, `changed` and `anchors_removed` (two runs each), the gate
  (`introduced`: a CI run against its baseline, D9) and the `inverse` check
  (what the change removed that pages outside the build link to, D56) each in their own child,
  the crawl run's digest (D64) in `digest`, its document in the `document` child, its sections' queries in `sections`,
  the run it is compared with, and the anchors removed since (in the JSON alone), in `compare` (D73), the breakage new and fixed since, each new link with its cause, in `cause` (D74), what is new since in each section that marks it in `novelty` (D75), its JSON in `json`, its HTML as the built-in view and any a deployment writes (D95, D96) in `view`: the template engine, the views named on the command line and their rendering over one model in the parent
  file, its formatter, the escaping it writes and its filters in `filters`, where a view's templates come from (the built-in files under `bruce:`, a view's directory, nothing outside it) in `loader`, the model version a view declares in `declared`, the built-in view's template, stylesheet and palette, embedded from the directory it holds; the model a rendering reads, holding no words but a sitemap note's, in `model`:
  the findings its counts, lists and ranking keep under the severity map, a kind at none being none (D92), in its `standing` child, the broken targets a section lists in `targets`, the settings off their defaults and the changes that judge in `policy` (D91), the rows of a table found by column name in `row`, and the whole of it as the one tree of values a template reads in `context`; the policy rows that differ between the two runs compared in the `changes` child (D91),
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
  sends a URL the build holds to it and any other to the live fetcher. It sits above `config` (it reads the rate, the hosts, the roots, the
  robots token, the user agent and how a run asks) and `url`; the crawler above it follows the hops (D39).

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
  row types in the `rows` child, the views over them (`link_verdicts`, the table macro it calls, the `policy_in_force` rows and `doubled_slash` targets it joins, the guard naming the run a comparison judges by and `link_hints`, the profiles' hints it offers the reports, D87) in the
  `views` child, a run's recorded policy among the tables (D78) and a trial's judging rows in its place for a connection in the `trial` child (D84), deleting old runs (D2) in the `prune` child, writing one run as a baseline file
  (D21), importing one as a run, listing the targets a run holds with its pages' facts, which a profile resolves through (D55), and copying a target's rows between runs (D52) in the `baseline` child and the single
  writer thread, which also runs a requested copy, in the `writer` child. It sits above `profile`
  because it asks each profile for its views and answers its lookups (D39).

## Profiles

- **bruce** — `profile/`: the trait, the registry, the fact and lookup
  types and the prefix a language-menu link makes of a locale (D67) in the parent file, and a child module for each profile compiled in
  (D33), `generic` and `antora`, whose Resolve is its `resolve` child (D55) and whose hint for an unresolved xref is its `hints` child (D65), and `declarative`, a profile built from a config's `[[profile]]` (D33), wrapping the compiled profile it extends where it names one (D94), and the `selectors` child, which parses a selector's text once per run for every handler, the core's and the profiles' (D11), and the `interned` child, which hands a declared profile's name and region out as the same `'static` text for every registry built from one definition. `cli` alone builds the registry, the file's profiles ahead of the compiled ones (D39, D35).

**Not in one place.** I16's URL layout, a component-version's prefix in a
page's URL, is read in `crawl/baseline.rs` (`Held::record`, `Held::holds`), `report/inverse.rs`
(`prefix`) and `profile/antora/resolve.rs` (`prefix`); a shared reader would sit
below `profile` (D39).

## Forge edit-URL shapes

- **bruce** — `forge.rs`: the shape parse and the source it gives, and `forge/rule.rs`, the source
  rules a config states, tried before it (D93), below `config`, which builds one to check it, and
  `profile`, so the finish step can read the source (D39).

## Canonical URLs and IDs

- **bruce** — `url.rs`: the canonical form, `url_id` and a final `/` toggled (D72), below `config`, which reads it to match a root's scope on the form (D39, D44).

## Configuration

- **bruce** — `config.rs`: the loader, its defaults and its validation, and
  the allowlist's matching of a host (D68), `config/profile.rs`, a declarative profile as the file defines it and the rows a run records it as (D33, D78), `config/selector.rs`, the selectors a file writes, split and checked against what lol_html streams, and `config/policy.rs`, the
  judging settings a run is judged by and records, with their defaults (D78), a row of the record and how a file writes it in `config/policy/entry.rs`, the severity map in `config/policy/severity.rs` (D82), the outcome and the reason a link carries, the enums every writer and reader names and the map validates against (`store` re-exporting `Outcome`), in `config/policy/verdict.rs`, and the outcomes that fail the gate in `config/policy/gate.rs` (D83), the fact the digest groups by in `config/policy/digest.rs` (D86) and how a crawl asks, recorded with them and judged by nothing, in `config/policy/asking.rs` (D85), at
  the bottom of the layering but for `url`, so every module may read a setting (D39).
