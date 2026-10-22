# Decisions

The decisions the code cannot explain: shapes kept for a deliverable not yet built, defaults chosen
over an alternative, and obvious changes argued and refused. Nothing here says how the code works
(the named module does). External facts are cited by `I<n>` from `site-invariants.md`. Cite an
entry as `docs/design/decisions.md`, "D12"; the rest of the rules, the line cap included, are
`docs/process.md`, "The decision register".

The modules follow D39's layering. D1–D30 were settled in the bootstrap grilling (2026-10-17,
frozen as `historical/initial.md`), D31 the same day, D32 onward in the phases' grillings and slices, and an entry's
**Code** is the item that holds it. D25 and D29 are spent: D25 was superseded by D30, and D29 became `roadmap.md`'s
phase index.

<!-- decision-watermark: D59 -->

## Scope and modes

### D1 Two fetchers behind one trait: live HTTP crawl first, local build directory second
Parsing and the DB writer are shared. The local-dir mode exists for CI verification of a docs change
before it deploys. A full site build takes hours, so nobody runs one locally. Rejected: local-only,
which cannot see the deployed site or external hosts; a CI pipeline of its own walking the build and
querying the baseline, which rebuilds the frontier, the hops (a local alias is one), D16 and the writer.
Reopens: a published full-site build artifact. Code: `cli::check_dir`, `fetch::routed::Routed`.

### D3 Allowlisted external hosts are checked one hop deep and never crawled
An allowlisted target gets a full GET, so its fragment can be checked. A link to any other host is
stored as `unchecked` with no request, keeping the link graph whole. Rejected: crawling allowlisted
hosts, which would make them roots by another name (D5 holds roots).

### D10 Default scope: en-US, every fedora version; locales are a root that is off by default
Frozen f26–f43 pages are cheap and current pages link into them, so their anchors must be known. Translated
heading ids can break fragments, but the translation team owns that (I9). A URL on a root's host in no root's
scope, unless allowlisted, is `unchecked`, reason `out_of_scope`, with no request. Rejected: latest-only, which
leaves inbound links to frozen versions unverifiable; such a URL one hop deep, the language switcher's 67 other
locales costing 67 requests a page (`grep -o 'class="language" href' fixtures/docs-fp-o/en-US/a11y/index.html`,
less its own). Code: `crawl::frontier::Frontier::plan`.

### D13 Every reference kind is extracted, each target fetched once per run
`<a>`, `<area>`, `<img>`/`srcset`, `<link>` (canonical included), `<script>`, `<iframe>`, media
(`src` on `<video>`, `<audio>`, `<source>`, `<track>`; `poster`) and the `url=` of a meta refresh,
each link carrying its `kind`; no page yields `sitemap` (an entry) or `sitemap_file` (a file, D47).
Assets get a HEAD request with GET fallback; reports default to `kind = anchor`, the gate (D9) to every
kind. `mailto:`, `tel:`, `javascript:` and `data:` are `unchecked`. A `<link>` whose `rel` is only
`preconnect` or `dns-prefetch` is skipped; the first `<base href>` resolves every href after it. Rejected:
`<a href>` only (misses broken images, canonicals); ignoring `<base>` (resolves a site wrongly).

### D14 Discovery is sitemap and link-following both, and each page records which
Each root's sitemap is discovered (D40) or set (docs.fp.o's is under its en-US root, I1). Recording both
sources yields orphans (in the sitemap, with no inbound link) and unlisted pages (reachable, absent
from the sitemap). Each page records the root whose scope claimed it, and each run its roots, both
canonical, so a root is its own target's `url`. Rejected: either source alone, which loses one of those
reports; matching a page to its root by scope in SQL, which `include` globs (D44) defeat; the root as
configured, which an escape or a stripped query fails to match. Code: `report::views`, `store::rows`.

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
generator links to a forge. Rejected: a `fedora` profile above `antora`, empty on day one; a report
`--component` filter, Antora in the core, where `--fact antora.component=…` reads `page_facts` (D37).
Reopens: a deployment quirk that config and profile settings cannot express.

### D33 Profiles are compiled in; declarative per-root rules come later, as their own phase
A profile is a module behind a trait, so a new generator is a PR. Facts a profile records are
key/value strings, so a TOML rule (selector + attribute, region selector, URL regex) can produce
the same facts later without a schema change. Resolution logic stays code. Rejected now: WASM or
out-of-process plugins (an ABI, sandbox and versioning burden for no second user).

### D34 An anchor's placement is `content`, `chrome` or `unknown`; the profile draws the line
Antora's region is `<article class="doc">` (I8). `generic` takes `<main>` or `[role=main]`, and
records `unknown` on a page with neither, so a heuristic guess never poses as a fact and the
chrome report says only what it knows. Rejected: everything `content` when no landmark exists;
`hash_root` setting the region too, a root of `html` putting every chrome id in `content` (D15), a
per-root region being D33's rules. Code: `extract`, `extract::hash`.

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
Rejected: discovery in the Antora profile only, which leaves routing a breakage to its repo (D17) on
one generator; a `title` match in `antora`, a docs.fp.o quirk in a generator profile (D32).

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
`mise run sim` extracts the guide's tagged code blocks into a gitignored `sim/` and checks its config
loads, so a guide that drifts from the tool fails there, and any machine rebuilds it. docs.fp.o's config
exists here only as that guide's worked example. Rejected: a committed template directory, which drifts
from the guide describing it; a hand-built `sim/`, which no other machine can rebuild; running the
crawl there, a two-hour job at every sim.
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

### D2 Runs are append-only: every row but `meta`'s carries a `run_id`, and only a run's `finished_at` is updated
This answers when a link broke and which upstream change did it, and allows run diffs. Old runs are
pruned explicitly (`bruce prune`). A stopped run is never resumed: its rows stay, `finished_at` NULL,
and the next starts afresh. Rejected: latest-state upsert, which cannot date a breakage; a persisted
frontier to resume from, much machinery for a job that fits in a night.

### D15 Pages store metadata, a content hash and their profile's facts; bodies only on request
The hash detects change because validators are unreliable (I3). Profile facts (D37) group reports by what owns a page. Bodies go
in an opt-in, prunable table. The hash is xxh3-128 (not adversarial; faster than BLAKE3) over tags, attributes and decoded text
in a scope `hash_root`/`hash_exclude` set, by default the profile's region; a page matching no root is hashed whole, less
exclusions. Rejected: storing bodies by default (about 140 MB per en-US run, uncompressed); raw bytes (docs.fp.o's `Last build:`
changes every page every build, I19); a hash of the extracted record (title, anchors, links), blind to a prose edit; one fixed
default root (an exclusion alone would barely act); an empty or region fallback for a set root matching nothing, hiding the
misconfiguration a page changing every build shows. Code: `extract::hash`. Evidence: `tests/extract.rs`, the `Last build:` test.

### D17 Each page stores its source file and repo, read from its edit link
This routes a breakage to the repo and file that own it, and groups more precisely than component
(some repos supply several). A CI build's pages carry none (I15), so CI maps a build by its
component-versions (D54, D56). Finding the link is core (D36); docs.fp.o's markup is I8.

### D26 A URL's identity is a u64 hash of its canonical form, stable across runs
Cross-run joins then need no URL comparison. `links` keeps one row per occurrence, raw href
included, so a report can point at the link in its source. Tables: `runs`, `targets`, `pages`,
`page_facts` (D37), `anchors`, `links`, and opt-in `bodies`; the columns are `store::schema`'s. An href resolving to no URL is a
target keyed by its raw text (`url` that text, `broken_url`, `invalid_url`), one per text per run.
Rejected: deduplicated links with a count, which is smaller but loses the raw href; for an
unresolvable href, no link row (reported nowhere) or a crawl-time `links.outcome` and no target.

### D43 A DB records its schema version, and `bruce` refuses a DB of another version
A `meta` table holds `schema_version`, and the refusal names both versions. A 0.x release migrates no
DB: a schema change is named in its release notes and the user starts a new DB, the old binary still
reading the old one, so migrations wait for a release that promises them. Views are outside the
version: `Store::install` replaces each at every open, so a changed view needs no bump and refuses no
baseline. Rejected: recreating on mismatch, which silently destroys the history D2 keeps; migrations
from day one, for no user; a one-off script rewriting an old DB, a migration in all but name, evidence
needing the new schema being crawled afresh. Code: `store::schema`, `store::Store::install`.

### D28 Fragment verdicts are computed in SQL after the crawl, not during it
The crawler holds no anchor state and need not order fetches around targets. The verdict is a view, `link_verdicts`, computed at
each read, so a stopped run needs no second path and no stored column can disagree with it; the reports and the gate (D9) read
the same join. One writer feeds DuckDB appenders through a bounded channel, DuckDB allowing one writing process per file.
Rejected: checking fragments in memory as pages arrive; a table written at a run's end. Reopens: a read slow enough to
materialise (0.2 s warm over 587839 links: run 1 of `runs/p4.10-en-us-20261019-2125/combined.duckdb`, the view as a `TEMP` view
under `.timer on` in `duckdb -readonly`, the DB being schema 3, which `bruce` refuses). Code: `store::views`. Evidence:
`store::views::tests`.

## Classification

### D7 Canonical URLs keep the fragment apart; redirects are followed and stated
Lowercase the scheme and host, drop default ports, percent-decode only unreserved bytes (a `%2F`
must not move the path), keep the query (per root it may be stripped), and keep `/foo` and `/foo/`
distinct. Follow up to 10 hops, a 2xx page's delay-0 refresh one of them (I14); a loop or more hops
is `broken_url`. The fragment is checked against the final page, and `redirected` is a non-failing
modifier (I4, I5). `crawl` follows hops, not `fetch`: a refresh is found only by reading the page.
Rejected: normalising the trailing slash, which servers do not; a delayed refresh as a hop, its page
having content of its own. Code: `url::canonical`, `crawl::hops`. Evidence: `url::tests`.

### D12 A fragment is valid where a browser would scroll to it
Valid: any non-empty `id`, `name` on `<a>` only, and always `#` and `#top` in any case. Matching is exact,
after percent-decoding. Each anchor stores its tag and its placement (D34), so links to
template chrome are reportable (I9). Text fragments are `unchecked`.
Rejected: every `name` attribute, which makes `#viewport` and `#generator` pass on every page.

### D16 Five outcomes; `error` is separate from broken and never fails CI alone
`ok`, `broken_url` (4xx except below, NXDOMAIN, redirect loop), `broken_anchor`, `error` (timeouts,
TLS, 5xx after 3 retries with backoff, and 401/403/429 with status kept), `unchecked`. A `Retry-After`
over 60 s is `error` at once rather than stalling the run. A 2xx body over 16 MiB is `ok`, reason
`too_large`, and not extracted, half a page's anchors giving false `broken_anchor`s. Rejected: `error`
folded into `broken_url`, which fails CI on a flaky proxy; `too_large` as `error`, for a URL that
works, or as a sixth outcome, which every report and the gate would have to place. Code:
`crawl::outcome`, `fetch::http`.

### D20 An unresolved xref is a broken page reference, never a same-page fragment
Live crawl: `broken_url`, reason `unresolved_xref`, raw refspec kept, with a "did you mean" hint
from a fuzzy match over resource IDs, run only on failures. CI: resolved via D19 first (I6).
Rejected: checking `#comp::page.adoc` against the page's own ids, which is what a naive checker does.

### D46 Where a redirect lands is a target of its own, written by whoever claims it first
A `final_url` then names a `targets` row bar a stopped run's, so D28's fragment join is two key lookups
from the link to the final page's `anchors`. The redirecting target is claimed at discovery, its landing
URL after the hops end; the first claim writes the landing target and, if it reads it, its page, a loser
still has its own answer from the chain. Rejected: holding the hops open on the landing URL's own task,
which makes one target's outcome wait on another's; a memo of settled targets, to save a repeat request
to a landing URL already claimed. Reopens: that repeat mattering at docs.fp.o's pace.
Code: `crawl::Shared::write`. Evidence: `crawl::tests::a_redirect_is_a_chain_and_its_landing_page_is_recorded_once`.

## Antora and CI

### D4 CI resolves targets outside its build against a published baseline, then live
A scheduled full crawl publishes a baseline holding each page's anchor set. CI also runs the
inverse check: pages outside the build linking to pages, files or anchors this change removes (D56). Rejected: live-only
(slow and non-deterministic), and skipping cross-component links (the breakage this tool exists for).

### D9 CI fails only on breakage the change introduced
A link is `(source URL, target URL, fragment)` of any kind (D13), with the build dir mapped to
production URLs; only built pages' links count. It is introduced if it is broken here and the
baseline's is not, or is absent. The inverse check fails by default; `--inverse=warn` turns it down (D27).
Accepted noise: a renamed page's links all look new. `report introduced` judges every kind too, so
re-reporting a CI run gives the rows that set its exit. Rejected: failing on all breakage, which keeps
CI permanently red; `check-dir --kind`, narrowing the gate, D50's `skip` silencing a noisy URL instead.
Code: `report::gate`. Evidence: `report::gate::tests::the_gate_reads_every_kind_unless_a_report_narrows_it`.

### D19 Pages carry Antora resource IDs, and unresolved refspecs resolve through them
The Antora profile records `(component, version, module, rel_path)` as facts (D37): module and
`rel_path` from the edit URL (I8); component and version, verbatim, from the nav container, or on a
page with none the component from the URL under the site root (I13) and no version. A refspec
resolves by Antora's rules; a placeholder or absent version (I8) means the DB's only or latest one.
Rejected: the module from the URL (ambiguous with subdirectories); no component without a nav
container (xrefs into it unresolvable); a version from the URL (no versioned component lacks one); the
fallback keyed on a nav container lacking `data-component`, inert where that happens (no lunr, I12).

### D21 CI consumes a single-run baseline file, not the history DB
`bruce export-baseline` writes one finished run into a fresh DB of the current schema, so the views,
reports and D43's refusal read it unchanged and the gate compares like with like (D28). CI takes
`--baseline <path|url>`. A deployment's scheduled job produces it, and where it is hosted is the
deployment's (D41). Its verdicts stand whatever their age; the gate prints the crawl's date.
Rejected: shipping the full history DB to CI; a slim schema of its own, a second version and set of
queries; a `--max-age` refusal, failing a PR for a reason its author cannot fix, until one is asked for.
Code: `store::baseline`.

### D22 A local build's links resolve locally, then against the baseline, then by allowlist
A relative href is resolved against the page's production URL. Under the prefix and built: check
locally (`foo/` → `foo/index.html`). Under the prefix and not built: `broken_url` in a
component-version the build holds, else baseline, then live. An unresolved xref: D19 outside what
the build holds, then the same steps. Externals default to the baseline's verdict
(`--externals=skip|baseline|live`), so a slow host never blocks a PR. Rejected: the baseline for a
held page, which passes a link to a page the change deletes; `--externals` reaching an in-scope URL
the build lacks, a URL on the site new to the gate being the one most worth asking.

## Operation

### D5 Configuration is optional TOML, and no flag overrides a key; roots are separate from the allowlist
`[[root]]` entries are crawled fully, each with include/exclude globs, its own sitemap URL and its
profile (D35); `allowlist` lists hosts checked one hop deep, `[rate]` holds the default limit and
`[host."<h>"]` a host's own `per_second`, `concurrency` and `robots` (D8). Roots are configurable so the tool is not
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
to `/user/login` for any repo (I17), a false `ok`, and skipping `/user/login*` marks every link sent there.
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
The second press closes the run's `Writes`: a write not yet begun sends nothing, and the close blocks,
unbounded, until each begun has sent its last row, the writer draining meanwhile. Rejected: dropping
the runtime, whose wait covers all blocking work, an extraction or a parse nobody reads and whatever a
dependency adds (a resolver off hickory runs `getaddrinfo` there), and whose skipping of queued work
tokio does not guarantee; a bound on the wait, which can only end it mid-target; a third press, a
wedged DuckDB being a bug to fix. Code: `crawl::Writes`, `cli::crawl::crawl`.
Evidence: `crawl::tests::an_abandoned_run_holds_the_page_it_was_writing_whole`.

### D58 SIGINT and SIGTERM are each a press, and a stopped `check-dir` judges nothing
`crawl` and `check-dir` stop at the first signal of either kind (D48) and abandon at the second (D49), and
exit 128 plus the first's number (130, 143). A stopped `check-dir` runs no gate, says on stderr the build was
not judged, and leaves its run unfinished (D2): a gate over a cut build reads what was never written as
absent. GitHub's SIGTERM comes 7.5 s after its SIGINT and its kill 2.5 s after that (I24), inside the
30 s grace, so the second press must abandon. Rejected: one flat 130, which hides which signal ended the run
from a runner's log; judging what was written. Code: `cli::interrupt`, `cli::check_dir::Checked`.
Evidence: `tests/signals.rs`.

### D59 A bare `report` keeps the latest finished run, says of a later unfinished one, and refuses a baseline it imported
Without `--run` a report reads the latest finished run (D2), says on stderr that a later run stopped before
the end, and **refuses** where that run is a later unfinished run's `baseline_run` (D58 leaves such a run):
the rows would be the imported baseline's, read as the build's. `diff`'s default `to` does likewise.
`--run`, `--to` and `--from` read what they name. Rejected: the latest run, finished or not, which would
read a stopped crawl's partial rows over the last whole run; a warning alone, which a CI log buries under
a report that looks whole. Code: `report::run::one`. Evidence: `report::tests::the_default_refuses_the_baseline_of_a_later_unfinished_run`.

### D51 A target records where its answer came from, in a column the writer fills and no view reads
`targets.answered_by` is `local`, `baseline` or `live`, `NOT NULL` with no default, so a path that copies or
writes a target must say which. A target the build holds, and the `not_built` verdict on one it lacks, is
`local`; every other row a crawl writes is `live`, `unchecked` included: the run decided it, as a copy does not.
The verdict views ignore it, so the gate compares like with like (D28); `report::inverse` alone reads it,
`local` meaning built (D56). Rejected: a default of `live`, which
labels a forgotten copy as a request; a table of copies, giving `link_verdicts` a second path. Reopens: a report
wanting to split a run's answers by route. Code: `store::schema`, `store::AnsweredBy`. Evidence: `store::tests::a_target_records_where_its_answer_came_from`.

### D52 A baseline file is imported as a run of its own, and a target is copied between runs row by row
`import_baseline` copies the file's one run in under the next number, or finds the run whose `started_at`
and `config_hash` match (the file keeps both): the gate is then a two-run query in one database, as `diff`
is. `copy_target` copies a target, its landing target, and each one's `pages` row and `anchors`, as
`baseline`; a target the run already holds is untouched, `page_facts` stay behind so a copied page is
never one the build holds (D22). Rejected: attaching the file, the gate reading across two databases and
`baseline_run` naming a run in a file gone after the job. Reopens: a reuse that misses on a hand-edited
file. Code: `store::baseline`. Evidence: `store::baseline::tests::a_baseline_already_imported_is_reused_and_copies_nothing`.

### D53 In a local build, an href beginning `/` resolves against the prefix
The build directory's root is the site's root, what a static server serving it at `/` shows, so
`extract` joins such an href (not `//`) onto `--prefix`, a `<base href>` that is root-relative too.
Why: a partial build has no `site.url` (I11), so the UI's footer writes `/legal/privacy/` with an empty
`site.path`, which against the production origin is a 404 on every built page, a break the tool made.
Rejected: the production origin, with the guide telling deployers to `skip` those URLs. Reopens: a
build whose root-relative hrefs are meant for the origin, not the prefix. Code: `extract::links::Links::resolve`.
Evidence: `extract::tests::in_a_local_build_an_href_beginning_a_slash_resolves_against_the_site_root_d53`.

### D54 A URL the build lacks waits for the held set, and a copy from the baseline waits for the end
An in-scope URL under the prefix and not built is judged after every built page is read: in a component-version one carried
(I16) it is `broken_url`, reason `not_built`; otherwise it goes where a URL outside the build goes: the baseline's rows if
held, else a request one hop deep. The copies are sent last, `targets` having no key to stop a copied landing page doubling
the build's own. `--externals=skip` means never ask: an allowlisted URL gets the baseline's rows where held (a new link to
one broken there is introduced), else `unchecked`, reason `externals_skipped`, as does a hop onto one (D50's way); in-scope
URLs are not reached. Rejected: a pre-scan of the build (D38); `skip` as no verdict, passing that link. Reopens: a prefix
below the site root. Code: `crawl::baseline`. Evidence: `crawl::baseline::tests`.

### D55 An xref waits for the held set, resolves through the baseline's pages, and keeps its refspec
A built page's tagged link is written once every built page is read (D54's wait). A refspec whose component-version the build holds stays `unresolved_xref`; else the
profile's Resolve maps it through the baseline's pages, loaded before the run as its targets are (D52), and the link points at that page with the refspec's fragment,
`xref_refspec` kept, so `link_verdicts` reads a refspec as unresolved where the link is to its own page or its fragment is the refspec (I6). No version: the one whose pages
sit off their own segment (I5), named under it where no page is found (D57); none or several, Antora's order (I20). Rejected: a column or cleared refspec marking a resolved
link (schema 5 refusing every published baseline, D43); a config key of prerelease versions; `rawhide` by name (D32). Reopens: Antora marking an unresolved xref other than
as `#<refspec>` (I6). Code: `crawl::xref`, `profile::antora::resolve`. Evidence:
`crawl::baseline::tests::an_xref_into_a_component_the_build_lacks_resolves_through_the_baseline`, `profile::antora::resolve::tests`.

### D56 The inverse check reads the held component-versions back from the CI run, and counts the links of pages outside them
A built page's `antora.*` facts and URL give the held prefixes (I16), so `report inverse --run` works from the DB alone. The
linking pages are the baseline's crawled pages in no held prefix and not built. A removed file is a baseline file in a held
prefix with no `local` `ok` target, every file of the build being a seed (`ci`). Rejected: every page not built, a deleted page
failing it on links gone with it, while those it misses are KD5's; `Held` moved under `crawl` and `report`, a `crawl`-to-`ci`
edge against D39; a held set stored in the run, schema 5 refusing the baseline (D43); touched pages by `source_repo`, CI's pages
carrying no edit link (I15); seeding the baseline's targets under the held prefixes, known only once the built pages are read.
Reopens: I16 changing, its layout read in three places (D57). Code: `report::inverse`. Evidence: `report::inverse::tests`.

### D57 An xref no page's facts find is the URL its refspec names, answered as any URL the build lacks
Antora resolves a refspec through a page-alias, and to either of two files sharing an output URL; the baseline holds facts once
per URL, and an alias only if crawled. So where Resolve's facts find no page it returns the URL I16 names, under the prefix read
off a baseline page of the component-version, at the segment its pages sit at (I5), the refspec naming the version or not; D22's
chain answers it: the baseline's rows, else a request. A URL answering `broken_url` makes the link `unresolved_xref`
(`link_verdicts` rule 1); `error` and `unchecked` stay. Rejected: the baseline's redirect targets alone (an uncrawled alias is missed);
live alone; any outcome but `ok` as unresolved, failing a PR on a timeout or refusal its author cannot fix (D16) and judging a derived
URL more harshly than a page the facts found (D55). Reopens: I16 changing (the layout in three places). Code: `profile::antora::resolve`. Evidence: `crawl::baseline::tests::an_xref_no_page_has_the_coordinates_of_resolves_by_the_url_it_names`.

### D27 One binary, `bruce`, with subcommands `crawl`, `check-dir`, `report`, `export-baseline`, `prune`
`check-dir` writes its results to a local DB too, so reports work on CI runs, and its exit status
follows D9 and D16. It reads the config `crawl` reads, `--prefix` naming the build's production URL,
so `skip`, the allowlist and the scopes plan a URL as they did for the baseline. `--inverse` is a flag,
not a key: the config is the deployment's, shared by every repo, and one repo's strictness is its own.
Rejected: a CI-only config section or file, or flags alone, a difference from the baseline's config
reading as breakage. The flags are the manual's (`docs/manual/`). Code: `cli`.

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
Evidence: `historical/initial.md`, "Existing tools"; `cargo bench` in `spikes/tokenizer/` at b436db8, `runs/tokenizer-spike/`.

### D23 DuckDB is bundled; distribution is a release binary on GitHub
The `duckdb` crate's `bundled` feature makes a self-contained binary that a CI job in the
Fedora-minimal docs image fetches in one step. Rust is pinned in `mise.toml`, whose cargo tasks sit
beside `check`, which runs the same commands with its own flags. A `v*` tag builds the binary in a
`debian:trixie` container, glibc 2.41 its floor (Fedora 42 and later meet it), and publishes it at
`mwinters0/bruce`'s releases, immutable once published. Rejected: `check` calling the tasks, which
would make `mise run fmt` stop writing or lose the summaries; F44's glibc, shutting out Trixie and F42; static musl, DuckDB's C++ being untried there; a release built from a maintainer's tree; the Fedora forge, whose policy admits Fedora software only; for now, a container image, RPM.
Code: `.github/workflows/release.yml`. Evidence: `scripts/workflows.py`, the workflow's glibc check.

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
timing threshold in `check`, flaky at the spike's 8% run-to-run noise (`runs/tokenizer-spike/`) and tied
to one machine.
