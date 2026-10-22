# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 6020ee8 --> The marker names the commit `scripts/repoint.py`
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
`crawl`, the frontier, the hops of a redirect, D16's outcomes, sitemaps, the run that writes a target's rows and its stop, and a run from seeds checked against a baseline: the component-versions the build holds, `not_built`, the baseline's rows copied for what it lacks, and the xrefs into a component the build lacks resolved to the baseline's pages or the URLs they name;
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
The crate is `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`), and `THIRD-PARTY-NOTICES.txt` is generated from `Cargo.lock` and the DuckDB library texts vendored under `licenses/duckdb/` (libpg_query's PostgreSQL License paragraphs by hand) by `scripts/notices.py`, whose unit tests fail when it is out of date or the texts have drifted from the DuckDB tarball.
`.github/workflows/release.yml` builds a `v*` tag in `debian:trixie` and links `libstdc++` and `libgcc_s` in, so glibc is the binary's only library, and publishes the archive on GitHub (D23), `v0.1.0` being the first release; `scripts/workflows.py` fails on any workflow's `uses:` and prints a workflow's steps as a script to rehearse in a container.

## P6 progress

The spec is [`../wip/roadmap-P6-release.md`](../wip/roadmap-P6-release.md).

- [x] **6.1** License and notices: `MIT OR Apache-2.0`, both texts and
  `Cargo.toml`'s `license`, and the third-party notices file generated from
  `Cargo.lock`. Notes: [`../wip/roadmap-P6.1-license-notes.md`](../wip/roadmap-P6.1-license-notes.md).
- [x] **6.1.1** DuckDB's bundled libraries' license texts in the notices, every
  `third_party/` directory the tarball holds, mbedtls under Apache-2.0, and
  each MPL-2.0 crate's exact source. Notes:
  [`../wip/roadmap-P6.1.1-bundled-licenses-notes.md`](../wip/roadmap-P6.1.1-bundled-licenses-notes.md).
- [x] **6.1.2** libpg_query's PostgreSQL License paragraphs, hand-vendored from
  postgresql.org beside the two copyright lines of its `SOURCE-NOTICE`, marked
  as hand-vendored and printed in the notices; the drift check does not hold
  them verbatim. Notes:
  [`../wip/roadmap-P6.1.2-postgresql-license-notes.md`](../wip/roadmap-P6.1.2-postgresql-license-notes.md).
- [x] **6.2** The release workflow: a `debian:trixie` build on a `v*` tag, the
  archive of binary, notices and licenses with its sha256, a draft made, filled
  and published by the id its create returns; the no-third-party-actions standing rule and its check;
  D23 and D43 amended. `v0.1.0-rc.1` built, published, downloaded and run on
  Trixie and F44. Notes: [`../wip/roadmap-P6.2-release-workflow-notes.md`](../wip/roadmap-P6.2-release-workflow-notes.md).
- [x] **6.3** `v0.1.0`, and the README's quickstart: where the binary is, its
  glibc floor, and that a 0.x release migrates no DB. `v0.1.0` is published,
  and the quickstart's recipe passes against its assets on Trixie and F44.
  Notes: [`../wip/roadmap-P6.3-v0.1.0-notes.md`](../wip/roadmap-P6.3-v0.1.0-notes.md).

## Not started

Every other phase in [`../design/roadmap.md`](../design/roadmap.md)'s index.
P9 is specified and waits for P8; P7 and P8 have not been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".
