# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 5158320 --> The marker names the commit `scripts/repoint.py`
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
the profile's facts, the other locales its language menu names and the content hash with the scope it covered;
`forge`, the edit-URL shape parse;
`url`, the canonical form, `url_id` and a final `/` toggled;
`fetch`, the fetcher trait, the HTTP fetcher for one hop, each host's pace, in-flight cap and robots.txt, the body decoding, the local fetcher over a map of URLs to files and the router that sends a URL the build holds to it and any other to HTTP;
`crawl`, the frontier with the other locales learned and the URLs on a root's host outside every scope waiting on them, the hops of a redirect, D16's outcomes, the ask of a broken 4xx's URL, or of the broken 4xx a redirect lands on, with its final `/` toggled (D72), the schedule that gives a build's reads a budget of their own and each live host its share of the run's other slots (D61), sitemaps, the run that writes a target's rows and its stop, and a run from seeds checked against a baseline: the component-versions the build holds, `not_built`, the baseline's rows copied for what it lacks, and the xrefs into a component the build lacks resolved to the baseline's pages or the URLs they name;
`ci`, a build directory read as its production URLs; and
`report`, the `report_*` views, the choice of run, the eight single-run reports, `diff`, `changed` and `anchors-removed` over two runs, and the gate (`introduced` with the inverse check, the redirected links, the errors and the breakage already in the baseline; `check-dir --inverse fail|warn`), as a text table or JSON, and the digest of a crawl run (D64), written as dated JSON and HTML by `bruce report digest`, compared with the run before it, a named run or a baseline file (D73), the breakage new since it attributed to what it held (D74).
`docs/manual/` holds the config reference, `bruce crawl`'s, `bruce check-dir`'s, `bruce report`'s, `bruce prune`'s, `bruce export-baseline`'s and the deployment guide's config section, which `mise run sim` extracts into `sim/` and checks. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`, and `fixtures/antora/`, a
three-component Antora site, the Fedora UI bundle and seven built pages, which
`mise run e2e` rebuilds and then crawls, exports and runs `check-dir` over, asserting
the gate's exit status and sections.
`mise run shot` renders a page of HTML headless with Servo to PNGs, for review.
The crate is `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`), and `THIRD-PARTY-NOTICES.txt` is generated from `Cargo.lock` and the DuckDB library texts vendored under `licenses/duckdb/` (libpg_query's PostgreSQL License paragraphs by hand) by `scripts/notices.py`, whose unit tests fail when it is out of date or the texts have drifted from the DuckDB tarball (D60).
`.github/workflows/release.yml` builds a `v*` tag in `debian:trixie` and links `libstdc++` and `libgcc_s` in, so glibc is the binary's only library, and publishes the archive on GitHub (D23), its body naming a schema change from the latest release's (D43), `v0.1.0` being the first release; `scripts/workflows.py` fails on any workflow's `uses:` and prints a workflow's steps as a script to rehearse in a container.

## Not started

No phase is current. Every phase in
[`../design/roadmap.md`](../design/roadmap.md)'s index not `Complete` or
`Struck`: P9 is specified, to be grilled again before it is sliced; P7 has
not been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".

- **`runs/headless-browser-trial/` (292 MB, Chrome's headless shell and the
  libraries it loads) is kept, not deleted once Servo's renders had been
  compared with Chrome's.** Servo cannot check a phone's tables or print
  (`scripts/shot.py`), and the trial is the only Chrome on this machine, its
  three system libraries absent otherwise. Reconsidering: delete it and accept
  Servo's blind spots, or pin Chrome in `mise.toml` (the trial's `http:`
  backend, a checksum by hand) so `mise run shot` can render with it. The
  trial's `README.md` says how to render and print with it.
