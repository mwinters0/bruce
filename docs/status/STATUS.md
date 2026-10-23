# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 864c1c6 --> The marker names the commit `scripts/repoint.py`
measures the record's growth from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

The `bruce` crate is D39's top-level modules, a binary whose subcommands
are `bruce crawl`, `bruce check-dir`, `bruce report`, `bruce prune` and `bruce export-baseline`, and Rust 1.99.1 pinned in `mise.toml`. Eleven modules hold
code: `cli`, the command line, the five subcommands and the profile registry; `config`, the loader; `store`, the DuckDB schema, its version check, the
`link_verdicts` view, the deleting of old runs, the baseline export and import, the index of a baseline's targets and its pages' facts, the copy of a target's rows between runs, which the writer also runs on request, and the single writer for `pages`, `page_facts`, `targets`, `anchors` and `links`; `profile`, the trait, the registry and the
`generic` and `antora` profiles' Detect, Views, Extract and Resolve; `extract`, the pass over one page, which yields its
profile, title, anchors with placement, links with kind and tag, the edit link,
the profile's facts and the content hash;
`forge`, the edit-URL shape parse;
`url`, the canonical form and `url_id`;
`fetch`, the fetcher trait, the HTTP fetcher for one hop, each host's pace, in-flight cap and robots.txt, the body decoding, the local fetcher over a map of URLs to files and the router that sends a URL the build holds to it and any other to HTTP;
`crawl`, the frontier, the hops of a redirect, D16's outcomes, the schedule that gives a build's reads a budget of their own and each live host its share of the run's other slots (D61), sitemaps, the run that writes a target's rows and its stop, and a run from seeds checked against a baseline: the component-versions the build holds, `not_built`, the baseline's rows copied for what it lacks, and the xrefs into a component the build lacks resolved to the baseline's pages or the URLs they name;
`ci`, a build directory read as its production URLs; and
`report`, the `report_*` views, the choice of run, the eight single-run reports, `diff`, and the gate (`introduced` with the inverse check, the redirected links, the errors and the breakage already in the baseline; `check-dir --inverse fail|warn`), as a text table or JSON.
`docs/manual/` holds the config reference, `bruce crawl`'s, `bruce check-dir`'s, `bruce report`'s, `bruce prune`'s, `bruce export-baseline`'s and the deployment guide's config section, which `mise run sim` extracts into `sim/` and checks. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`, and `fixtures/antora/`, a
three-component Antora site, the Fedora UI bundle and seven built pages, which
`mise run e2e` rebuilds and then crawls, exports and runs `check-dir` over, asserting
the gate's exit status and sections.
The crate is `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`), and `THIRD-PARTY-NOTICES.txt` is generated from `Cargo.lock` and the DuckDB library texts vendored under `licenses/duckdb/` (libpg_query's PostgreSQL License paragraphs by hand) by `scripts/notices.py`, whose unit tests fail when it is out of date or the texts have drifted from the DuckDB tarball (D60).
`.github/workflows/release.yml` builds a `v*` tag in `debian:trixie` and links `libstdc++` and `libgcc_s` in, so glibc is the binary's only library, and publishes the archive on GitHub (D23), `v0.1.0` being the first release; `scripts/workflows.py` fails on any workflow's `uses:` and prints a workflow's steps as a script to rehearse in a container.

## P8 progress

The spec is [`../wip/roadmap-P8-history.md`](../wip/roadmap-P8-history.md).

- [ ] **8.1** A hash records its scope: `extract::hash` says which it hashed,
  the writer records it and a run its roots' `hash_root` and `hash_exclude`,
  schema 5. Closes on an en-US crawl launched detached under `runs/`.
- [ ] **8.2** A release names its schema change: `release.yml` compares
  `SCHEMA_VERSION` with the latest published release's, rehearsed by
  `scripts/workflows.py`.
- [ ] **8.3** The digest's document and JSON: `report/digest`, its
  `format_version`, `bruce report digest --out --name --format`, the
  sections over one run, the manual.
- [ ] **8.4** The digest's HTML: the rendering, the no-script test, the golden
  file over `mise run e2e`'s site.
- [ ] **8.5** An unresolved xref's hint: the `antora` view, `broken`'s and the
  digest's `hint`, measure and floor chosen on en-US's rows.
- [ ] **8.6** The maintainer's review of a real en-US digest: no code, the
  verdict and what it spawns in the notes.
- [ ] **8.7** `changed` and `anchors-removed`: `report` children and kinds,
  the scope changed and the root missed flagged.
- [ ] **8.8** The digest's comparison: `--from`, `--since` with `cli::file`,
  section 3's causes, section 7, the figures' changes; over a second en-US
  crawl against 8.1's.

## Not started

Every phase in [`../design/roadmap.md`](../design/roadmap.md)'s index not
`Complete`, `Struck` or `Current`: P9 is specified and waits for P8; P7 has
not been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".
