# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: df2cf7c --> The marker names the commit `scripts/repoint.py`
measures the record's growth from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

The `bruce` crate is D39's top-level modules, a binary whose subcommands
are `bruce crawl` and `bruce report`, and Rust 1.99.1 pinned in `mise.toml`. Ten modules hold
code: `cli`, the command line, the two subcommands and the profile registry; `config`, the loader; `store`, the DuckDB schema, its version check, the
`link_verdicts` view and the single writer for `pages`, `page_facts`, `targets`, `anchors` and `links`; `profile`, the trait, the registry and the
`generic` and `antora` profiles' Detect, Views and Extract, with Resolve
declared and empty; `extract`, the pass over one page, which yields its
profile, title, anchors with placement, links with kind and tag, the edit link,
the profile's facts and the content hash;
`forge`, the edit-URL shape parse;
`url`, the canonical form and `url_id`;
`fetch`, the fetcher trait, the HTTP fetcher for one hop, each host's pace, in-flight cap and robots.txt, and the body decoding; and
`crawl`, the frontier, the hops of a redirect, D16's outcomes, sitemaps, the run that writes a target's rows and its stop; and
`report`, the `report_*` views, the choice of run, and the eight single-run reports as a text table or JSON. The rest hold only their doc.
`docs/manual/` holds the config reference, `bruce crawl`'s, `bruce report`'s and the deployment guide's config section, which `mise run sim` extracts into `sim/` and checks. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`.

## P4 progress

The spec is [`../wip/roadmap-P4-reports.md`](../wip/roadmap-P4-reports.md).

- [x] **4.1** Schema 3: `links`' verdict columns dropped, `pages.root` and
  `runs.roots`; closes KD3.
  [Notes](../wip/roadmap-P4.1-schema-3-notes.md).
- [x] **4.2** `skip`: the config key and the frontier's `skipped`.
  [Notes](../wip/roadmap-P4.2-skip-notes.md).
- [x] **4.2.1** A skipped hop ends the chain; amends D50.
  [Notes](../wip/roadmap-P4.2.1-skipped-hop-notes.md).
- [x] **4.3** The deployment guide's config section and `mise run sim`.
  [Notes](../wip/roadmap-P4.3-deploy-guide-notes.md).
- [x] **4.3.1** The guide's example: robots.txt and the skip glob; amends
  D18.
  [Notes](../wip/roadmap-P4.3.1-guide-example-notes.md).
- [x] **4.4** The first en-US crawl, detached from `sim/`.
  [Notes](../wip/roadmap-P4.4-first-crawl-notes.md).
- [x] **4.5** `link_verdicts`, timed on 4.4's DB; amends D28.
  [Notes](../wip/roadmap-P4.5-link-verdicts-notes.md).
- [x] **4.6** `bruce report`: the run, filters, formats, and `broken`,
  `errors` and `stats`.
  [Notes](../wip/roadmap-P4.6-report-notes.md).
- [x] **4.6.1** `stats` counts unchecked targets beside links.
  [Notes](../wip/roadmap-P4.6.1-stats-targets-notes.md).
- [x] **4.7** `inbound`, `sitemap`, `orphans`, `unlisted` and `redirects`.
  [Notes](../wip/roadmap-P4.7-page-reports-notes.md).
- [ ] **4.7.1** `unlisted`'s roots: canonical, and read only when a file
  answered.
- [ ] **4.8** `diff`.
- [ ] **4.9** `prune`.
- [ ] **4.10** The second en-US crawl, with the finished tool.

## Not started

P5 onward in [`../design/roadmap.md`](../design/roadmap.md)'s index. None has
been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".
