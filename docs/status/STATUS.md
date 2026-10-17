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
no subcommand exists, and Rust 1.99.1 pinned in `mise.toml`. Five modules hold
code: `config`, the loader; `store`, the DuckDB schema, its version check and
the single writer for two tables; `profile`, the trait, the registry and the
`generic` and `antora` profiles' Detect, Views and Extract, with Resolve
declared and empty; `extract`, the pass over one page, which yields its
profile, title, anchors with placement, links with kind and tag, the edit link
and the profile's facts; and
`forge`, the edit-URL shape parse. The rest hold only their doc.
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
- [x] **2.3** The edit link and `forge`: the core ranking and the shape parse
  ([notes](../wip/roadmap-P2.3-edit-link-notes.md)).
- [x] **2.4** Antora's Extract: its region, coordinates and unresolved-xref
  tags ([notes](../wip/roadmap-P2.4-antora-extract-notes.md)).
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

- **`antora` takes a component from the URL only where the page has no nav
  container at all** (P2.4). A UI whose nav container omits `data-component`
  (asciidoctor.org, couchbase.com, I12) therefore records no component, and
  D19 cannot resolve an xref into its pages. That follows the spec's wording
  ("Antora's coordinates": "a page with no nav container"), whose evidence is
  docs.fp.o's `without_menu` pages alone. Keying the fallback on a missing
  `data-component` instead would cover those UIs, but they have versioned
  components, and a component from the URL with no version reads as the
  latest (D19), which on a frozen version's page is wrong. Reconsidering
  changes one condition in `Page::finish` (`src/profile/antora.rs`) and D19's
  wording.
