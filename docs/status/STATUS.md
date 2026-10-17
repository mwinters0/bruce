# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: 5404bcf --> The marker names the commit `scripts/repoint.py`
measures the record's growth from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

The `bruce` crate is a scaffold: D39's top-level modules, a binary that says
no subcommand exists, and Rust 1.99.1 pinned in `mise.toml`. Four modules hold
code: `config`, the loader; `store`, the DuckDB schema, its version check and
the single writer for two tables; `profile`, the trait, the registry and the
`generic` and `antora` profiles' Detect, Views and content region, with Resolve
declared and empty; and `extract`, the pass over one page, which yields its
profile, title, anchors with placement and links with kind. The rest hold only their doc.
`docs/manual/` holds the config reference. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`.

## P2 progress

The spec is [`../wip/roadmap-P2-extractor.md`](../wip/roadmap-P2-extractor.md).

- [x] **2.1** The pass and its anchors: the Extract hook's new shape, the
  generator pre-pass, the title, anchors with tag and placement, the fixture
  snapshots, and the bench moved into `bruce`
  ([notes](../wip/roadmap-P2.1-pass-and-anchors-notes.md)).
- [x] **2.2** Links: every kind, `srcset`, `<base href>` and `refresh`,
  resolved against the page's URL
  ([notes](../wip/roadmap-P2.2-links-notes.md)).
- [ ] **2.3** The edit link and `forge`: the core ranking and the shape parse.
- [ ] **2.4** Antora's Extract: its region, coordinates and unresolved-xref
  tags.
- [ ] **2.5** The content hash: xxh3-128 over a scope set by `hash_root` and
  `hash_exclude`.

## Not started

P3 onward in [`../design/roadmap.md`](../design/roadmap.md)'s index. None has
been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".

