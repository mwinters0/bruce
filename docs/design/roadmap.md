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
| P1–P8 | **Struck** at the keystones of 2026-10-21 to 2026-10-31 | their decisions are [`decisions.md`](decisions.md)'s |
| P10 — declarative extraction rules | **Complete**, 2026-11-01 | [`roadmap-P10-extraction.md`](roadmap-P10-extraction.md), its notes [`roadmap-P10-extraction-notes.md`](roadmap-P10-extraction-notes.md) |
| P11 — the digest's prose in config | Sketched; not grilled | this file, below |
| P9 — the deployment guide | Specified; follows P11, and is grilled again before it is sliced | [`../wip/roadmap-P9-deployment.md`](../wip/roadmap-P9-deployment.md) |
| P12 — compiled profiles for other generators | Sketched; not grilled | this file, below |
| P13 — a selection language beyond CSS | Sketched; not grilled | this file, below |

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

## P10 — Declarative extraction rules

Complete: [`roadmap-P10-extraction.md`](roadmap-P10-extraction.md), and its notes
[`roadmap-P10-extraction-notes.md`](roadmap-P10-extraction-notes.md).

## P11 — The digest's prose in config

A deployment replacing the digest's built-in text with its own, the guide
that closes the HTML ("How to read this report") first: an entry's text
overridden or an entry added, read from the deployment's config, not a
compiled profile (D32), and perhaps a template filled from the run. The
maintainer foresees more overrides and basic templating; which texts, and
whether a template reads the run's recorded policy, is the grilling's.
Follows the site policy, which decides what the guide's text may claim
(D91), and precedes P9, whose guide documents the config.

## P9 — The deployment guide

Specified: [`../wip/roadmap-P9-deployment.md`](../wip/roadmap-P9-deployment.md).

## P12 — Compiled profiles for other generators

Profiles compiled in for generators beyond Antora (Sphinx, Docusaurus, MkDocs
Material, Docsy), each saved site its evidence, and with them the line between
compiled and config that P10 drew by best effort against docs.antora.org
(P10's spec, "P10 draws the line by best effort, and ships no new
generator"). Detection is its own question: Sphinx and Docsy emit no generator
meta (I12), so D35's `auto` cannot pick them. **v1.0 waits on it**
(maintainer, 2026-10-31): a 1.0 needs high confidence in what a site can
configure and in the config format's stability, which only several kinds of
site supported can give.

## P13 — A selection language beyond CSS

What a site's rules and `generic`'s heuristics select by, beyond lol_html's
streaming CSS (D11): something like XPath or a document-level selector, so a
site names its edit link, locale menu or region by structure and text rather
than the core matching link titles and phrases (D36). P10's selectors are CSS
under keys that leave room for it (P10's spec, "Selectors are CSS, under a key
that leaves room"). A DOM-level language reopens D11's one streaming pass.
**v1.0 waits on it** (maintainer, 2026-10-31), as it does on P12.
