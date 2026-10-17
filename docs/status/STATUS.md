# Status

What is built, right now. Rewritten in place as state changes. The decisions
the code cannot explain are [`../design/decisions.md`](../design/decisions.md).
What is still ahead is [`../design/roadmap.md`](../design/roadmap.md), whose
index table is the schedule. Where what is built falls short is
[`deficiencies.md`](deficiencies.md). Dated pickup notes and plan-changing
discoveries are in `history/`.

<!-- repointed: d8a6d8b --> The marker names the commit `scripts/repoint.py`
measures the record's growth from, here the commit P1's code starts from.
Red means a repoint is due ([`../process.md`](../process.md),
"Repointing").

## What exists

No Rust. The design is settled ([`../design/decisions.md`](../design/decisions.md))
and the development process is in place: [`../process.md`](../process.md), the
`go`, `gosub`, `dwal`, `process` and `grilling` skills, and `mise run check`
over the docs. `fixtures/` holds 30 saved docs.fp.o pages and ten hand-written
pages, chosen and saved by `scripts/fixtures.py`.

## P1 progress

The spec is [`../wip/roadmap-P1-scaffold.md`](../wip/roadmap-P1-scaffold.md).

- [x] **1.1** Fixtures: 30 docs.fp.o pages chosen by measurement, and
  hand-written pages for other generators' markup, with a tabs page
  reconstructed from I9.
  [notes](../wip/roadmap-P1.1-fixtures-notes.md)
- [ ] **1.1.1** A real docs.fp.o page with tabs (I9): none is among the 698
  sampled. Scan the unsampled sitemap URLs, a detached job of about 35 minutes,
  or take a page the maintainer names; if none exists, strike the row and amend
  I9.
- [ ] **1.2** Tokenizer spike: html5gum against lol_html in `spikes/tokenizer/`;
  D11 names the winner.
- [ ] **1.3** Scaffold: the crate, the Rust pin, the mise tasks, the D39
  modules, and the check that only `cli` names a concrete profile.
- [ ] **1.4** Config: the loader with D40's defaults, and the manual's config
  reference.
- [ ] **1.5** Store and the profile trait: the schema with `meta` and
  `page_facts`, the writer skeleton, and the `Profile` trait.

## Not started

P2 onward in [`../design/roadmap.md`](../design/roadmap.md)'s index. None has
been grilled.

## Decisions worth another look

Calls made without the maintainer present that a person should still weigh in
on. They are cautionary and informational, never blocking. **Capped at five.**
Closing an entry means filing it and then deleting it, done by the session that
hears the answer. Where the review affirms a call and changes nothing, its
reasoning goes beside the mechanism it governs first. Full rules:
[`../process.md`](../process.md), "Decisions worth another look".

1. **I8 left as written though six pages falsify it.** `arm-sbc`, `docs`,
   `engineering`, `mindshare`, `spins-labs` and `tools` have no `nav-container`,
   so no component or version, though I8 says every page names both. P1.1
   found them in a 698-page sample (`fixtures/docs-fp-o/survey.json`; file
   `fixtures/docs-fp-o/en-US/arm-sbc/index.html`). An unattended session may
   not amend a standing register (`../process.md`, "Working unattended"), so I8
   stands, `arm-sbc/` is a fixture, and the extractor's inbox carries the fact
   (`../design/roadmap-P2-extractor-inbox.md`). Amending I8 to "every page of a
   component", and deciding what D17, D19 and D34 do for a page with none,
   changes only those entries' wording unless the maintainer wants these pages
   to carry a component of their own.
