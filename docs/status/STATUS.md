# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: bcfe26f --> The marker names the commit `scripts/repoint.py`
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
`report`, the `report_*` views, the choice of run, the eight single-run reports, `diff`, `changed` and `anchors-removed` over two runs, and the gate (`introduced`, in the outcomes the CI run's `gate.fail` names, the introduced breakage the list leaves out and the inverse check following the list, each in a section of its own (D83), the redirected links, the errors and the breakage already in the baseline; `check-dir --inverse fail|warn`), as a text table or JSON, and the digest of a crawl run (D64), written as dated JSON and through the views `--view` names by `bruce report digest`, the default view an HTML minijinja template embedded in the binary over the model (D95) and a deployment's own a file extending it or replacing it, in its directory and declaring its model version, a `.csv` one quoting each value (D96), `--write-default-view` exporting the default's files, compared with the run before it, a named run or a baseline file (D73), the breakage new since it attributed to what it held (D74), its dots, legend, counts and ranking following the site's severity map, a kind at none out of its lists (D82, D92) and its Housekeeping listing the settings off their defaults and, compared, those that differ between its runs (D91), and its pages grouped by the fact `[digest]` `group_by` names (D86); `--config` on every report and the digest judges by a file's judging settings in place of the runs' recorded ones, the trial written nowhere (D84).
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

## P9 progress

The deployment guide: [`../wip/roadmap-P9-deployment.md`](../wip/roadmap-P9-deployment.md).

- [ ] **9.1** Evidence: a fresh en-US crawl under schema 7, detached; whether a DB pruned nightly holds a steady size under `prune --keep 30`; whether Anubis challenges a GitHub runner, from a throwaway workflow in the deployment repo
- [ ] **9.2** The site-root refresh: `ci` serving a top `index.html` meta refresh at its own path, D22 amended, the concept map's clause, `e2e.py`'s `UNCHANGED_REDIRECTED` reversed
- [ ] **9.3** Exit status 3: the typed schema refusal in `store::schema`, mapped in `cli` for `crawl`, `check-dir`, `report`, `prune` and `export-baseline`, the manuals' exit tables
- [ ] **9.4** The guide's scheduling, publishing and delivery: the script, the systemd units, `bruce-locales.toml`, `per_second = 3`, the staged directory with `SHA256SUMS` and names by content, retention and the schema archive, the digest's comparison, the pull script, the workflows' flowchart; `sim` checking the scripts, `e2e` running the pull
- [ ] **9.5** The guide's CI section: the workflow with its fetch and check, `sim`'s YAML and `uses:` checks, `e2e` running its steps, the Antora pin checked against production's
- [ ] **9.6** The dev deployment: `v0.3.0-rc.1` (the maintainer's tag), the deployment repo's nightly workflow and orphan Pages branch, the crawl's time on Actions, the guide corrected where they show it wrong
- [ ] **9.7** The closing evidence: a quick-docs mirror (the maintainer's repository) running the guide's workflow against the Pages URL, a planted break failing and a clean change passing
- [ ] **9.8** The maintainer's review of the en-US reports, the digest included: no code
- [ ] **9.9** The every-locale crawl, weekly on the maintainer's machine: history, `prune` and the schema archive exercised, the guide's locale figures, the digest's size deciding whether its results are published
- [ ] **9.10** The release P9 closes on (the maintainer's tag), and the README's CI section corrected

## Not started

Every phase in [`../design/roadmap.md`](../design/roadmap.md)'s index not
`Current`, `Complete` or `Struck`: P15, P16, P12 and P13 have not been
grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".

