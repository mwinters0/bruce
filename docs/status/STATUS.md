# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 6e81a2f --> The marker names the commit `scripts/repoint.py`
measures the record's growth from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

The `bruce` crate is a scaffold: D39's top-level modules, a binary that says
no subcommand exists, and Rust 1.99.1 pinned in `mise.toml`. Seven modules hold
code: `cli`, the profile registry; `config`, the loader; `store`, the DuckDB schema, its version check and
the single writer for two tables; `profile`, the trait, the registry and the
`generic` and `antora` profiles' Detect, Views and Extract, with Resolve
declared and empty; `extract`, the pass over one page, which yields its
profile, title, anchors with placement, links with kind and tag, the edit link,
the profile's facts and the content hash; and
`forge`, the edit-URL shape parse; and
`url`, the canonical form and `url_id`. The rest hold only their doc.
`docs/manual/` holds the config reference. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`.

## P3 progress

The spec is [`../wip/roadmap-P3-crawler.md`](../wip/roadmap-P3-crawler.md).

- [x] **3.1** Canonical URLs: `url`, `strip_query`, and scope matched on the
  canonical form; amends D44. [Notes](../wip/roadmap-P3.1-canonical-urls-notes.md).
- [ ] **3.2** The store's rows: schema 2, hop structs, the `sitemap` kind, and
  the writer's `targets`, `anchors` and `links`; amends D13.
- [ ] **3.3** One hop over HTTP: the fetcher trait, reqwest, retries, the size
  cap and the charset.
- [ ] **3.4** Per-host politeness: rate, `concurrency`, robots.txt and
  `user_agent`; amends D11.
- [ ] **3.5** The crawl: the frontier, URL classes, hops, outcomes and the
  writer feed.
- [ ] **3.6** Sitemaps: discovery, indexes, gzip and `sitemap` links.
- [ ] **3.7** `bruce crawl`: the subcommand, Ctrl-C and the manual; amends D5.
- [ ] **3.8** The live crawl of `/en-US/fedora-docs/`.

## Not started

P4 onward in [`../design/roadmap.md`](../design/roadmap.md)'s index. None has
been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".

None open.
