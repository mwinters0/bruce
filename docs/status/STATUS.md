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
the profile's facts, the other locales its language menu names and the content hash with the scope it covered;
`forge`, the edit-URL shape parse;
`url`, the canonical form and `url_id`;
`fetch`, the fetcher trait, the HTTP fetcher for one hop, each host's pace, in-flight cap and robots.txt, the body decoding, the local fetcher over a map of URLs to files and the router that sends a URL the build holds to it and any other to HTTP;
`crawl`, the frontier with the other locales learned and the URLs outside every scope waiting on them, the hops of a redirect, D16's outcomes, the schedule that gives a build's reads a budget of their own and each live host its share of the run's other slots (D61), sitemaps, the run that writes a target's rows and its stop, and a run from seeds checked against a baseline: the component-versions the build holds, `not_built`, the baseline's rows copied for what it lacks, and the xrefs into a component the build lacks resolved to the baseline's pages or the URLs they name;
`ci`, a build directory read as its production URLs; and
`report`, the `report_*` views, the choice of run, the eight single-run reports, `diff`, and the gate (`introduced` with the inverse check, the redirected links, the errors and the breakage already in the baseline; `check-dir --inverse fail|warn`), as a text table or JSON, and the digest of a crawl run (D64), written as dated JSON and HTML by `bruce report digest`.
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

## P8 progress

The spec is [`../wip/roadmap-P8-history.md`](../wip/roadmap-P8-history.md).

- [x] **8.1** A hash records its scope: `extract::hash` says which it hashed,
  the writer records it and a run its roots' `hash_root` and `hash_exclude`,
  schema 5. Closes on an en-US crawl launched detached under `runs/`.
  Notes: [`../wip/roadmap-P8.1-hash-scope-notes.md`](../wip/roadmap-P8.1-hash-scope-notes.md).
- [x] **8.2** A release names its schema change: `release.yml` compares
  `SCHEMA_VERSION` with the latest published release's, rehearsed by
  `scripts/workflows.py`.
  Notes: [`../wip/roadmap-P8.2-release-schema-notes.md`](../wip/roadmap-P8.2-release-schema-notes.md).
- [x] **8.3** The digest's document and JSON: `report/digest`, its
  `format_version`, `bruce report digest --out --name --format`, the
  sections over one run, the manual.
  Notes: [`../wip/roadmap-P8.3-digest-json-notes.md`](../wip/roadmap-P8.3-digest-json-notes.md).
- [x] **8.3.1** The broken targets ranked by their links, then their pages,
  and the housekeeping's sitemap entries no page was fetched for, reusing
  `report sitemap`'s: `report/digest`, its tests, D64 and the manual.
  Notes: [`../wip/roadmap-P8.3.1-digest-ranking-notes.md`](../wip/roadmap-P8.3.1-digest-ranking-notes.md).
- [x] **8.4** The digest's HTML: `report/digest/html`, `--format json|html|both`,
  the no-script test, the golden file `fixtures/antora/digest.html` over
  `mise run e2e`'s site, the manual.
  Notes: [`../wip/roadmap-P8.4-digest-html-notes.md`](../wip/roadmap-P8.4-digest-html-notes.md).
- [x] **8.4.1** The digest's HTML at a phone's width and in print: tables
  that scroll, a marker on every `<summary>`, print breaks, and the minor
  defects M19's trial found, spot-checked in Chrome's headless shell.
  Notes: [`../wip/roadmap-P8.4.1-digest-layout-notes.md`](../wip/roadmap-P8.4.1-digest-layout-notes.md).
- [x] **8.4.2** The digest's direction: every heading, count and column says
  whether it counts links to a target or links on a page, the figures named
  as links, the drill-down's two `anchor`s told apart; earned by 8.6's first
  finding.
  Notes: [`../wip/roadmap-P8.4.2-digest-direction-notes.md`](../wip/roadmap-P8.4.2-digest-direction-notes.md).
- [x] **8.4.3** The digest's sections folded and reordered: each after the
  figures a `<details>` starting collapsed, the pages before the repos before
  the targets; earned by 8.6's second finding.
  Notes: [`../wip/roadmap-P8.4.3-digest-sections-notes.md`](../wip/roadmap-P8.4.3-digest-sections-notes.md).
- [x] **8.4.4** The digest's unchecked links explained: each reason defined,
  the hosts or prefixes it covers most, the external hosts checked, in the
  HTML and the JSON; earned by 8.6's third finding.
  Notes: [`../wip/roadmap-P8.4.4-unchecked-legend-notes.md`](../wip/roadmap-P8.4.4-unchecked-legend-notes.md).
- [x] **8.5** An unresolved xref's hint: the `antora` view, `broken`'s and the
  digest's `hint`, measure and floor chosen on en-US's rows.
  Notes: [`../wip/roadmap-P8.5-xref-hint-notes.md`](../wip/roadmap-P8.5-xref-hint-notes.md).
- [x] **8.10** A scheme that cannot work is broken: the schemes a public web
  page uses stay `unchecked`, every other `broken_url`.
  Notes: [`../wip/roadmap-P8.10-scheme-notes.md`](../wip/roadmap-P8.10-scheme-notes.md).
- [x] **8.11** A URL on a root's host outside every scope is asked once,
  unless it is another locale's page, learned from the `antora` language menu;
  an invariant for the menu, D10 amended.
  Notes: [`../wip/roadmap-P8.11-out-of-scope-notes.md`](../wip/roadmap-P8.11-out-of-scope-notes.md).
- [x] **8.12** An allowlist entry `*.<domain>` names its subdomains, never a
  root's own host; the deployment guide's allowlist names the Fedora and Red
  Hat sites, D5 amended; `data` and `javascript` join D66's working schemes.
  Notes: [`../wip/roadmap-P8.12-allowlist-domains-notes.md`](../wip/roadmap-P8.12-allowlist-domains-notes.md).
- [x] **8.13** A host that does not resolve or never answers: its links
  `broken_url` or `error` (D16), not `unchecked`, `robots_unavailable`; KD21
  struck.
  Notes: [`../wip/roadmap-P8.13-robots-failure-notes.md`](../wip/roadmap-P8.13-robots-failure-notes.md).
- [x] **8.15** robots.txt read per origin (scheme, host and port), as RFC
  9309 scopes it, pace and concurrency staying per name (D8, D69): no link's
  verdict depends on which of its name's origins a crawl asked first.
  Notes: [`../wip/roadmap-P8.15-robots-origin-notes.md`](../wip/roadmap-P8.15-robots-origin-notes.md).
- [x] **8.14** The second en-US crawl, under the deployment guide's config
  after 8.10 to 8.13 and 8.15: launched detached (`.claude/skills/gosub/handoff.md`),
  its database the evidence 8.6's review resumes on and 8.8 compares with
  8.1's; the guide's "about two hours" corrected from its time.
  Notes: [`../wip/roadmap-P8.14-second-crawl-notes.md`](../wip/roadmap-P8.14-second-crawl-notes.md).
- [x] **8.16** A redirected link whose final URL differs only by a final `/`:
  `ok`, reason `pedantic_trailing_slash`, the housekeeping's redirects and the
  gate's introduced redirected links showing the reason and listing it last
  (D7, D16), shown on 8.14's database.
  Notes: [`../wip/roadmap-P8.16-pedantic-slash-notes.md`](../wip/roadmap-P8.16-pedantic-slash-notes.md).
- [ ] **8.19** Severity marked in the digest: a dot per link or target row
  and per section summary, broken red, attention amber, pedantic a grey ring.
- [ ] **8.6** The maintainer's review of a real en-US digest: no code, the
  verdict and what it spawns in the notes.
  In progress; findings so far in
  [`../wip/roadmap-P8.6-digest-review-notes.md`](../wip/roadmap-P8.6-digest-review-notes.md).
- [ ] **8.17** A `broken_url` link whose URL with its final `/` toggled
  answers: still broken, the digest naming that URL as its fix.
- [ ] **8.18** A redirect whose `Location` doubles a slash reported once, and
  a page reached only by a redirect from off a root not that root's page.
- [ ] **8.7** `changed` and `anchors-removed`: `report` children and kinds,
  the scope changed and the root missed flagged.
- [ ] **8.8** The digest's comparison: `--from`, `--since` with `cli::file`,
  section 3's causes, section 7, the figures' changes; over 8.14's en-US
  crawl against 8.1's.
- [ ] **8.9** Servo's renders of the digest beside Chrome's headless shell's,
  after 8.6's verdict: whether Servo alone is enough to check how the
  digest renders, the verdict in the notes.

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
