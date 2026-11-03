# Modes, commands and deployment

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. What `bruce` is and how it is run: the two fetching
modes, the one binary and its subcommands, the run that needs no config, and the line between this repo, which builds
the tool, and a deployment, which configures and runs it. `cli` holds the subcommands; the deployment guide and
`scripts/sim.py` hold the simulated deployment.

### D1 Two fetchers behind one trait: live HTTP crawl first, local build directory second
Parsing and the DB writer are shared. The local-dir mode exists for CI to verify a docs change before it deploys, a full
site build taking hours, so nobody runs one locally. Rejected: local-only, which cannot see the deployed site or
external hosts; a CI pipeline of its own walking the build and querying the baseline, which rebuilds the frontier, the
hops (a local alias is one), D16 and the writer. Reopens: a published full-site build artifact. Code: `fetch::Fetcher`,
`fetch::routed::Routed`, `cli::check_dir`.

### D27 One binary, `bruce`, with subcommands `crawl`, `check-dir`, `report`, `export-baseline`, `prune`
`check-dir` writes its results to a local DB too, so reports work on CI runs; its exit status follows D9 and D16. It
reads the config `crawl` reads, `--prefix` naming the build's production URL, so `skip`, the allowlist and the scopes
plan a URL as they did for the baseline. `--inverse` is a flag, not a key: the config is the deployment's, shared by
every repo, and one repo's strictness is its own. Rejected: a CI-only config section or file, or flags alone, a
difference from the baseline's config reading as breakage. Code: `cli`.

### D30 The name is `bruce`, after the Bruce claw anchor
It was free on crates.io on 2026-10-17. `anchorage` (D25) was taken there.

### D40 A run needs no config: the URL given is the root, and every other setting has a default
Without a file, the URL is the only root, scoped to its path prefix, its sitemap found as `config::Root::sitemap` says;
profile `auto` (D35), robots.txt and its `Crawl-delay` obeyed (D18), `DEFAULT_PER_SECOND` per host otherwise, no
allowlist. A config overrides each. Rejected for now: a required config, and `bruce init`, until someone asks for one.
Code: `config`.

### D41 This repo builds the tool and never deploys it
A deployment (docs.fp.o's config, its nightly crawl, where its baseline is published) lives with its deployer. The
manual documents how to configure one, and a gitignored simulated deployment here verifies the tool supports it.
Rejected: a shipped `fedora.toml`, or a preset compiled in, either putting a deployment into the tool's release.

### D42 The simulated deployment is the manual's deployment guide, executed
`mise run sim` extracts the guide's tagged blocks into a gitignored `sim/` and checks its configs load, its scripts parse
and its units are wired, so a guide that drifts from the tool fails there and any machine rebuilds it; docs.fp.o's config
exists here only as the guide's worked example. `cli::tests` holds the scripts' `bruce` lines to the command line and
`mise run e2e` runs the scripts over the fixture site. Rejected: a committed template directory, which drifts from the
guide describing it; a hand-built `sim/`, which no other machine can rebuild; running the crawl there, a two-hour job at
every sim; parsing the CI workflow as YAML there, which takes PyYAML, whose YAML 1.1 reads `on:` as true, for a parse
`e2e`'s js-yaml makes at every guide change. Code: `scripts/sim.py`, `scripts/deployment.py`. Evidence:
`scripts/test_sim.py`.

### D103 A deployment publishes one directory, fetched through its `SHA256SUMS`, the files it lists named by content
The job stages `SHA256SUMS` (written last) over the binary, config and baseline, named by their first 16 hex digits
and kept a night more, and the digests, the latest under fixed names too; the three's are for people. A fetcher
fetches `SHA256SUMS`, what it lists, then `sha256sum -c`, failing as could-not-run (I28). Why: a cache gives a
stale `SHA256SUMS` beside fresh files; CI must never join two nights' binary and baseline, and a puller can retry.
Rejected: retrying in CI, which lacks ten minutes; a `manifest.json`, needing `jq`; a version file; a rolling
release, swapped per asset; for a puller, never reusing a listed name (the latest digest, the dated ones), two
scripts' logic to save a timer's wait. Reopens: a host with no cache. Code: the guide's `bruce-nightly.sh`,
`bruce-pull.sh`. Evidence: `scripts/test_deployment.py`, `mise run e2e`.

### D104 The scheduled job is one script that answers a schema change itself and publishes nothing from a failed crawl
Exit 3 (D102) from the crawl moves the database aside as the one archive and crawls a fresh one. The digest compares
with the baseline the job replaces (D73's file), uncompared on 3 or none, so a host with a disk and one without run one
script; a digest that still fails does not stop the job, CI needing the baseline, and the job ends non-zero. Any other
crawl failure stops before staging, and prune runs after it. Why: nobody is there to follow a release note, and D43
stands. Rejected: a procedure keyed on the notes; the run before in the database (D73's default), a second path; the
baseline fetched from the published URL, losing the dated digests where no disk keeps them. Reopens: a second refusal
answered differently. Code: the guide's `bruce-nightly.sh`. Evidence: `scripts/test_deployment.py`.
