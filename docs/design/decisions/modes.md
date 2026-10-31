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
`mise run sim` extracts the guide's tagged blocks into a gitignored `sim/` and checks its config loads, so a guide that
drifts from the tool fails there and any machine rebuilds it; docs.fp.o's config exists here only as the guide's worked
example. Rejected: a committed template directory, which drifts from the guide describing it; a hand-built `sim/`, which
no other machine can rebuild; running the crawl there, a two-hour job at every sim. Code: `scripts/sim.py`. Evidence:
`scripts/test_sim.py`.
