# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 44c5db8 --> The marker names the commit `scripts/repoint.py`
measures the record's growth from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

The `bruce` crate is D39's top-level modules, a binary whose subcommands
are `bruce crawl`, `bruce report` and `bruce prune`, and Rust 1.99.1 pinned in `mise.toml`. Ten modules hold
code: `cli`, the command line, the three subcommands and the profile registry; `config`, the loader; `store`, the DuckDB schema, its version check, the
`link_verdicts` view, the deleting of old runs and the single writer for `pages`, `page_facts`, `targets`, `anchors` and `links`; `profile`, the trait, the registry and the
`generic` and `antora` profiles' Detect, Views and Extract, with Resolve
declared and empty; `extract`, the pass over one page, which yields its
profile, title, anchors with placement, links with kind and tag, the edit link,
the profile's facts and the content hash;
`forge`, the edit-URL shape parse;
`url`, the canonical form and `url_id`;
`fetch`, the fetcher trait, the HTTP fetcher for one hop, each host's pace, in-flight cap and robots.txt, and the body decoding; and
`crawl`, the frontier, the hops of a redirect, D16's outcomes, sitemaps, the run that writes a target's rows and its stop; and
`report`, the `report_*` views, the choice of run, and the eight single-run reports and `diff` as a text table or JSON. The rest hold only their doc.
`docs/manual/` holds the config reference, `bruce crawl`'s, `bruce report`'s, `bruce prune`'s and the deployment guide's config section, which `mise run sim` extracts into `sim/` and checks. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`.

## P5 progress

The spec is [`../wip/roadmap-P5-ci.md`](../wip/roadmap-P5-ci.md).

- [ ] **5.1** The Antora fixture site: `fixtures/antora/`, the vendored UI
  bundle, `mise run e2e` building the site whole and in part.
- [ ] **5.2** Schema 4, the `targets` answer's provenance; the fresh en-US
  crawl launched detached.
- [ ] **5.3** `export-baseline`, its manual page and the gzip measurement.
- [ ] **5.4** The baseline's import and reuse, and the store's copy of a
  target's rows between runs.
- [ ] **5.5** The local build: `ci`'s walk and prefix mapping, `fetch/local`,
  `crawl` taking seeds; `check-dir` with no baseline.
- [ ] **5.6** The frontier's baseline class, held component-versions and
  `not_built`; amends D22.
- [ ] **5.7** `antora`'s Resolve, called in a CI run.
- [ ] **5.8** The gate: `report introduced`, the exit status, `check-dir`'s
  output and manual page.
- [ ] **5.9** The inverse check: `report inverse` and `--inverse`.
- [ ] **5.10** The end-to-end test's planted changes and assertions.
- [ ] **5.11** quick-docs as published and changed, against the en-US
  baseline.

## Not started

P6 onward in [`../design/roadmap.md`](../design/roadmap.md)'s index. None has
been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".
