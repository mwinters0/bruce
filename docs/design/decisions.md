# Decisions

The decisions the code cannot explain: shapes kept for a deliverable not yet built, defaults chosen
over an alternative, and obvious changes argued and refused. Nothing here says how the code works
(the named module does). External facts are cited by `I<n>` from `site-invariants.md`. Cite an
entry as `docs/design/decisions.md`, "D12"; the rest of the rules, the line cap included, are
`docs/process.md`, "The decision register".

The modules are P1's scaffold, and `config`, `store` and `profile` are built. D1–D30 were settled in the bootstrap grilling (2026-10-17, frozen as
`historical/initial.md`), D31 the same day, D32 onward in P1's grilling, and its **Code** is the module that will hold it once built. D25 and D29
are spent: D25 was superseded by D30, and D29 became `roadmap.md`'s phase index.

<!-- decision-watermark: D44 -->

## Scope and modes

### D1 Two fetchers behind one trait: live HTTP crawl first, local build directory second
Parsing and the DB writer are shared. The local-dir mode exists for CI verification of a docs change
before it deploys. A full site build takes hours, so nobody runs one locally. Rejected: local-only,
which cannot see the deployed site or external hosts. Reopens: a published full-site build artifact.

### D3 Allowlisted external hosts are checked one hop deep and never crawled
An allowlisted target gets a full GET, so its fragment can be checked. A link to any other host is
stored as `unchecked` with no request, keeping the link graph whole. Rejected: crawling allowlisted
hosts, which would make them roots by another name (D5 holds roots).

### D10 Default scope: en-US, every fedora version; locales are a root that is off by default
Frozen f26–f43 pages are cheap and current pages link into them, so their anchors must be known.
Translated heading ids can break fragments, but the translation team owns that (I9). Rejected:
latest-only, which leaves inbound links to frozen versions unverifiable.

### D13 Every reference kind is extracted, each target fetched once per run
`<a>`, `<area>`, `<img>`/`srcset`, `<link>` (canonical included), `<script>`, `<iframe>`, media
(`src` on `<video>`, `<audio>`, `<source>`, `<track>`; `poster`) and the `url=` of a meta refresh,
each link carrying its `kind`. Assets get a HEAD request with GET fallback, and reports default to
`kind = anchor`. `mailto:`, `tel:`, `javascript:` and `data:` are stored as `unchecked`. A `<link>`
whose `rel` is only `preconnect` or `dns-prefetch` names an origin and is skipped. The first
`<base href>` resolves every href after it. Rejected: `<a href>` only, which misses broken images
and canonicals at near-zero cost; ignoring `<base>`, which resolves a whole site wrongly and silently.

### D14 Discovery is sitemap and link-following both, and each page records which
Each root's sitemap is discovered (D40) or set (docs.fp.o's must be, I1). Recording both
sources yields orphans (in the sitemap, with no inbound link) and unlisted pages (reachable, absent
from the sitemap). Rejected: either source alone, which loses one of those reports.

## Profiles

### D32 Two extension layers: generator profiles in code, the deployment in config
The core knows the web. A profile knows a site generator (`generic`, `antora`). A deployment
(docs.fp.o) is config only, owned outside this repo (D41). Forge edit-URL shapes are core, as every
generator links to a forge. Rejected: a `fedora` profile above `antora`, empty on day one.
Reopens: a deployment quirk that config and profile settings cannot express.

### D33 Profiles are compiled in; declarative per-root rules come later, as their own phase
A profile is a module behind a trait, so a new generator is a PR. Facts a profile records are
key/value strings, so a TOML rule (selector + attribute, region selector, URL regex) can produce
the same facts later without a schema change. Resolution logic stays code. Rejected now: WASM or
out-of-process plugins (an ABI, sandbox and versioning burden for no second user).

### D34 An anchor's placement is `content`, `chrome` or `unknown`; the profile draws the line
Antora's region is `<article class="doc">` (I9). `generic` takes `<main>` or `[role=main]`, and
records `unknown` on a page with neither, so a heuristic guess never poses as a fact and the
chrome report says only what it knows. Rejected: everything `content` when no landmark exists.

### D35 A root's profile defaults to `auto`, which picks one per page by its generator meta
`profile = "auto" | "generic" | "antora"` per root. Under `auto`, `<meta name="generator">` picks
the profile, falling back to `generic` (Docsy and Sphinx emit none, I12), and the page row stores
the result. The meta precedes anything a profile extracts. Rejected: deciding once per root, which
mislabels a root spanning generators; config only, which makes every Antora user opt in.

### D36 The page → source tie-back is core: an edit-link heuristic and forge URL shapes
Every generator's edit link points at a forge, so any site gets page → file with no config. The core
ranks the `<a href>`s earning a tier: `rel=edit`, an edit phrase in `title` or text, a known class
(I12); inside a tier a forge-shaped URL, then document order. A link in no tier is none. docs.fp.o
needs no override (I8): its title is a phrase. `pages` keeps `source_url`, plus `source_repo`,
`source_ref` and `source_path` when a path shape parses on any host, the ref one segment (KD1).
Rejected: discovery in the Antora profile only, which leaves CI's build → repo map (D9, D17) on one
generator; a `title` match in `antora`, a docs.fp.o quirk in a generator profile (D32).

### D37 Profile facts live in one core table, `page_facts`, under namespaced keys
`(run_id, url_id, key, value)`, keys like `antora.component`. A profile ships views pivoting its
keys into columns, and D19 resolves through Antora's. D33's rules can write only a key and a value.
Rejected: a typed table per profile (a migration story each); a JSON column (no typed joins, no
simpler). Reopens: D19 slow over the view, fixed by materializing that view alone.

### D40 A run needs no config: the URL given is the root, and every other setting has a default
Without a file, the URL is the only root, scoped to its path prefix. Its sitemap comes from
robots.txt `Sitemap:` lines, then `sitemap.xml` under the root, then at the host. Profile `auto`
(D35), robots.txt and its `Crawl-delay` obeyed (D18), 2 req/s per host otherwise, no allowlist. A config
overrides each. Rejected for now: a required config, and `bruce init`, until someone asks for it.

### D41 This repo builds the tool and never deploys it
A deployment (docs.fp.o's config, its nightly crawl, where its baseline is published) lives with
its deployer. The manual documents how to configure one, and a gitignored simulated deployment here
verifies that the tool supports it. Rejected: a shipped `fedora.toml`, or a preset compiled in,
either of which puts a deployment into the tool's release.

### D42 The simulated deployment is the manual's deployment guide, executed
`mise run sim` extracts the guide's tagged code blocks into a gitignored `sim/` and runs them, so a
guide that drifts from the tool fails there, and any machine rebuilds it. docs.fp.o's config exists
here only as that guide's worked example. Rejected: a committed template directory, which drifts
from the guide describing it; a hand-built `sim/`, which no other machine can rebuild.

### D38 A profile has four hooks: detect, extract, resolve, views
Detect claims a generator string (D35). Extract hands the core's one lol_html pass (D11) a declared
content region (selectors, D34), element handlers and a finish step run over the core's settled
result; handlers write a sink: a fact, a tag on the link being read (I6), a preferred edit link (D36).
Resolve maps a tagged reference to a URL through the store's lookup (D19, D20, D22). Views is its SQL
(D37). `generic` leaves Resolve and Views empty, having no reference that is not a URL and no facts to
pivot. Rejected: region handlers in each profile, each re-deciding `unknown` (D34), which a page's end
alone settles. Reopens: a hook empty in `generic` for any other reason, the trait bending around Antora.

### D39 The layering: `cli` › `crawl`, `ci`, `report` › `fetch`, `extract`, `store` › `profile` › `forge`, `url`, `config`
Edges point down only. `store` implements `profile`'s lookup trait and asks each profile for its
views, so a profile never reaches the DB. Only `cli` names a concrete profile, to build the
registry. Rejected: `profile` above `extract`, driving the pass, which makes the core a library of
the profile, the inverse of D32.

## Storage and history

### D2 Runs are append-only: every row carries a `run_id` and nothing is overwritten
This answers when a link broke and which upstream change did it, and allows run diffs. Old runs are
pruned explicitly (`bruce prune`). Rejected: latest-state upsert, which cannot date a breakage.

### D15 Pages store metadata, a content hash and their profile's facts; bodies only on request
The hash detects change because validators are unreliable (I3). Profile facts (D37), Antora's
component, version and module among them, group reports by what owns a page. Bodies go in an opt-in, prunable table. Rejected: storing
bodies by default (about 140 MB per en-US run, uncompressed).

### D17 Each page stores its source file and repo, read from its edit link
This routes a breakage to the repo and file that own it, maps CI builds to repos, and groups more
precisely than component (some repos supply several). Finding the link is core (D36); docs.fp.o's
markup is I8.

### D26 A URL's identity is a u64 hash of its canonical form, stable across runs
Cross-run joins then need no URL comparison. `links` keeps one row per occurrence, raw href
included, so a report can point at the link in its source. Tables: `runs`, `targets`, `pages`,
`page_facts` (D37), `anchors`, `links`, and opt-in `bodies`. The drafted columns are in
`historical/initial.md`, "D26", as D34, D36 and D37 amend them.
Rejected: deduplicated links with a count, which is smaller but loses the raw href.

### D43 A DB records its schema version, and `bruce` refuses a DB of another version
A `meta` table holds `schema_version`, and the refusal names both versions. Before a release every
DB is ours and disposable, so migrations wait for the first release (P6). Rejected: recreating on
mismatch, which silently destroys the history D2 keeps; migrations from day one, for no user.

### D28 Fragment verdicts are computed in SQL after the crawl, not during it
The crawler holds no anchor state and need not order fetches around targets. The same join feeds
the reports. One writer feeds DuckDB appenders through a bounded channel, DuckDB allowing one
writing process per file. Rejected: checking fragments in memory as pages arrive.

## Classification

### D7 Canonical URLs keep the fragment apart; redirects are followed and stated
Lowercase the scheme and host, drop default ports, percent-decode where safe, keep the query (per
root it may be stripped), and keep `/foo` and `/foo/` distinct. Follow up to 10 hops; a loop or more
hops is `broken_url`. The fragment is checked against the final page, and `redirected` is a
non-failing modifier (I4, I5). Rejected: normalising the trailing slash, which servers do not.

### D12 A fragment is valid where a browser would scroll to it
Valid: any non-empty `id`, `name` on `<a>` only, and always `#` and `#top`. Matching is exact,
after percent-decoding. Each anchor stores its tag and its placement (D34), so links to
template chrome are reportable (I9). Text fragments are `unchecked`.
Rejected: every `name` attribute, which makes `#viewport` and `#generator` pass on every page.

### D16 Five outcomes; `error` is separate from broken and never fails CI alone
`ok`, `broken_url` (4xx except below, NXDOMAIN, redirect loop), `broken_anchor`, `error` (timeouts,
TLS, 5xx after 3 retries with backoff, and 401/403/429 with status kept), `unchecked`. Rejected:
`error` folded into `broken_url`, which fails CI on a flaky proxy.

### D20 An unresolved xref is a broken page reference, never a same-page fragment
Live crawl: `broken_url`, reason `unresolved_xref`, raw refspec kept, with a "did you mean" hint
from a fuzzy match over resource IDs, run only on failures. CI: resolved via D19 first (I6).
Rejected: checking `#comp::page.adoc` against the page's own ids, which is what a naive checker does.

## Antora and CI

### D4 CI resolves targets outside its build against a published baseline, then live
A scheduled full crawl publishes a baseline holding each page's anchor set. CI also runs the
inverse check: pages in other repos linking to anchors this change removes. Rejected: live-only
(slow and non-deterministic), and skipping cross-component links (the breakage this tool exists for).

### D9 CI fails only on breakage the change introduced
A link is `(source URL, target URL, fragment)`, with the build dir mapped to production URLs. It is
introduced if that link is not broken in the baseline. The inverse check fails by default and can
be configured down to a warning. Renamed pages make all their links look new; that noise is
accepted. Rejected: failing on all breakage, which keeps CI permanently red.

### D19 Pages carry Antora resource IDs, and unresolved refspecs resolve through them
The Antora profile records `(component, version, module, rel_path)` as facts (D37): module and
`rel_path` from the edit URL (I8); component and version, verbatim, from the nav container, or on a
page with none the component from the URL under the site root (I13) and no version. A refspec
resolves relative to its page by Antora's rules; a placeholder or absent version (I8) means the
component's only or latest version in the DB. Rejected: inferring the module from the URL
(ambiguous with subdirectories); no component without a nav container, leaving xrefs into it
unresolvable; a version from the URL, as no versioned component lacks a nav container.

### D21 CI consumes a single-run baseline file, not the history DB
`bruce export-baseline` writes one run as a compact DuckDB file. CI takes `--baseline <path|url>`.
A deployment's scheduled job produces it, and where it is hosted is the deployment's (D41). Rejected:
shipping the full history DB to CI.

### D22 A local build's links resolve locally, then against the baseline, then by allowlist
A relative href is resolved against the page's production URL. Under the prefix and built: check
locally (`foo/` → `foo/index.html`). Under the prefix and not built: baseline, then live. An
unresolved xref: D19, then the same steps. Externals default to the baseline's verdict
(`--externals=skip|baseline|live`), so a slow host never blocks a PR.

## Operation

### D5 Configuration is optional TOML, with CLI overrides; roots are separate from the allowlist
`[[root]]` entries are crawled fully, each with include/exclude globs and its own sitemap URL.
`allowlist` lists hosts checked one hop deep, `[rate]` holds the default limit and `[host."<h>"]`
a host's own `per_second` and `robots` (D8). Each root names its profile (D35). Roots are
configurable so the tool is not tied to Fedora, and every key has a default (D40). Keys are
snake_case and `[[root]]` singular, as Cargo's `[[bin]]`; rejected: kebab-case, which lychee and
mise, the nearest tools, do not use; `allowlist` inside `[host]`, since a root's own host belongs
there and in no allowlist, so the two answer different questions.

### D44 A root's scope is its path prefix until `include` names globs, which match the whole URL
`include` replaces the prefix rather than narrowing it, so one root can span locales; `exclude`
always wins. Globs are globset's with `*` and `?` stopping at `/`, so a version segment matches as
`f2?`. Rejected: globs over the path alone, which cannot scope by host; regexes, which users write
worse; `include` narrowing the prefix, which keeps a root at `/en-US/` in one locale. Reopens: a
root needing both its prefix and an include, today written as one glob. A root's `url` must be in
its own scope, refused at load, so a crawl never starts on a page it would not keep.
Code: `config::Root::in_scope`. Evidence: `config::tests::include_replaces_the_prefix_and_exclude_wins`.

### D6 Reports are SQL views stored in the DB and run by `bruce report <kind>`
Ad-hoc `duckdb` users see the same views as the CLI. Output is text, or `--format json`.

### D8 Rate limits, concurrency and robots.txt are per host; only a root's host may skip robots.txt
Defaults in config (D40), with per-host settings in `[host]`. Back off on 429/503 and honour
`Retry-After`. robots.txt is always honoured for external hosts and switchable only for a host a
root starts on, refused at load otherwise. Every run refetches every page (I3). Rejected: one
global rate, which throttles the root to the slowest host's courtesy; `robots` on a root, where two
roots on one host could disagree and a host an `include` reaches had no setting.

### D18 A host's `Crawl-delay` is obeyed unless a root's host overrides it with `per_second`
The default (D40). For docs.fp.o (I2), a nightly baseline does not need speed. A `per_second`
written for a host a root starts on beats `Crawl-delay`: a faster crawl is an explicit override by
whoever has permission. On any other host the slower wins, and the default rate never beats it.
Rejected: a `crawl_delay` key, a per-host rate in other units; `robots = false` alone passing
`Crawl-delay`, when docs.fp.o wants its disallows obeyed. Performance work targets the local-dir
mode and the pipeline.

### D27 One binary, `bruce`, with subcommands `crawl`, `check-dir`, `report`, `export-baseline`, `prune`
`check-dir` writes its results to a local DB too, so reports work on CI runs, and its exit status
follows D9 and D16. The drafted flags are `historical/initial.md`, "D27".

### D30 The name is `bruce`, after the Bruce claw anchor
It was free on crates.io on 2026-10-17. `anchorage` (D25) was taken there.

## Build

### D11 Our own stack, not lychee-lib; lol_html is the tokenizer
reqwest, governor, lol_html and duckdb. What is ours (crawl frontier, link graph, anchor semantics)
lychee does not model, and its extractor would need working around (D12's rejection). lol_html and
html5gum tied on the spike's workload (every fixture equal, speed within noise); lol_html matches
CSS selectors while streaming, which profiles and D33's rules want. Rejected: html5gum (2 crates to
lol_html's 42), which leaves us an open-element stack and a selector matcher to write. lol_html's
other costs were weighed and accepted: undecoded attributes and text, and context only by handler order.
Evidence: `historical/initial.md`, "Existing tools"; `spikes/tokenizer/` at commit b436db8, its numbers in P1's notes.

### D23 DuckDB is bundled; distribution is a release binary on the forge
The `duckdb` crate's `bundled` feature makes a self-contained binary that a CI job in the
Fedora-minimal docs image fetches in one step. Rust is pinned in `mise.toml`. Rejected for now:
container image, RPM. Its cargo tasks sit beside `check`, which runs the same commands with its own
flags. Rejected: `check` calling the tasks, which would make `mise run fmt` stop writing or lose
the check's summaries.

### D31 One crate, `bruce`; its concepts are separated by top-level module, not by crate
The binary is the only consumer, so a crate boundary buys nothing a module boundary does not.
Segregation is still enforced: each concept in one place (`concepts.md`), module edges one-way
(`module_graph.py`), files under `file_limits.py`'s caps. The layering between top-level modules is
D39. Rejected: a workspace (core / I/O / CLI crates).
Reopens: a second consumer of the core, or the evidence the `seams` skill's §4 names.

### D24 Tests run on saved real pages, a mock HTTP server, and an optional Antora build
Fixture pages from docs.fp.o (I6, I8, I9), hand-written pages for other generators' markup (I12),
wiremock for redirects, 429 and robots, and a criterion
parser bench that `mise run check` runs once, in test mode, as a nextest test; `mise run bench`
times it. An end-to-end 2–3-component Antora site sits behind a task, as it is slow. Rejected: a
timing threshold in `check`, flaky at the spike's 8% run-to-run noise and tied to one machine.
