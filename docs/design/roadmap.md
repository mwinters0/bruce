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
| P1 — scaffold and parser spike | Complete | [`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md), [notes](roadmap-P1-scaffold-notes.md) |
| P2 — the extractor | Complete | [`roadmap-P2-extractor.md`](roadmap-P2-extractor.md), [notes](roadmap-P2-extractor-notes.md) |
| P3 — the crawler | Current | [`../wip/roadmap-P3-crawler.md`](../wip/roadmap-P3-crawler.md) |
| P4 — verdicts and reports; the first en-US crawl | Sketched; not grilled | this file, below |
| P5 — CI mode: baseline and `check-dir` | Sketched; not grilled | this file, below |
| P6 — release and the deployment guide | Sketched; not grilled | this file, below |
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

The decisions are in [`decisions.md`](decisions.md), and the external facts
they rest on are in [`site-invariants.md`](site-invariants.md). The bootstrap
plan is frozen as [`historical/initial.md`](historical/initial.md).

## Standing rules

None yet. A standing rule is added by a grilling, and only with the
maintainer.

## P1 — Scaffold and parser spike

The spec is
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md): the split
between the generic core and its profiles (D32–D38), the layering (D39),
fixtures, the tokenizer spike, the scaffold, the config and the store. What it
left for later phases is [`roadmap-P1-scaffold-notes.md`](roadmap-P1-scaffold-notes.md).

## P2 — The extractor

The spec is [`roadmap-P2-extractor.md`](roadmap-P2-extractor.md):
the streaming pass over one page, the Extract hook's new shape (D38), link
kinds (D13), the edit link and its forge parse (D36), Antora's coordinates
(D19) and a content hash scoped by config (D15). What it left for later
phases is [`roadmap-P2-extractor-notes.md`](roadmap-P2-extractor-notes.md).

## P3 — The crawler

The spec is [`../wip/roadmap-P3-crawler.md`](../wip/roadmap-P3-crawler.md):
the canonical form and `url_id` (D7, D26), what each URL class costs a
request, redirects and meta-refresh aliases as hops, per-host politeness (D8,
D18), sitemaps (D14), the store's remaining rows (D28) and `bruce crawl`,
closing with a live crawl of one docs.fp.o component.

## P4 — Verdicts, reports, and the first en-US crawl

Post-crawl fragment verdicts in SQL (D28), outcome classification (D16),
`bruce report` (D6, D27) and `bruce prune` (D2). Ends with a real en-US crawl,
which delivers the sitemap, inbound ranking and both broken stats before any CI
work, run from the simulated deployment (D41, D42). The deployment guide starts
here, with the config section that crawl needs. A full crawl at docs.fp.o's
`Crawl-delay` runs about 40 minutes, so it is
a long job (`CLAUDE.md`, "Long-running processes").

## P5 — CI mode: baseline and `check-dir`

`bruce export-baseline` (D21), the local fetcher and its resolution order (D1,
D22), resource-ID resolution as `antora`'s Resolve hook (D19, D38), the inverse check and exit status (D4, D9),
and the end-to-end Antora test (D24).

## P6 — Release and the deployment guide

A release binary on the forge (D23). The manual's guide to configuring a
deployment: its config, a scheduled crawl, and publishing a baseline for CI
(D21). The guide is verified against the simulated deployment (D41). Running
docs.fp.o's nightly crawl is the deployer's job, not this repo's.

## P7 — Declarative per-root rules

TOML rules that let a site extend extraction without writing Rust: a selector
and attribute giving a fact, a content-region selector, and a URL regex giving
a source (D33). They write the same key/value facts a compiled profile does
(D37). Resolution logic stays code. Scheduled when a site outside the compiled
profiles needs one.
