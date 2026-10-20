# P5 — CI mode: baseline and `check-dir`

What P5 does and why, settled in its grilling on 2026-10-20, which drained
its inbox. The decisions are in [`../design/decisions.md`](../design/decisions.md)
and are cited here, never restated.

## Intent

A content repo's CI checks its partial Antora build (I11) against a baseline
exported from a scheduled full crawl, and fails only on the breakage the
change introduced (D4, D9, D21, D22). P5 delivers `bruce export-baseline`,
`bruce check-dir` with the local fetcher and D22's resolution order,
`antora`'s Resolve hook for unresolved xrefs (D19, D38), the gate with its
inverse check and exit status (D9, D16), and D24's end-to-end Antora test. It
closes on evidence: a real partial build of one content repo checked against
a baseline exported from P4's second en-US crawl.

## Scope

Settled 2026-10-20. **Resolve and the inverse check are in**: without
Resolve, every xref into a component the partial build lacks renders as
`#<refspec>` (I6, I10) and reads as `broken_url`, so a repo linking into many
components, as quick-docs does, is red on every PR; without the inverse
check, removing an anchor others link to passes, where D9 fails it by
default. **D20's "did you mean" hint is not P5's**: it changes no CI verdict,
being live-report text, and its fact moved to P8's inbox. **D24's synthetic
2–3-component Antora site is the repeatable test; the real build is the
closing evidence**, as P3 and P4 closed on live crawls.

## The baseline is the store's schema holding one run

Settled 2026-10-20. **`bruce export-baseline` copies one run's rows into a
fresh DB file stamped with the current `schema_version`**, and `check-dir`
attaches it, so `link_verdicts`, the `report_*` views and D43's refusal work
on it unchanged, and the gate compares like with like (D28). Rejected: a slim
purpose-built schema, a second schema with its own version and queries.
Trimming the copy (chrome nav links repeat on every page) is a later,
measured change; P5 reports the copy's size, gzipped and not.

## A `check-dir` run is a crawl

Settled 2026-10-20. **`check-dir` is the crawl driver with a second
`Fetcher` (D1)**, seeded with every `.html` file under the build directory,
a partial build's sitemap not being trusted. A built URL under the prefix is
read from disk; one under the prefix and not built, or outside it, is
answered from the attached baseline, then live, per D22 and `--externals`.
The run lands in the local DB as an ordinary run, so `bruce report` reads it
(D27), and **the gate compares two runs** on `report_broken`'s key: the
linking page's `url_id`, the target URL and the fragment (`''` for none).
Rejected: a separate pipeline in `ci` that walks the directory and queries
the baseline without writing targets, which rebuilds the frontier, the hops
(a local meta-refresh alias is still a hop), D16's outcomes and the writer.

### A baseline answer is copied into the run

Settled 2026-10-20. **A URL in a root's scope and not built is a new
frontier class, answered by copying the baseline's rows for it into the
run**: its target, its landing target where it redirected, that page's
`pages` row and its `anchors`, the rows `link_verdicts` joins. A URL the
baseline holds no target for falls through to live. **Each target records
where its answer came from**, `local`, `baseline` or `live`, in a new
`targets` column: schema 4, no migration (D43). Rejected: copying nothing
and giving `link_verdicts` a second path into the attached baseline, the
verdict logic then living twice, which P4 refused in making it a view.

### The baseline is imported as a run of its own

Settled 2026-10-20. **`check-dir` copies the baseline file's run into the
local DB as a run, and the CI run's `runs.baseline_run` names it**, the
column the bootstrap schema drew for it and nothing writes yet
(`src/store/schema.rs`). Out-of-build answers are copied from that run, and
the gate is a two-run query in one DB, the shape `diff` has: `bruce report
diff` reads a baseline against its CI run, and `prune` treats it as any run.
**A baseline already imported is reused**, matched on its run's
`started_at` and `config_hash`, so repeated local runs do not pile up
copies. Rejected: attaching only, the gate then reading across two DBs and
`baseline_run` naming a run in a file gone after the job.

### `export-baseline` and `--baseline`

Settled 2026-10-20 (D21). **`bruce export-baseline --out <file> [--run N]
[--db f]`** takes the latest finished run by default and **refuses an
unfinished or stopped run**, whose missing targets (D48) would each become a
live request in every CI job; it refuses an existing `--out` without
`--force`. **`check-dir --baseline <path|https-url>`** downloads a URL to a
temporary file and decompresses a `.gz` path or URL; nothing is cached, a CI
job caching it if it wants and the import's reuse covering local reruns. The
export slice measures gzip on P4.10's run, for the guide (P6) to say whether
to publish it gzipped.

### What a build holds

Settled 2026-10-20, amending D22. **A build holds the component-versions
its built pages carry**, `antora.component` and `antora.version` from the
nav container, which renders whatever `CI` is. Then:

- **An in-scope URL not built, in a held component-version, is
  `broken_url`, reason `not_built`**: the build is the truth for what it
  holds, and the baseline answers only outside it. Otherwise a link to a
  page the PR deletes passes on the baseline's `ok`.
- **Resolve goes through the baseline only for a refspec whose
  component-version is not held**; an unresolved xref into a held one stays
  `unresolved_xref`, Antora having had that component's whole catalog.
- **The inverse check's touched pages are the baseline's in a held
  component-version** (below, "The inverse check").

Accepted: another repo's component in the playbook is built from its branch
as it is now, so what it changed since the baseline's crawl shows in this
PR's gate, real breakage though not the PR's; each row names its component.
**Two limits are `KD<k>`s when they land**: a component split across
repos' start paths (Antora allows it) and partly built reads its unbuilt
pages as `not_built`, which quick-docs' build does not do; and a `generic`
page carries no fact, so a `generic` build holds nothing, D22 applies as
first written, and a link to a page the PR deletes passes, `(c)` until a
non-Antora site is checked in CI. Rejected: a flag naming the globs a PR
owns, which every repo's CI keeps current by hand.

### Root-relative hrefs resolve against the prefix

Settled 2026-10-20. **In a local build, an href beginning `/` resolves
against the prefix**: the build directory's root is the site's root, what a
static server serving it at `/` shows. The UI's footer writes
`/legal/privacy/` with `site.path` empty, which against the production
origin is a 404 on every built page. Rejected: the production origin, with
the guide telling deployers to `skip` those URLs, working around a false
break the tool made.

### Resolve

Settled 2026-10-20 (D19, D20). Resolve sees only a refspec whose
component-version the build does not hold, so only one naming its own
component: a built page's missing module never matters.

- **The parse** is D19's, `[version@]component:[module]:page`, an empty
  module `ROOT`, a missing version the baseline's only or latest one.
- **The lookup** is `Lookup`'s, into the imported baseline run, on the
  `component`, `version`, `module` and `rel_path` facts. A baseline page
  with no edit link has no module or `rel_path` (168 of P4.10's 2320
  `antora` pages) and is no target: a `KD<k>` when it lands, such an xref
  staying unresolved.
- **A resolved link's target is the resolved URL plus the refspec's
  fragment**, `xref_refspec` kept, and it is judged as any link; one that
  does not resolve stays `unresolved_xref`.
- **A live crawl never calls Resolve**: production's catalog is whole, so
  its unresolved xrefs are broken (D20).

### The config is the deployment's, and `--prefix` maps the build

Settled 2026-10-20. **`check-dir` reads the config file `crawl` reads**, so
`skip`, the allowlist, `[host]`, the roots' scopes and the profile choice
plan a URL as they did for the baseline; a difference between the two would
read as breakage. **`--prefix URL` is the production URL of the build
directory's root**, refused outside every root's scope; a file is a page at
the prefix plus its relative path, and `foo/index.html` is `foo/` too (D22).
The deployer publishes the config beside the baseline, the guide's to say
(P6). Rejected: a CI-only config section or file, and flags alone.

## What the gate fails on

Settled 2026-10-20, answering what P4's first real `diff` left open
(`../design/roadmap-P4-reports-notes.md`, "What P5, CI mode, inherits").

- **The gate reads the links on built pages only**: every baseline link on
  a page not built is absent from the CI run, which a plain `diff` would
  call fixed.
- **Introduced** is `broken_url` or `broken_anchor` in the CI run where the
  same key in the baseline is anything but broken: absent, `ok`,
  `unchecked` or `error` (D9 read literally). A link the baseline never
  asked about can then fail CI, the cost accepted: it is new to what the
  gate knows.
- **Each introduced row names its state in the baseline**, so a new link, a
  link that was `ok` and one that was `unchecked` (robots, say) read apart.
- **`error` in the CI run never fails** (D16), and is listed.
- **Breakage already in the baseline does not fail**: it is listed after
  the rest as a count, printed in full on request.

### What goes live

Settled 2026-10-20. **A URL the baseline holds no target for goes live**, in
scope or external, at the config's pace: a URL new to the gate is the one
most worth asking, and the links new to it are a handful per PR.
`--externals=skip` (D22) remains for a deployer wanting no external
request. **There is no staleness rule**: the baseline's verdict stands
whatever its age, and the summary prints the baseline run's date. Rejected:
`unchecked`, reason `not_in_baseline`, for a new external URL, which passes
a new broken link; a `--max-age` refusal, failing CI for a reason the PR's
author cannot fix, until someone asks for one.

## The inverse check

Settled 2026-10-20 (D4, D9).

- **A page the change touches** is a baseline page in a component-version
  the build holds (above, "What a build holds"). Rejected: a baseline page
  whose `source_repo` a built page names, which matches nothing, a built
  page having no forge edit link (below, "Facts found while grilling").
- **Removed** is an anchor such a page had in the baseline and lacks in the
  build, or a page of such a repo the baseline holds and the build does not.
  A page-alias the build emits is built, a meta-refresh page at the old URL,
  so a rename with an alias removes no page; its anchors land on the new one.
- **Inverse breakage** is a baseline link from a page not in the build to a
  removed page or anchor, listed with its linking page and grouped by that
  page's repo or component: D9's "N inbound pages across M components".

## Exit status and output

Settled 2026-10-20 (D9, D16, D27).

- **Exit 0** when nothing is introduced; **1** on introduced breakage, or on
  inverse breakage while the inverse check fails; **2** when the tool could
  not run (a bad config, an unreadable baseline, a schema mismatch, a prefix
  out of scope). An `error` never moves the code.
- **Text to stdout, or `--format json`** as `report` has it, in the order:
  introduced breakage with each link's baseline state, inverse breakage,
  introduced redirected links, then counts of `error`s and of breakage
  already in the baseline.
- **`--inverse=fail|warn`, a flag**, defaulting to `fail`: the config is the
  deployment's, shared by every repo, and how strict one repo's CI is is
  that repo's job. D5 has no key for it to override.

### Introduced redirected links are listed, never failing

Settled at P3's grilling, 2026-10-17 (drained from the inbox): some users
treat every link through a redirect as a task, others care only about
breakage, and both are served **without configuration**. `check-dir`'s output
lists introduced redirected links apart from the breakage that fails the gate
(D7's non-failing `redirected`, D9).

## The command line and the manual

Settled 2026-10-20 (D27).

- **`bruce check-dir DIR --prefix URL --baseline PATH|URL`**, both
  required, with `--config f`, `--db f`, `--externals skip|baseline|live`
  (default `baseline`), `--inverse fail|warn` (default `fail`), `--format
  text|json`, and `--all` printing breakage already in the baseline in
  full.
- **`bruce export-baseline --out f [--run N] [--db f] [--force]`.**
- **`bruce report introduced` and `bruce report inverse`** take `--run` a
  CI run, reading the baseline from its `baseline_run`.
- **The manual gains `check-dir.md` and `export-baseline.md`**, and
  `report.md` the two kinds, each in its slice's change as P4's were. The
  deployment guide's CI section, the scheduled export, where it is
  published and a repo's CI job, is P6's.

## The end-to-end test and the evidence

Settled 2026-10-20 (D24).

- **The test site is hand-written, under `fixtures/antora/`**: three tiny
  components, one versioned, one `~`, one with two modules and a
  page-alias, holding xrefs among all three, fragments, and an anchor other
  components link to.
- **The Fedora UI bundle is vendored into `fixtures/` by
  `scripts/fixtures.py`**, its source commit and sha256 recorded: the
  playbook's URL is a moving job artifact, and the nav container, the edit
  link and the footer are the UI's.
- **`mise run e2e`**, Antora 3.1.15 from npm, builds the whole site, serves
  it from a local static server, crawls it, exports the baseline, builds one
  component alone with planted changes (a deleted page, a removed anchor
  others link to, a new broken link, a new broken xref out of the build, a
  valid one out of it), runs `check-dir`, and asserts the exit status and
  each section of the output. It is slow and needs node, so it is not in
  `mise run check`.
- **The closing evidence is a fresh en-US crawl under schema 4**, launched
  detached (CLAUDE.md, "Long-running processes") and exported: P4.10's DB
  is schema 3, which D43 refuses. Rejected: a one-off script rewriting it
  to schema 4, a migration in all but name before the release D43 waits
  for.
- **quick-docs is built with `npx antora`** from its `site.yml` and
  `--html-url-extension-style=indexify`, and checked twice: **as
  published**, the noise floor, every introduced row in it either real
  breakage elsewhere (above, "What a build holds") or a bug, the notes
  saying which; and **with the planted changes**, each rule firing on real
  content. A gate not silent on an unchanged build is red on every PR.

## Where the code lives

Settled 2026-10-20, on D39's layering, where `ci`, `crawl` and `report` sit
side by side and none calls another.

- **`ci`**: the walk of the build directory into seeds and the prefix
  mapping, a file to its production URL; which files are pages, and at
  what URL, being CI's concept rather than a fetcher's.
- **`fetch/local`**: the local fetcher (D1), reusing `fetch/charset`.
- **`crawl`** takes the seeds and the imported baseline run as inputs; its
  `frontier` child holds the baseline class.
- **`store`**, a new `baseline` child: export, import, and copying a
  target's rows from one run to another, `store` owning every write.
- **`profile/antora`**: Resolve, which `crawl` calls, being above both
  `profile` and `store`.
- **`report`**: the gate as two reports, `introduced` and `inverse`, so
  `bruce report introduced --run <ci>` works alone and the text and JSON
  rendering is shared.
- **`cli`**, a `check_dir` child and an `export_baseline` one: `check_dir`
  composes `ci`'s seeds, the import, the crawl and the two reports, and sets
  the exit status.

Rejected: the walk in `fetch/local`, deleting `ci`; `ci` driving the crawl,
an edge sideways in D39's tier.

## Facts found while grilling

- **One en-US run is a 28.8 MB DuckDB file**: 6433 pages, 588551 links,
  93521 anchors, 172452 targets, 8938 `page_facts` rows
  (`runs/p4.10-en-us-20261019-2125/en-us.duckdb`).
- **Resolve is declared and does nothing** (`src/profile/antora.rs`,
  `Profile::resolve`), and the store's `Lookup` has one method,
  `pages_with_facts` (`src/profile.rs`).
- **The baseline's Antora facts cover the pages with an edit link**: of
  P4.10's 2320 `antora` pages, 2320 carry `antora.component`, 2314
  `antora.version`, and 2152 `antora.module` and `antora.rel_path`, the
  same 2152 that have a `source_repo`. Its 92 `xref_refspec` links (72
  distinct) target the linking page itself, the refspec as the fragment.
- **A `Fetcher` answers one hop** with a status, a location and a body
  (`src/fetch.rs`), and the baseline holds no bodies (D15), so it cannot
  stand in as a fetcher for a page; `link_verdicts` joins a link's target,
  landing target, `pages` row and `anchors` within one run
  (`src/store/views.rs`).
- **A local build's pages carry no forge edit link.** Antora derives an
  edit URL only for github.com, gitlab.com, bitbucket.org and pagure.io
  remotes (`content-aggregator/lib/compute-origin.js:8`, v3.1.15), and only
  production sets `edit_url` (`docs-fp-o/build-scripts/rebuild-site.py:167`).
  A worktree source gets `page.fileUri`, which the Fedora UI links while
  `CI` is unset (`ui-bundle src/partials/edit-this-page.hbs:1`), so
  docsbuilder's page links `file:///antora/modules/ROOT/pages/<page>.adoc`;
  with `CI` set, as Forgejo Actions sets it, there is no edit link; a
  remote source in the playbook gets none either way. The nav container,
  and so `antora.component` and `antora.version`, renders regardless.
- **A partial build has no `site.url`** (docs-template's and quick-docs'
  `site.yml`), so no canonical `<link>`, `sitemap.xml`, `robots.txt` or
  `404.html` (`site-generator/lib/generate-site.js:42,47`); content, asset,
  nav and start-page hrefs are relative. The UI's footer writes
  root-relative `{{site.path}}/legal/privacy/`, `/legal/` and
  `/project/code-of-conduct/` (`footer-content.hbs:19,26,33`), `/legal/…`
  locally where production has `/en-US/legal/…`.
- **Layout under indexify**: `public/<component>/<version>/<module>/<page>/index.html`,
  `ROOT` omitted, and no version segment for `~` or `master`
  (`content-catalog.js:502-576`); UI assets under `public/_/`. A page-alias
  is a page at the alias path whose meta refresh (delay 0) and script name
  the target relatively (`redirect-producer/lib/produce-redirects.js:131`).
  `public/index.html` is a redirect to the start page, standing at the
  prefix's own URL, which in production is the site's home page.
- **An unresolved xref logs at error and does not fail the build**:
  `failure_level` defaults to `fatal` and docsbuilder passes none
  (`playbook-builder/lib/config/schema.js:260`).
- **quick-docs' playbook** builds its worktree plus `docs/team-docs`
  (`fedora-docs`), `server/user-documentation` (`fedora-server`) and
  `operations/docs` (`operations`), all versionless, on `main`, and
  `quick-docs` itself is `version: master`. Production pulls a second
  start path of `operations/docs`, `releases`, which is its own component
  (`releases/antora.yml`), so quick-docs builds whole components. Its
  modules xref `package-maintainers::` (5), `atomic-desktops::` (2),
  `legal::`, `iot::`, `fedora:release-notes:` and
  `program_management::changes_policy.adoc`, all outside the build.
- The docs-fp-o image carries Antora 3.1.12, production 3.1.15; the files
  above match between them. Sources read: Antora v3.1.15, docs-template
  `107905a`, ui-bundle `7f5fb3d`, quick-docs `70cd18e`, docs-fp-o `prod`.
- **This machine has node and npx, not podman**, so the end-to-end test and
  the evidence build run Antora from npm, not the docs-fp-o image.

## The slices

In order; each makes the next one's mistakes visible. Nothing builds or
tests while a crawl runs (`CLAUDE.md`, "Long-running processes"), so the
slices after P5.2's launch land in later sessions or wait for it.

### P5.1 The Antora fixture site

`fixtures/antora/`, the vendored UI bundle, and `mise run e2e` building the
whole site and one component alone, with a few partial-build pages saved as
fixtures; no `bruce` assertion yet. Every later slice tests against what a
partial build writes rather than what one is remembered to write.

### P5.2 Schema 4, and the fresh en-US crawl

The `targets` column naming where an answer came from, a crawl writing
`live`; then the en-US crawl launched detached, ready for P5.11.

### P5.3 `export-baseline`

The store's export, the command and its manual page, and the gzip
measurement.

### P5.4 The baseline's import

The import and its reuse, and the store's copy of a target's rows from one
run to another, tested before any crawl path calls them.

### P5.5 The local build

`ci`'s walk and prefix mapping, root-relative hrefs against the prefix,
`fetch/local`, and `crawl` taking seeds: `check-dir` runs with no baseline.
New modules only; the frontier is untouched.

### P5.6 The baseline class

The frontier's baseline class, the held component-versions and
`not_built`: `check-dir` runs against a baseline. A rework of a tested core
path, kept apart from P5.5.

### P5.7 Resolve

`antora`'s Resolve, and `crawl` calling it in a CI run, on P5.6's held set.

### P5.8 The gate

`report introduced`, the redirected and already-broken sections, the exit
status, `check-dir`'s output and its manual page.

### P5.9 The inverse check

`report inverse` and `--inverse`, on P5.6's touched pages.

### P5.10 The end-to-end test

`mise run e2e`'s planted changes and every assertion.

### P5.11 quick-docs against the en-US baseline

quick-docs as published and with the planted changes, against P5.2's
crawl exported.
