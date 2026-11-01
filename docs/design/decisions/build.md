# Build, layering and release

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. How the code is organised and shipped: one crate and its
module layering, the stack it is built on, the tests, the licence and notices, and the release workflow.
`scripts/module_graph.py` checks the layering, `scripts/notices.py` the notices and `.github/workflows/release.yml`
builds and publishes a release.

### D31 One crate, `bruce`; its concepts are separated by top-level module, not by crate
The binary is the only consumer beside the crate's own tests, benches and `examples/sim_config.rs`, so a crate boundary
buys nothing a module boundary does not. Segregation is enforced all the same: each concept in one place
(`concepts.md`), module edges one-way (`module_graph.py`), files under `file_limits.py`'s caps; the layering is D39.
Rejected: a workspace of core, I/O and CLI crates. Reopens: a second consumer of the core, or the evidence the `seams`
skill's §4 names.

### D39 The layering: `cli` › `crawl`, `ci`, `report` › `fetch`, `extract`, `store` › `profile` › `config` › `forge`, `url`
Edges point down only. `config` reads `url`, its scope matching the canonical form (D44), and `forge`, building a source
rule to refuse it at load (D93). `store` implements `profile`'s `Lookup` and asks each profile for its views, so a
profile never reaches the DB. Only `cli` names a concrete profile, to build the registry (`module_graph.py`'s
`CONFINED`). Rejected: `profile` above `extract`, driving the pass, which makes the core a library of the profile, the
inverse of D32; scope moved to `crawl` to keep `config` lowest, splitting it and losing D44's refusal at load; a tier
check in `module_graph.py`, its cycle check catching nearly every upward edge. Reopens: a skip-tier edge passing review.

### D11 Our own stack, not lychee-lib; lol_html is the tokenizer
reqwest, texting_robots, lol_html and duckdb, a host's pace its own next-allowed instant (`fetch::http::host`). What is
ours (crawl frontier, link graph, anchor semantics) lychee does not model, and its extractor would need working around
(D12's rejection). lol_html and html5gum tied on the spike's workload; lol_html matches CSS selectors while streaming,
which profiles and D33's rules want. Rejected: html5gum, leaving us an open-element stack and a selector matcher to
write; governor, with no per-key quota (boinkor-net/governor#193). Reopens: lol_html sharing a rewriter's matching
program between pages, the cost left once `profile::selectors` parsed each selector once a run, within the `extract`
bench's noise. Evidence: `historical/initial.md`, "Existing tools"; the spike's
`cargo bench` at b436db8, `runs/tokenizer-spike/`.

### D24 Tests run on saved real pages, a mock HTTP server, and an optional Antora build
Fixture pages from docs.fp.o (I6, I8, I9), docs.antora.org and a Sphinx site (D32), hand-written pages for other
generators' markup (I12), wiremock for redirects, 429 and robots, and a criterion parser bench that `check` runs once
in test mode and `mise run bench` times. An end-to-end three-component Antora site sits behind `mise run e2e`, being
slow. The mock servers speak `http` alone, so two origins differ by port, and no test resolves a name to NXDOMAIN
through the crawl. Rejected: a timing threshold in `check`, flaky at the spike's 8% run-to-run noise
(`runs/tokenizer-spike/`) and tied to one machine. Code: `benches/extract.rs`, `scripts/e2e.py`. Evidence:
`tests/extract.rs`, `fixtures/`.

### D23 DuckDB is bundled; distribution is a release binary on GitHub
The `duckdb` crate's `bundled` feature makes one binary a CI job in the Fedora-minimal docs image fetches in one step. A
`v*` tag builds it in `debian:trixie`, glibc 2.41 its floor, with `libstdc++` and `libgcc_s` linked in so glibc is its
only library, and publishes it on `mwinters0/bruce`. Rust is pinned in `mise.toml`; `check` runs cargo itself, not the
tasks, which would stop `mise run fmt` writing or lose the summaries. Rejected: F44's glibc, excluding Trixie and F42;
static musl, DuckDB's C++ untried there; a build from a maintainer's tree; the Fedora forge, admitting Fedora software
only; for now, a container image, RPM or aarch64; `-static-libstdc++`, a no-op as rustc names `-lstdc++` and `-lgcc_s`
itself. Code: `.github/workflows/release.yml`. Evidence: `scripts/workflows.py`; its glibc and `ldd` checks;
`runs/release-static/`.

### D60 `bruce` is `MIT OR Apache-2.0`, and a release carries every notice its binary owes, made offline
`LICENSE-MIT` names the holder once, its year first publication, never bumped. `THIRD-PARTY-NOTICES.txt` is committed,
generated from `Cargo.lock` and the license files of every `third_party/` directory DuckDB's tarball holds, compiled in
or not, vendored at the tarball's source id so generating and checking stay offline. mbedtls is taken under
Apache-2.0; an MPL-2.0 crate names its source (§3.2(a)); libpg_query's PostgreSQL License, absent from its source, is
copied from postgresql.org. Rejected: `cargo about`; exactly the libraries linked, needing an include scan; fetching at
generation; a pointer to DuckDB's repository; the Bison skeleton's FSF notice, its exception asking none; `The bruce
contributors`; a year range. Reopens: a DuckDB tag off the tarball's source id. Code:
`scripts/notices.py`. Evidence: `test_notices.ThisRepo`.

### D62 A release is tried as `-rc.N` prereleases, since a published release is immutable
A tag with a `-` suffix is a GitHub prerelease, so Latest never names one; the workflow is iterated on
`v<x>.<y>.<z>-rc.<N>` tags until right, and the bare tag follows. Releases are immutable, so a fault in a published
one's assets is fixed in the next patch, and a tag moves only while nothing has published from it. Every tag push is
the maintainer's. Rejected: full releases from the start, each fix an immutable `v0.1.x`. Code:
`.github/workflows/release.yml`. Evidence: every tag from `v0.1.0-rc.1` to `v0.2.0`, run on `debian:trixie`,
`fedora:44` and `fedora-minimal:42` by `runs/release-v0.1.0/recipe.sh`, logs in `runs/release-{rc2,v0.1.0,v0.2.0}/`.

### D63 The release workflow publishes the draft its create call returned, by id
`gh api -X POST` creates the draft and keeps its id; both assets are uploaded to it by `curl`, it is read back to hold
both, and it is published by that id; one listing deletes a failed run's drafts, logging each. Why: a draft is found
neither by tag (`GET /releases/tags/<tag>` omits drafts) nor reliably by listing: `v0.1.0`'s Publish failed twice
inside GitHub incidents, its listing missing a 15-minute-old draft. Rejected: `gh release create`, which prints a URL
and not an id; `gh release upload`, which finds its draft by tag; finding the release by listing. Code: the Publish
step of `.github/workflows/release.yml`. Evidence: runs 39820173534 (`v0.1.0-rc.3`) and 39822121858 (`v0.1.0`) on
`mwinters0/bruce`, Publish logging `draft <id>` and what it holds.

### D89 A release's notes are its annotated tag's message, drafted by `/release` from a template
The maintainer edits the draft and tags with it; the workflow refuses a lightweight tag or an empty annotation before
building, and makes the message the body, D43's schema item placed under its ⚠️ CAUTION. Why: a published release is
immutable (D62), so its text must exist before Publish runs, and the tag is the one input the maintainer's push already
carries. Rejected: GitHub's generated notes, commit subjects in no user's terms; a notes file committed per release, a
commit whose one reader is the workflow; a hand edit after Publish, which an unattended Publish forgets. Code:
`release.yml`'s "Check the tag against the crate" and "Write the release body", `.claude/skills/release/`. Evidence:
`test_workflows.TagStep`, `test_workflows.BodyStep`.
