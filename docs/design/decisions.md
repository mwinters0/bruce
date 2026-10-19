# Decisions

The decisions the code cannot explain: shapes kept for a deliverable not yet built, defaults chosen
over an alternative, and obvious changes argued and refused. Nothing here says how the code works
(the named module does). External facts are cited by `I<n>` from `site-invariants.md`. Cite an
entry as `docs/design/decisions.md`, "D12"; the rest of the rules, the line cap included, are
`docs/process.md`, "The decision register".

The modules are P1's scaffold, and `cli`, `config`, `store`, `profile`, `extract`, `forge`, `url`, `fetch`, `crawl` and `report` hold code. D1–D30 were settled
in the bootstrap grilling (2026-10-17, frozen as `historical/initial.md`), D31 the same day, D32 onward in P1's grilling, and
an entry's **Code** is the module that will hold it once built. D25 and D29 are spent: D25 was superseded by D30, and D29
became `roadmap.md`'s phase index.

<!-- decision-watermark: D50 -->

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
each link carrying its `kind`; no page yields `sitemap` (an entry) or `sitemap_file` (a file, D47).
Assets get a HEAD request with GET fallback; reports default to `kind = anchor`. `mailto:`, `tel:`,
`javascript:` and `data:` are `unchecked`. A `<link>` whose `rel` is only `preconnect` or
`dns-prefetch` is skipped; the first `<base href>` resolves every href after it. Rejected: `<a href>`
only, which misses broken images and canonicals; ignoring `<base>`, which resolves a site wrongly.

### D14 Discovery is sitemap and link-following both, and each page records which
Each root's sitemap is discovered (D40) or set (docs.fp.o's is under its en-US root, I1). Recording both
sources yields orphans (in the sitemap, with no inbound link) and unlisted pages (reachable, absent
from the sitemap). Rejected: either source alone, which loses one of those reports.

### D47 A run reads its sitemaps before any page, and each root links to the files it read
A listed page is then claimed by its sitemap before a link can name it, so `pages.in_sitemap` holds
for every listed page; the files (69 at docs.fp.o) cost about a minute on a pace-bound run. Each
file a root read is a `sitemap_file` link from it, so a broken one shows as any broken link and "no
sitemap" is per root; a probe that misses writes nothing, nothing having named it. Rejected: reading
sitemaps beside the pages (`in_sitemap` left to a race); a target per failed probe; a `runs` column,
per run and naming no broken file. Code: `crawl::Shared::discover`.
Evidence: `crawl::tests::each_root_links_once_to_each_sitemap_it_read`.

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
Code: `scripts/sim.py`. Evidence: `scripts/test_sim.py`.

### D38 A profile has four hooks: detect, extract, resolve, views
Detect claims a generator string (D35). Extract hands the core's one lol_html pass (D11) a declared
content region (selectors, D34), element handlers and a finish step run over the core's settled
result; handlers write a sink: a fact, a tag on the link being read (I6), a preferred edit link (D36).
Resolve maps a tagged reference to a URL through the store's lookup (D19, D20, D22). Views is its SQL
(D37). `generic` leaves Resolve and Views empty, having no reference that is not a URL and no facts to
pivot. Rejected: region handlers in each profile, each re-deciding `unknown` (D34), which a page's end
alone settles. Reopens: a hook empty in `generic` for any other reason, the trait bending around Antora.

### D39 The layering: `cli` › `crawl`, `ci`, `report` › `fetch`, `extract`, `store` › `profile` › `forge`, `config` › `url`
Edges point down only. `config` reads `url`, its scope matching the canonical form (D44). `store`
implements `profile`'s lookup trait and asks each profile for its views, so a profile never reaches
the DB. Only `cli` names a concrete profile, to build the registry. Rejected: `profile` above
`extract`, driving the pass, which makes the core a library of the profile, the inverse of D32;
scope moved to `crawl` to keep `config` lowest, splitting it and losing D44's refusal at load; a tier
check in `module_graph.py`, its cycle check catching nearly every upward edge. Reopens: a skip-tier
edge passing review.

## Storage and history

### D2 Runs are append-only: every row carries a `run_id` and nothing is overwritten
This answers when a link broke and which upstream change did it, and allows run diffs. Old runs are
pruned explicitly (`bruce prune`). Rejected: latest-state upsert, which cannot date a breakage.

### D15 Pages store metadata, a content hash and their profile's facts; bodies only on request
The hash detects change because validators are unreliable (I3). Profile facts (D37), Antora's component, version and module
among them, group reports by what owns a page. Bodies go in an opt-in, prunable table. The hash is xxh3-128 (not adversarial;
faster than BLAKE3) over tags, attributes and decoded text in a scope `hash_root`/`hash_exclude` set, by default the profile's
region; a page matching no root is hashed whole, less exclusions. Rejected: storing bodies by default (about 140 MB per en-US
run, uncompressed); raw bytes (docs.fp.o's `Last build:` changes every page every build); one fixed default root (an exclusion
alone would barely act); an empty or region fallback for a set root matching nothing, hiding the misconfiguration a page changing
every build shows. Code: `extract::hash`. Evidence: `tests/extract.rs`, the `Last build:` test.

### D17 Each page stores its source file and repo, read from its edit link
This routes a breakage to the repo and file that own it, maps CI builds to repos, and groups more
precisely than component (some repos supply several). Finding the link is core (D36); docs.fp.o's
markup is I8.

### D26 A URL's identity is a u64 hash of its canonical form, stable across runs
Cross-run joins then need no URL comparison. `links` keeps one row per occurrence, raw href
included, so a report can point at the link in its source. Tables: `runs`, `targets`, `pages`,
`page_facts` (D37), `anchors`, `links`, and opt-in `bodies`. The drafted columns are in
`historical/initial.md`, "D26", as D34, D36 and D37 amend them. An href resolving to no URL is a
target keyed by its raw text (`url` that text, `broken_url`, `invalid_url`), one per text per run.
Rejected: deduplicated links with a count, which is smaller but loses the raw href; for an
unresolvable href, no link row (reported nowhere) or a crawl-time `links.outcome` and no target.

### D43 A DB records its schema version, and `bruce` refuses a DB of another version
A `meta` table holds `schema_version`, and the refusal names both versions. Before a release every
DB is ours and disposable, so migrations wait for the first release (P6). Rejected: recreating on
mismatch, which silently destroys the history D2 keeps; migrations from day one, for no user.

### D28 Fragment verdicts are computed in SQL after the crawl, not during it
The crawler holds no anchor state and need not order fetches around targets. The verdict is a view,
`link_verdicts`, computed at each read, so a stopped run needs no second path and no stored column
can disagree with it; the reports and P5's gate read the same join. One writer feeds DuckDB appenders
through a bounded channel, DuckDB allowing one writing process per file. Rejected: checking fragments
in memory as pages arrive; a table written at a run's end. Reopens: a read slow enough to materialise
(0.1 s over 587839 links, 172239 targets). Code: `store::views`. Evidence: `store::views::tests`.

## Classification

### D7 Canonical URLs keep the fragment apart; redirects are followed and stated
Lowercase the scheme and host, drop default ports, percent-decode only unreserved bytes (a `%2F`
must not move the path), keep the query (per root it may be stripped), and keep `/foo` and `/foo/`
distinct. Follow up to 10 hops; a loop or more
hops is `broken_url`. The fragment is checked against the final page, and `redirected` is a
non-failing modifier (I4, I5). Rejected: normalising the trailing slash, which servers do not.
Code: `url::canonical`. Evidence: `url::tests`.

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

### D46 Where a redirect lands is a target of its own, written by whoever claims it first
A `final_url` then names a `targets` row bar a stopped run's, so P4's fragment join is two key lookups
from the link to the final page's `anchors`. The redirecting target is claimed at discovery, its landing
URL after the hops end; the first claim writes the landing target and, if it reads it, its page, a loser
still has its own answer from the chain. Rejected: holding the hops open on the landing URL's own task,
which makes one target's outcome wait on another's; a memo of settled targets, to save a repeat request
to a landing URL already claimed. Reopens: that repeat mattering at docs.fp.o's pace.
Code: `crawl::Shared::write`. Evidence: `crawl::tests::a_redirect_is_a_chain_and_its_landing_page_is_recorded_once`.

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
resolves by Antora's rules; a placeholder or absent version (I8) means the DB's only or latest one.
Rejected: the module from the URL (ambiguous with subdirectories); no component without a nav
container (xrefs into it unresolvable); a version from the URL (no versioned component lacks one); the
fallback keyed on a nav container lacking `data-component`, inert where that happens (no lunr, I12).

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

### D5 Configuration is optional TOML, and no flag overrides a key; roots are separate from the allowlist
`[[root]]` entries are crawled fully, each with include/exclude globs, its own sitemap URL and its
profile (D35); `allowlist` lists hosts checked one hop deep, `[rate]` holds the default limit and
`[host."<h>"]` a host's own `per_second` and `robots` (D8). Roots are configurable so the tool is not
tied to Fedora, and every key but a root's `url` has a default (D40). `bruce crawl`'s URL stands in
for a file's absent roots and nothing else; per-key flags wait for someone to ask (`cli::crawl`). Keys
are snake_case and `[[root]]` singular, as Cargo's `[[bin]]`. Rejected: kebab-case, which lychee and
mise do not use; `allowlist` inside `[host]`, a root's own host belonging there and in no allowlist.

### D44 A root's scope is its path prefix until `include` names globs, which match the whole canonical URL
`include` replaces the prefix rather than narrowing it, so one root can span locales; `exclude` always
wins. Globs are globset's, `*` and `?` stopping at `/`, so a version segment matches as `f2?`. They
match the canonical form (D7), so a glob written with an unreserved character (`~`) matches its
escape too, and one written with an escape the form decodes (`%7E`) never does. Rejected: globs over the path alone,
which cannot scope by host; regexes; `include` narrowing the prefix. Reopens: a root needing both
its prefix and an include. A root's `url` must be in its own scope, refused at load.
Code: `config::Root::in_scope`. Evidence: `config::tests::scope_is_matched_on_the_canonical_form`.

### D50 A config `skip` list makes a matching link `unchecked`, reason `skipped`, before scope and the allowlist
Top-level globs on the plain canonical form (D44's, the query kept), any host, a root's scope included:
the deployer said not to ask. A hop onto one ends the chain, a sitemap's too, as a robots.txt-forbidden
hop does; a root's `url` matching one is refused at load. Why: the forge answers `issues/new` with a 303
to `/user/login` for any repo, a false `ok`, and skipping `/user/login*` marks every link sent there.
Rejected: detecting a login from a 401 or a login-like redirect, which misfires elsewhere; learning it
from the first logins; `*` crossing `/` in a query, or the query unmatched, which forks D44's globs.
Code: `crawl::frontier::Frontier::hop_plan`. Evidence: `crawl::tests::a_redirect_onto_a_skipped_url_ends_the_chain`.

### D6 Reports are SQL views stored in the DB and run by `bruce report <kind>`
Ad-hoc `duckdb` users see the same views as the CLI. Output is text, or `--format json`. A view
takes no parameter, so each `report_*` view holds every run and `bruce report` binds the run and the
filters in its query; `cli` installs them at every open, `store` being below `report` (D39).
Rejected: a view per run; installing them only when `bruce report` runs, which hides them from
`duckdb` after a crawl; a JSON shape carrying `stats`' totals or `unlisted`'s roots with no sitemap,
until a consumer needs one, a script reading `link_verdicts` or `report_roots` meanwhile. Code:
`report`, `cli::db`.

### D8 Rate limits, concurrency and robots.txt are per host; only a root's host may skip robots.txt
Defaults in config (D40), with per-host settings in `[host]`. Back off on 429/503 and honour
`Retry-After`. robots.txt is always honoured for external hosts and switchable only for a host a
root starts on, refused at load otherwise. Every run refetches every page (I3). A host's robots.txt
follows RFC 9309: a 4xx allows all; a 5xx, no answer or a 429 outlasting the retries (which says nothing
of the file) closes the host for the run. Rejected: one global rate, which throttles the root to the
slowest host's courtesy; `robots` on a root, where two roots on one host could disagree and a host an
`include` reaches had no setting. Code: `fetch::http::host`. Evidence: `fetch::http::host::tests`.

### D18 A host's `Crawl-delay` is obeyed unless a root's host overrides it with `per_second`
The default (D40). A `per_second` written for a host a root starts on beats `Crawl-delay`: a faster
crawl is an explicit override by whoever has permission. On any other host the slower wins, and the
default rate never beats it. The guide's docs.fp.o example (I2) sets `robots = false` with `per_second = 1`,
D8's switch for a root's host: its `Disallow`s, written for search engines, hid nine live 404s. A deployer
controlling the site writes a `robots.txt` group for its agent instead. Rejected: a `crawl_delay` key;
`robots = false` alone passing `Crawl-delay`, reading the file it switches off. Evidence:
`config::tests::the_guide_turns_robots_off_for_the_root_host_alone`.

### D48 A stop drops each target's hops 30 s after it; "cut" is every target found and not written
Every started target's hops end at the same instant, the stop plus `STOP_GRACE`, and one whose hops
ended still writes whole, so a stopped run holds whole targets and its counts are exact. "Cut" is
claimed less written, the queued included: a lower bound on what the run lacks, unfound URLs not in
it. Rejected: aborting the tasks in `drive`, which cannot reach a write already on the blocking pool,
its rows landing after `drive` returns; counting only targets started and dropped, which says how the
stop went, not what the run lacks. Code: `crawl::Shared::in_grace`, `crawl::Summary::cut`.
Evidence: `crawl::tests::a_target_unfinished_at_the_grace_is_cut`.

### D49 An abandoned run waits for the writes it has begun, and only for them
The second Ctrl-C closes the run's `Writes`: a write not yet begun sends nothing, and the close blocks,
unbounded, until each begun has sent its last row, the writer draining meanwhile. Rejected: dropping
the runtime, whose wait covers all blocking work, an extraction or a parse nobody reads and whatever a
dependency adds (a resolver off hickory runs `getaddrinfo` there), and whose skipping of queued work
tokio does not guarantee; a bound on the wait, which can only end it mid-target; a third press, a
wedged DuckDB being a bug to fix. Code: `crawl::Writes`, `cli::crawl::crawl`.
Evidence: `crawl::tests::an_abandoned_run_holds_the_page_it_was_writing_whole`.

### D27 One binary, `bruce`, with subcommands `crawl`, `check-dir`, `report`, `export-baseline`, `prune`
`check-dir` writes its results to a local DB too, so reports work on CI runs, and its exit status
follows D9 and D16. The drafted flags are `historical/initial.md`, "D27".

### D30 The name is `bruce`, after the Bruce claw anchor
It was free on crates.io on 2026-10-17. `anchorage` (D25) was taken there.

## Build

### D11 Our own stack, not lychee-lib; lol_html is the tokenizer
reqwest, texting_robots, lol_html and duckdb, a host's pace its own next-allowed instant (`fetch::http::host`),
governor having no per-key quota (boinkor-net/governor#193). What is ours (crawl frontier, link graph,
anchor semantics) lychee does not model, and its extractor would need working around (D12's rejection).
lol_html and html5gum tied on the spike's workload; lol_html matches CSS selectors while streaming,
which profiles and D33's rules want. Rejected: html5gum (2 crates to lol_html's 42), which leaves us an
open-element stack and a selector matcher to write; a governor limiter per host, rebuilt as a host's rate changes.
Evidence: `historical/initial.md`, "Existing tools"; `spikes/tokenizer/` at commit b436db8, its numbers in P1's notes.

### D23 DuckDB is bundled; distribution is a release binary on the forge
The `duckdb` crate's `bundled` feature makes a self-contained binary that a CI job in the
Fedora-minimal docs image fetches in one step. Rust is pinned in `mise.toml`. Rejected for now:
container image, RPM. Its cargo tasks sit beside `check`, which runs the same commands with its own
flags. Rejected: `check` calling the tasks, which would make `mise run fmt` stop writing or lose
the check's summaries.

### D45 TLS is rustls on ring and DNS is hickory's, so a failure arrives typed
reqwest's default `rustls` feature builds aws-lc-rs, a C build, against D23's one self-contained
binary for the Fedora CI image; `rustls-no-provider` plus ring avoids it. hickory-dns makes NXDOMAIN
(`broken_url`, D16) a `NetError` down the error chain, where the system resolver gives a message
string. Cost: about 30 crates, and a resolver reading `resolv.conf`
itself. Rejected: aws-lc-rs; matching the system resolver's message. Reopens: ring failing to build
on the CI image. Code: `fetch::http`. Evidence: `fetch::http::tests::the_cause_is_found_down_the_chain`.

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
