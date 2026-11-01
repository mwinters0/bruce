# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 88be3bd --> The marker names the commit `scripts/repoint.py`
measures the record's growth from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

The `bruce` crate is D39's top-level modules, a binary whose subcommands
are `bruce crawl`, `bruce check-dir`, `bruce report`, `bruce prune` and `bruce export-baseline`, and Rust 1.99.1 pinned in `mise.toml`. Eleven modules hold
code: `cli`, the command line, the five subcommands and the profile registry; `config`, the loader, the policy a run is judged by and asks with, and the declarative profiles a file defines; `store`, the DuckDB schema, its version check, the
`link_verdicts` view, the guard naming the run a comparison judges by, `link_hints`, the policy a run records and a trial one in its place, the deleting of old runs, the baseline export and import, the index of a baseline's targets and its pages' facts, the copy of a target's rows between runs, which the writer also runs on request, and the single writer for `pages`, `page_facts`, `targets`, `anchors` and `links`; `profile`, the trait, the registry, the
`generic` and `antora` profiles' Detect, Views, Extract and Resolve and the declarative profile a config's `[[profile]]` builds (region, generator pattern, fact rules, source rules, edit link, the links of a language menu, and the compiled profile it extends); `extract`, the pass over one page, which yields its
profile, title, anchors with placement, links with kind and tag, the edit link,
the profile's facts, the other locales its profile's language-menu links name and the content hash with the scope it covered;
`forge`, the edit-URL shape parse and the source rules a profile states, tried before it (D93);
`url`, the canonical form, `url_id` and a final `/` toggled;
`fetch`, the fetcher trait, the HTTP fetcher for one hop, each host's pace, in-flight cap and robots.txt, the body decoding, the local fetcher over a map of URLs to files and the router that sends a URL the build holds to it and any other to HTTP;
`crawl`, the frontier with the other locales learned and the URLs on a root's host outside every scope waiting on them, the hops of a redirect, D16's outcomes, the ask of a broken 4xx's URL, or of the broken 4xx a redirect lands on, with its final `/` toggled (D72), the schedule that gives a build's reads a budget of their own and each live host its share of the run's other slots (D61), sitemaps, the run that writes a target's rows and its stop, and a run from seeds checked against a baseline: the component-versions the build holds, `not_built`, the baseline's rows copied for what it lacks, and the xrefs into a component the build lacks resolved to the baseline's pages or the URLs they name;
`ci`, a build directory read as its production URLs; and
`report`, the `report_*` views, the choice of run, the eight single-run reports, `diff`, `changed` and `anchors-removed` over two runs, and the gate (`introduced`, in the outcomes the CI run's `gate.fail` names, the introduced breakage the list leaves out and the inverse check following the list, each in a section of its own (D83), the redirected links, the errors and the breakage already in the baseline; `check-dir --inverse fail|warn`), as a text table or JSON, and the digest of a crawl run (D64), written as dated JSON and HTML by `bruce report digest`, compared with the run before it, a named run or a baseline file (D73), the breakage new since it attributed to what it held (D74), its dots, legend, counts and ranking following the site's severity map, a kind at none out of its lists (D82, D92) and its Housekeeping listing the settings off their defaults and, compared, those that differ between its runs (D91), and its pages grouped by the fact `[digest]` `group_by` names (D86); `--config` on every report and the digest judges by a file's judging settings in place of the runs' recorded ones, the trial written nowhere (D84).
`docs/manual/` holds the config reference, `bruce crawl`'s, `bruce check-dir`'s, `bruce report`'s, `bruce prune`'s, `bruce export-baseline`'s and the deployment guide's config section, which `mise run sim` extracts into `sim/` and checks. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages, 13 of docs.antora.org's, 9 of the
Sphinx project's documentation and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`, and `fixtures/antora/`, a
three-component Antora site, the Fedora UI bundle and seven built pages, which
`mise run e2e` rebuilds and then crawls, exports and runs `check-dir` over, asserting
the gate's exit status and sections.
`mise run shot` renders a page of HTML in Chrome's headless shell, in a container, to PNGs and a printed PDF, for review.
The crate is `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`), and `THIRD-PARTY-NOTICES.txt` is generated from `Cargo.lock` and the DuckDB library texts vendored under `licenses/duckdb/` (libpg_query's PostgreSQL License paragraphs by hand) by `scripts/notices.py`, whose unit tests fail when it is out of date or the texts have drifted from the DuckDB tarball (D60).
`.github/workflows/release.yml` builds a `v*` tag in `debian:trixie` and links `libstdc++` and `libgcc_s` in, so glibc is the binary's only library, and publishes the archive on GitHub (D23), its body the annotated tag's release notes, drafted by `/release` from its template, with a schema change from the latest release's named in them (D43; a lightweight tag is refused), `v0.1.0` being the first release; `scripts/workflows.py` fails on any workflow's `uses:` and prints a workflow's steps as a script to rehearse in a container.

## P10 progress

The spec is [`../wip/roadmap-P10-extraction.md`](../wip/roadmap-P10-extraction.md).

- [x] **10.1** Evidence: docs.antora.org's and a Sphinx site's pages saved as
  fixtures, and the "before" en-US crawl launched detached from a copy of
  today's binary: finished, exit 0, in
  `runs/p10-before-20261031-1647/en-us.duckdb`
  ([notes](../wip/roadmap-P10.1-evidence-notes.md)).
- [x] **10.2** Selectors compiled once per run, with no change of behaviour
  ([notes](../wip/roadmap-P10.2-selectors-notes.md)).
- [x] **10.3** A standalone declarative profile: `[[profile]]`'s
  `region`, `generator`, fact rules and `edit_link`, the registry, the
  `policy` rows, checked on the Sphinx fixture
  ([notes](../wip/roadmap-P10.3-declarative-notes.md)).
- [x] **10.3.1** Declared profiles as a `[[profile]]` array named by `name`,
  tried in its order, a name given twice refused, and toml's `preserve_order`
  dropped; D33's syntax, the manual's Profiles section and the loader's tests
  follow ([notes](../wip/roadmap-P10.3.1-profile-array-notes.md)).
- [x] **10.4** Source rules before `forge::parse`, and struck `KD1` and `KD2`
  ([notes](../wip/roadmap-P10.4-source-rules-notes.md)).
- [x] **10.5** `extends`: an extension over a compiled profile, its facts
  `<name>.*` beside the base's, and `pages.profile` its name
  ([notes](../wip/roadmap-P10.5-extends-notes.md)).
- [x] **10.5.1** A report giving an extension's pages its base's sitemap note
  through the run's recorded `extends`, and a comparison digest listing the
  `policy` rows that differ between its runs, the profile rules aligned
  ([notes](../wip/roadmap-P10.5.1-report-notes.md)).
- [x] **10.6** Locale learning in the core, `locale_links`, and the guide's
  `fedora` extension extracting what today's code does
  ([notes](../wip/roadmap-P10.6-locale-links-notes.md)).
- [x] **10.7** Neutral help examples and palette
  ([notes](../wip/roadmap-P10.7-neutral-text-notes.md)).
- [x] **10.8** The "after" crawl compared with 10.1's: no fact, locale or
  verdict differs that bruce caused; the three link verdicts that moved follow
  their targets or a page's own edit
  ([notes](../wip/roadmap-P10.8-closing-evidence-notes.md)).
- [x] **10.9** Antora's implicit ROOT module: a `pages/` directly under a
  start path read as `ROOT`'s, and KD34 struck
  ([notes](../wip/roadmap-P10.9-implicit-root-notes.md)).

## Not started

Every phase in [`../design/roadmap.md`](../design/roadmap.md)'s index not
`Current`, `Complete` or `Struck`: P11 has not been grilled; P9, which
follows P10 and P11, is specified, to be grilled again before it is sliced;
P12 and P13 have not been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".
