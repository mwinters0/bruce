# Roadmap

Where this project is going. A phase that has been specified gets its own doc,
`docs/wip/roadmap-P<N>-<slug>.md`, while it is in flight. That doc moves to this
directory at its wrap. The sections below are the index. A phase still sketched
here is sketched only deeply enough to keep current work from painting us into
a corner. **Each gets its own full grilling session when it becomes current**,
and the resulting spec becomes its own doc.

**`P<k>` is an identifier, not a position** (`../process.md`, "Naming and
lifecycle"). Phases are allocated numbers in the order they are found and run in
the order that suits. **The order is this table, top to bottom.** Numbers are
never reused.

| Phase | State | Where it is |
|---|---|---|
| P1–P5 | **Struck** at the first keystone, 2026-10-21 | their decisions are [`decisions.md`](decisions.md)'s |
| P6 — release | **Struck** at the second keystone, 2026-10-22 | its decisions are [`decisions.md`](decisions.md)'s |
| P8 — run-history reports | **Complete**, 2026-10-25 | [`roadmap-P8-history.md`](roadmap-P8-history.md), its notes [`roadmap-P8-history-notes.md`](roadmap-P8-history-notes.md) |
| P9 — the deployment guide | Specified; to be grilled again after P8 | [`../wip/roadmap-P9-deployment.md`](../wip/roadmap-P9-deployment.md) |
| P7 — declarative per-root rules | Sketched; not grilled | this file, below |

**A row's state is one of `Sketched`, `Specified`, `Current`, `Complete` or
`Struck`.** The prose after it is a caption. Two states are set in the same
change as a `STATUS.md` edit:
- **`Current` when the phase is sliced**, in the change that writes its
  checklist;
- **`Complete` at the wrap**, in the change that deletes the checklist.

A phase carrying a checklist is `Current`, and a `Current` phase carries one.
`scripts/deficiencies.py` reads this column and holds that pairing both ways.

## Project goals

A general-purpose link checker, usable on any site without customization, that
extends naturally to a site's generator and deployment. docs.fedoraproject.org
is the primary target, and its tie-back from each page to its Antora source is
the extension the maintainer's own use needs; another user's extension should
fit the same way. What existing tools don't provide:

- **Fragment validation**: `page#anchor` links whose page loads but whose
  anchor is missing are counted separately from broken URLs, as a per-page
  stat.
- **A persistent link graph** in DuckDB: every page checked, when it was
  checked, and what it links to. This yields the sitemap, inbound-link
  rankings, orphans, and run-to-run diffs.
- **A CI gate** for a single content repo's partial Antora build, failing on
  breakage the change introduced (D4, D9).
- **External links** checked only for allowlisted hosts (D3).
- **Fast**, with configurable politeness (D8, D18).

**CI and manual use are equals**: the gate run unattended in a content repo's
CI, and every command run by hand, each serve their purpose fully; neither is
the other's afterthought.

**A broken experience is reported whatever breaks it**: a link that does not
take its reader where it means to is broken whether the page's markup, the
server's configuration or a redirect between them is at fault, and where the
fix is known its reason names it.

The decisions are in [`decisions.md`](decisions.md), and the external facts
they rest on are in [`site-invariants.md`](site-invariants.md). The bootstrap
plan is frozen as [`historical/initial.md`](historical/initial.md).

## Standing rules

A standing rule is added only with the maintainer, and carries its check.

- **A config key is documented in the change that adds it**: a row of
  `../manual/config.md`'s `Key | Default | Meaning` tables, its default
  stated, and no row for a key the config does not read. Check:
  `config::tests::every_key_the_config_reads_is_in_the_manual_with_its_default`.
- **No workflow uses a third-party action**: every step under
  `.github/workflows/` is a command in the job's container, a checkout
  included, and what an action would give is built from commands. Check:
  `scripts/workflows.py`, which fails on a `uses:` (`mise run check` runs its
  tests over the real workflows).

## P8 — Run-history reports

Complete: [`roadmap-P8-history.md`](roadmap-P8-history.md) and its notes,
[`roadmap-P8-history-notes.md`](roadmap-P8-history-notes.md). A run's digest
as dated JSON and HTML, and the two-run reports `changed` and
`anchors-removed`.

## P9 — The deployment guide

Specified: [`../wip/roadmap-P9-deployment.md`](../wip/roadmap-P9-deployment.md).
It follows P8, whose schema change a 0.x release would make its first
deployers pay for.

## P7 — Declarative per-root rules

TOML rules that let a site extend extraction without writing Rust: a selector
and attribute giving a fact, a content-region selector, and a URL regex giving
a source (D33). They write the same key/value facts a compiled profile does
(D37). Resolution logic stays code. Scheduled when a site outside the compiled
profiles needs one.
