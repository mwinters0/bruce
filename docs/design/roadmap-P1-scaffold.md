# P1 — Scaffold and parser spike

What P1 does and why. It is complete; what it left for later phases is
[`roadmap-P1-scaffold-notes.md`](roadmap-P1-scaffold-notes.md). The decisions
are in [`decisions.md`](decisions.md) and are cited here, never restated.

## Intent

The goal widened before any code was written: a general-purpose checker usable
without customization, extended by a site's generator and deployment
(`../design/roadmap.md`, "Project goals"). P1 draws the line between the
generic core and those extensions (D32–D38). It lays out the modules along it
(D39), gathers the evidence that picks the tokenizer (D11), and lands the
scaffold, the config and the store everything later builds on.

**The first two slices produce evidence**, so the later slice numbers are in
allocation order, not schedule (`../process.md`, "Naming and lifecycle").

## The slices

### P1.1 Fixtures

Saved pages for D24, and no `bruce` code: a small save script and the pages.
- **About 30 real docs.fp.o pages**, chosen by measurement for shape: the
  largest page, pages with tabs (I9), pages with unresolved xrefs (I6), an
  unversioned component (I8), a frozen `f3x` page with a canonical link (I5),
  and a 404. Fedora's docs are CC BY-SA 4.0, so they are committed with
  attribution.
- **Small hand-written pages for the other generators**, reproducing only the
  markup I12 records: the edit links of Docusaurus, Material, Docsy and Furo,
  and pages with and without landmarks (D34). They test which link a heuristic
  picks, which needs the markup, not the content.

### P1.2 Tokenizer spike

html5gum against lol_html (D11), in a standalone Cargo project at
`spikes/tokenizer/`, outside the `bruce` crate. It is committed, so the numbers
can be reproduced from a hash. `concepts.py` and the size limits never see it.
Both tokenizers run the workload profiles will actually run:
- the core extraction: ids, `name` on `<a>`, and links with their kinds (D12,
  D13);
- a prototype Antora extract: the generator meta, the nav-container
  attributes, the edit link chosen by `title` among Fedora's three, the
  `article.doc` region, and unresolved-xref tagging;
- the generic heuristics: the landmarks (D34) and the ranked edit-link
  candidates (D36).

Speed is measured with criterion on the docs.fp.o fixtures only, since that is
the nightly workload. Correctness is measured on all of them. The code each
tokenizer needs is compared beside its speed, and streaming selector matching
weighs in the choice (D11).
- **Outcome:** D11 names the winner, and the notes record the numbers and the
  commit.
- D38's Extract signature is drawn from the winner. The prototypes are
  thrown away.
- P2's extractor slice deletes `spikes/tokenizer/`, and moves the permanent
  parser bench (D24) into `bruce`.

### P1.3 Scaffold

- **The crate `bruce`** (D30, D31), edition 2024.
- **Rust pinned exactly at 1.99.1** in `mise.toml`, the stable release of
  2026-10-13, as that file's own rule against floating versions asks.
  `rust-version = "1.99"` in `Cargo.toml`. If mise cannot install 1.99.1 yet,
  the slice pins 1.99.0 and says so.
- **The mise tasks:** build, test, clippy, fmt and the bench, joined to
  `mise run check` (D23).
- **The empty top-level modules of D39**, the first concept-map entries
  (`../design/concepts.md`), and a check that only `cli` names a concrete
  profile, since every standing constraint carries a check.

### P1.4 Config

- **The loader** (D5): every key with D40's default, so a run needs no file,
  and each root's `profile` (D35).
- **The manual starts here.** `docs/manual/` gets a config reference that
  documents each key and its default in the change that adds the key. The
  deployment guide joins it in P4 (D42).

### P1.4.1 Config keys per host

P1.4 put `robots` and `crawl_delay` on a root, though D8 limits per host. The
maintainer moved per-host settings to `[host."<h>"]`, folded `crawl_delay`
into its `per_second`, and refused a root starting outside its own scope (`../status/history/2026-10-17.md`, "P1.4's config call reviewed:
per-host keys, a root inside its scope"). The slice changes the loader, the
manual, and D5, D8, D18, D40 and D44 together.

### P1.5 Store and the profile trait

- **The schema** (D26) with `meta` (D43) and `page_facts` (D37).
- **The single-writer skeleton** (D28).
- **The `Profile` trait** (D38), drawn from P1.2's winner:
  - `generic` and `antora` implement Detect and Views;
  - Extract and Resolve are declared here, and their bodies land in P2 and
    P5;
  - it lands here because the store's Views hook needs it, and the spike's
    evidence is freshest right after P1.2.

P1's schema differs from the frozen draft (`historical/initial.md`, "D26"):
- `pages` gains `profile` (D35), `source_ref` and `source_path` (D36);
- component, version, module and `rel_path` move to `page_facts` (D37);
- `anchors.in_content` becomes `placement` (D34);
- `meta` is added (D43).

## Where the concepts live

Each concept P1 names, the top-level module that will hold it, and the
decisions it carries. The layering they sit in is D39. Each enters
`../design/concepts.md` with the slice that creates its files.

| Concept | Module | Decisions |
|---|---|---|
| Subcommands | `cli` | D27 |
| The crawl | `crawl` | D7, D8, D14, D16, D18, D40 |
| CI mode | `ci` | D4, D9, D21, D22 |
| Reports and pruning | `report` | D2, D6 |
| Fetchers | `fetch` | D1 |
| The extraction pass and its core heuristics | `extract` | D12, D13, D34, D36 |
| Storage | `store` | D26, D28, D37, D43 |
| Profiles | `profile` (the trait, `generic/`, `antora/`) | D32–D35, D38 |
| Forge edit-URL shapes | `forge` | D36 |
| Canonical URLs and IDs | `url` | D7, D26 |
| Configuration | `config` | D5, D35, D40 |

## What the split was drawn from

Before the grilling, each decision knew one of three things:
- **The generic web**: D2, D3, D7, D8, D9, D13, D14, D16, D21, D26, D28, and
  D12's anchor rule.
- **Antora, on any site**: D15's coordinates, D19, D20, D22's refspec
  resolution, and D12's content region.
- **Fedora's deployment**: I1's sitemap location, I2's `Crawl-delay`, D17's
  edit-URL parse, and D21's baseline hosting.

The maintainer is open to a third, deployment layer of code later, if a
docs.fp.o quirk turns up that config can't hold (D32's "Reopens").

## Facts found while grilling

Gathered 2026-10-17. "Source" means a template read upstream; "observed" means
live HTML. What a decision relies on is in `site-invariants.md` (I1, I12); the
rest is here for the slices that will need it.

- **Antora's default UI** (antora-ui-default `0e38223a`, source):
  - `<meta name="generator" content="Antora x.y.z">` (`head-info.hbs`).
  - `nav-container` carries `data-component`/`data-version` only when the page
    has a component (`nav.hbs:1`).
  - `<article class="doc">` (`article.hbs:1`).
  - The edit link is `div.edit-this-page > a`, with text "Edit this Page", no
    title and no class. It is suppressed under `env.CI`, or for a private
    origin.
- **Antora's automatic edit URLs** (v3.1.15 `compute-origin.js`, source) exist
  for github.com, gitlab.com, bitbucket.org and pagure.io only. Forgejo needs
  an explicit `edit_url`.
- **docs.fp.o's UI bundle** (ui-bundle `7f5fb3d9`, source and observed):
  - The generator meta, nav-container data attributes and `article.doc` match
    the default UI, with extra classes. Match on class tokens.
  - There are three icon-only `div.edit-this-page` links, told apart only by
    `title`: "Page history", "Edit this Page", "Report an issue".
- **Other Antora sites** (observed):
  - All emit the generator meta (3.2.x).
  - docs.asciidoctor.org and docs.couchbase.com put no data attributes on
    `nav-container`.
  - Their edit links vary: default UI, Spring's `div.sidebar-links`, and
    Couchbase's `li.tool.edit` with `title="Edit Page"`.
- **Other generators** (observed, Docsy from source):
  - **Generator meta:** Hugo emits one only if the theme asks, and Docsy
    doesn't. MkDocs Material and Docusaurus emit one. Sphinx emits none.
  - **Edit-link markers:** Docsy `a.td-page-meta--edit`; Material
    `a[rel=edit][title="Edit this page"]`; Docusaurus `a.theme-edit-this-page`;
    Furo `div.edit-this-page > a[rel=edit]`; RTD theme only the link text
    "Edit on GitHub".
- **Forge URL shapes** (observed):
  - Forgejo `/src/branch/<ref>/<path>`.
  - GitHub `/blob/` and `/edit/<ref>/<path>`.
  - GitLab `/-/blob/` and `/-/edit/`; a bare `/edit/` redirects to `/-/edit/`.
  - Pagure `/blob/<ref>/f/<path>` comes from source only and is unverified
    live.
- **Rust stable** is 1.99.1, released 2026-10-13
  (`static.rust-lang.org/dist/channel-rust-stable.toml`). `mise ls-remote
  rust` lists up to 1.99.0.
