# Link checker: initial design plan

**Frozen.** This is the bootstrap plan from the first planning session
(2026-10-17). It is never edited again (`../../process.md`, "Naming and
lifecycle"). Its decisions now live in `../decisions.md` and its external facts
in `../site-invariants.md`. The milestones became `../roadmap.md`'s phase index.
Where this file and those disagree, they win. The decision numbers here match
the register's, except D25, which was superseded, and D29, which moved to the
roadmap.

## Requirements (from the initial brief)

- Primary target: docs.fedoraproject.org, which Antora assembles from dozens of
  repos. Cross-repo anchor links often break.
- Validate anchors: a `url#fragment` link counts as broken if the page loads
  but the fragment isn't there.
- Per page, keep two separate stats: **broken outbound URLs** and **broken
  outbound anchors** (URL fine, fragment missing).
- Allowlist the external domains to check (other Fedora web properties, for
  example).
- Local DuckDB database holding every page checked, when it was checked, and
  what it links to. Uses: broken-link reports, rebuilding the full sitemap,
  ranking pages by inbound links.
- Rust, as fast as possible, with a configurable rate limit.

## Facts gathered

Gathered 2026-10-17. The sub-agent's working clones (source repos, lychee) are
in the session scratchpad and aren't kept.

### docs.fedoraproject.org

**Sitemap**
- There is no root `/sitemap.xml` (404). The sitemap index is at
  `/en-US/sitemap.xml`, and lists 68 per-component sitemaps
  (`/en-US/sitemap-<component>.xml`) with `<loc>` and `<lastmod>`.
- Sample counts: quick-docs 294, infra 265, fedora 479, packaging-guidelines
  82, fedora-coreos 78.
- `/fr/sitemap.xml` and `/de/sitemap.xml` exist but have the same content as the
  en-US index. **Translated pages are in no sitemap.**

**robots.txt**
- `Crawl-delay: 1`
- `Disallow: /*/Fedora*/`
- `Disallow: /*/Community_Services_Infrastructure/`

**URL layout**
- `/{locale}/{component}/{version?}/{module-path}/{page}/`, in the indexify
  style: every page is a directory with an `index.html`.
- The root `/` is a meta-refresh to `./en-US/`.

**Versions**
- Only the `fedora` component is versioned (f26–f43, `latest` = f44,
  `rawhide`). Every other component is unversioned.
- `/fedora/latest/` is the real copy. `/fedora/f44/…` gives a **302 to
  `/fedora/latest/…`**, via a generated `.htaccess`.
- Older versions are full copies, with `rel=canonical` pointing at `latest`.

**Locales**
- There are 67 locales (mixed naming, e.g. `es_419`, `zh_Hans`, `pt_BR`). Each
  appears to be a full copy of the site with English fallback, though that's
  unverified.

**Page count**
- en-US is ~1,838 pages (latest versions only), or **~2,300 including older
  fedora versions**.
- Across all locales the extrapolation is ~150k pages.

**Anchors in the HTML**
- Section headings: `<h2 id="_auto_id">` (auto-generated, `_` prefix) or
  `<h2 id="explicit">`.
- Inline `[[id]]` renders as an empty `<a id="id"></a>`.
- Block anchors are `<div id=…>`.
- No `name=` attributes are used on content.
- **Chrome ids appear on every page**: `search-field`, `preamble`,
  `search-input`, `themeSwitch`, `site-script`, `search-ui-script`, plus an
  empty `id=""`.
- `@asciidoctor/tabs` ids (`_tabs_1`, `_tabs_1_tab_a`,
  `_tabs_1_tab_a--panel`) are static in the HTML; JS only toggles visibility.
- Not audited: whether the UI bundle's JS injects ids.

**Links in the HTML**
- Antora xrefs, including cross-component ones, render as **relative** hrefs
  (`../legal/license-approval/`, `../Naming/#_frag`) with `class="xref page"`.
- Hand-written URLs stay absolute.
- Navigation and version-picker links are relative.

**HTTP behaviour**
- Server: Apache behind Fedora proxies. No CDN and no rate-limit headers.
- Sends `cache-control: max-age=1800`, plus ETag and Last-Modified, but they
  **differ between backend proxies**. `If-None-Match` with a fresh ETag
  returned 200, so **conditional revalidation is unreliable**.
- HEAD works, and so does gzip.
- `http://` gives 302 to https.
- A missing trailing slash gives a **301 to `http://…/` (scheme downgrade),
  then a 302 to https**: two hops.
- A 404 has a real 404 status.

**Sizes**
- Pages: 33–216 KB raw (packaging-guidelines index is 54 KB gzipped).
- `/en-US/search-index.js` is 19.9 MB (a lunr index; it lists pages, not
  anchors).

**Sources**
- The playbook is `site.yml` in https://forge.fedoraproject.org/docs/docs-fp-o
  (Forgejo; branch `prod`, with `stg` for staging). It moved there from GitLab.
- There are 63 unique source repos (59 on forge.fedoraproject.org, 7 on GitHub,
  4 on GitLab), giving 68 components.
- Translations come from `forge.fedoraproject.org/docs/translated-sources`.
- The site is built by `build-scripts/rebuild-site.py` in OpenShift.
- **No built-site tarball is published.**

### Forge, CI and Antora behaviour

**Forge**
- **Forgejo at forge.fedoraproject.org** (org `docs`). The GitLab content
  projects are archived "[MOVED]", and Pagure is gone.
- Still on GitLab: the UI bundle (`gitlab.com/fedora/docs/docs-website/ui-bundle`,
  job artifact zip) and the build image
  `registry.gitlab.com/fedora/docs/docs-website/docs-fp-o:latest`. That image
  runs Antora 3.1 with lunr and `@asciidoctor/tabs`.
- Migration tracker: https://forge.fedoraproject.org/docs/tickets/issues/35

**Production build**
- An OpenShift CronJob runs **hourly** (`50 * * * *`) via
  `docs-fp-o/build-scripts/rebuild-site.py` (`antora --extension=lunr site.yml`).
- Prod `site.yml` sets `runtime.log.level: fatal`, so **unresolved-xref errors
  are suppressed in production**.

**Per-repo CI**
- The Docs team's own repos (quick-docs, docs-template, release-notes) have
  **no CI**. docs-fp-o has only an hourly timestamp-monitor workflow.
- Some other teams' repos run Forgejo Actions on PRs that just run Antora:
  - infra/docs, security/docs, forge/documentation (`--log-failure-level=warn`);
  - apps/badges-docs, ci/docs, mindshare/docs (`fatal`);
  - ci/docs uploads `public/` as an artifact;
  - server/user-documentation also runs `vale`.
- GitLab-hosted repos (websites-apps/documentation, robotics) run Antora in MR
  CI, with a `public/` preview.
- **None of them check links beyond Antora's own log.**

**Partial builds**
- Single-repo builds follow docs-template's `docsbuilder.sh`: podman with the
  docs-fp-o image, `--html-url-extension-style=indexify site.yml`, output to
  `./public`.
- A partial playbook is **not always one component**. quick-docs adds
  team-docs, server/user-documentation and operations/docs. infra/docs adds
  operations/docs and epel/docs.

**Unresolved xrefs** (Antora 3.1, `convert-resource-ref.js` / `html5.js`)
- When the target page isn't in the catalog, the link renders as
  `<a href="#<raw refspec>" class="xref unresolved">`, e.g.
  `href="#program_management::changes_policy.adoc"`. Any fragment is kept
  (`#comp::page.adoc#anchor`).
- The href is the author's literal resource ID, **not normalised**, so the
  checker must parse Antora resource-ID syntax (`version@component:module:page`,
  where `::` means the ROOT module).
- Live prod pages already contain them. Examples on
  `/en-US/quick-docs/bugzilla-file-a-bug/`, `…/bugzilla-providing-a-stacktrace/`
  and `…/contribute-to-quick-docs/`.
- **A naive checker sees these as same-page fragments.**

**Antora and fragments**
- Antora **never validates fragments**; it checks only that the page exists.
  See Antora issues #1077 and #704. Fragment validation has to be external.

**Catalogs**
- Antora core publishes no page or anchor manifest.
- `@antora/atlas-extension` (alpha) emits `site-manifest.json`, which lists
  pages and resolves partial-build cross-component refs to a primary site. It
  has **no anchor lists**. docs.fp.o doesn't use it (404).
- `/en-US/search-index.js` (lunr) has 1,840 documents with component, version,
  name, url and section-heading `hash` ids. It has no block or inline anchors,
  is latest-only, and isn't a stable API.
- Untested idea: a small Antora extension could dump resource ID → URL plus
  per-document ids. `runtime.log.format: json` gives unresolved xrefs with
  file and line.

### Existing tools

**lychee** (Rust, v0.24.2, usable as a library via `lychee-lib`)
- Has `--include-fragments`: local files since v0.14 (2024), remote pages since
  v0.19 (2025-06).
- Doesn't crawl or recurse, and doesn't persist a link graph (its cache holds
  status only).
- Only checks fragments on GET.
- Collects **every** `id` and `name` attribute, including `<meta name>`, so
  `#viewport` and `#generator` would wrongly pass on Fedora pages.
- Has per-host throttling built on governor.

**Other checkers**
- htmltest: local files only, internal fragments only.
- LinkChecker (Python): crawls, has an AnchorCheck plugin, no incremental state.
- muffet (Go): crawls and checks fragments, no persistence.
- hyperlink (Rust): local files only, very fast, opt-in anchor checking.

The gap this tool fills is **crawl + fragment check + persistent link graph +
run history + CI baseline diff**. Nothing found does all of these.

### Rust crates (2026-10)
- `duckdb` 1.10506.1: Appender API, plus `appender-arrow`. The `bundled`
  feature builds from source; the default downloads a prebuilt libduckdb.
- HTML parsing: lol_html 3.0.1 (streaming), html5gum 0.8.4 (tokenizer, used by
  lychee and hyperlink), html5ever 0.40.1, scraper 0.27 (full DOM, likely
  slowest), tl 0.7.8 (unmaintained since 2024-01). No benchmarks were run.
- HTTP: reqwest 0.13.5 and hyper 1.12.
- Rate limiting: governor 0.10.4 (GCRA, keyed per-host).

## Decisions

### D1: Input mode: live crawl first, local build directory later
- Fetching sits behind a trait, with two implementations: an HTTP crawler
  (built first) and a reader for a local Antora output directory (built later).
  Parsing and the DB layer are shared between them.
- Why: a full site build takes hours, so nobody runs one locally. The
  local-directory mode is mainly for **CI verification**, i.e. checking a docs
  change before deployment.

### D2: Run history: append-only runs
- Every run gets a `run_id`. Pages, links and check results are recorded per
  run, never overwritten. That supports "when did this break", run-to-run diffs,
  and linking a breakage to an upstream change. Old runs can be pruned.

### D3: External links
- **Allowlisted domains**: check the target URL and its fragment (which needs a
  full GET of the HTML). **Never crawl outward** from those pages.
- **Non-allowlisted domains**: record the link in the DB as `unchecked`, with no
  network request. It stays in the link graph.

### D4: CI mode resolves out-of-build targets via the DB, then live
- A nightly (or scheduled) full crawl produces a DB that is published for CI.
- In CI, links to pages outside the local partial build are resolved against
  that DB, which stores **each page's anchor set** (a requirement). If the
  entry is missing or stale, fall back to fetching live.
- **Inverse check**: CI also uses the DB's link graph to find pages in *other*
  repos that link to anchors this change removes or renames.

### D5: Configuration
- TOML config file, with CLI flags overriding it.
- `[[root]]` entries: sites crawled fully (docs.fedoraproject.org), each with
  include/exclude URL globs (e.g. to skip old versions or locales).
- `allowlist`: external domains checked one hop deep (D3). It's separate from
  the roots.
- `[rate]` table for rate limiting.
- The roots are configurable so the tool isn't tied to Fedora.

### D6: Output, reports and exit status
- Named reports are CLI subcommands (`report broken`, `report inbound --top N`,
  `report sitemap`, …) backed by **SQL views stored in the DB**, so ad hoc
  `duckdb` users see the same views.
- Output is text by default, with `--format json`. CI annotations depend on the
  forge (being looked up).
- The exit code is configurable. In CI it fails only on breakage **introduced
  by the change**, not on pre-existing breakage, which would keep CI
  permanently red.

### D7: URL identity and redirects
- Canonical URL key:
  - lowercase the scheme and host;
  - drop default ports;
  - percent-decode where safe;
  - store the **fragment separately**;
  - keep the query string, with an optional per-root setting to strip it;
  - treat `/foo` and `/foo/` as distinct.
- Follow up to **10 redirect hops** and record the chain. A loop or too many
  hops counts as a broken URL.
- Check the fragment against the **final** page.
- "Link goes through a redirect" is its own **non-failing** stat. It catches,
  for example, links that pass through `/fedora/f44/` → `/latest/`, and
  slashless links that take a 2-hop scheme downgrade.

### D8: Rate limiting and concurrency
- Per-host rate limit and per-host concurrency cap. Defaults live in config,
  with per-host overrides. Root sites can be configured more aggressively than
  allowlisted third parties.
- Automatic backoff on 429/503, honouring `Retry-After`.
- robots.txt (Disallow, Crawl-delay) is honoured for external hosts. For root
  sites it's a **per-root switch**. docs.fp.o says `Crawl-delay: 1`, which would
  make an en-US crawl ~40 min.
- Every run refetches every page. Conditional revalidation isn't used because
  docs.fp.o's validators are inconsistent across proxies (see Facts).

### D9: "Introduced by the change" in CI
- A link's identity is `(source page URL, target URL, fragment)`, with a
  configured mapping from the local build dir to its live URL prefix.
- A broken link in CI counts as **introduced** if that exact link isn't broken
  in the baseline DB. That covers new links, links that were previously OK, and
  every link on a new page.
- Renamed pages make all their links look new. That noise is accepted.
- **Inverse check** (D4) **fails by default**, with a clear message (anchor
  removed, N inbound pages across M components). It can be configured down to a
  warning.

### D10: Default crawl scope
- en-US, **all fedora versions** (frozen f26–f43 included; ~470 pages, and
  their anchors are needed because current pages link to them).
- Locales are a configured root that's **off by default**. Translated
  auto-generated heading ids can break fragments, but the translation team
  owns that.

### D11: Own stack, not lychee-lib
- reqwest + governor + a streaming HTML tokenizer + duckdb.
- **Early spike**: benchmark html5gum against lol_html on a few hundred saved
  docs.fp.o pages, then pick one.
- lychee doesn't model the parts that are ours: the crawl frontier, the link
  graph and anchor semantics. It would also have to be worked around, e.g. its
  `<meta name>` false positives.

### D12: Fragment validity
- **Valid targets**: any non-empty `id` on any element, plus `name` on `<a>`
  only.
- `#` (empty) and `#top` are always valid.
- Matching is exact and case-sensitive, after percent-decoding.
- For each anchor, store its element tag and a **content vs chrome** flag
  (inside `<article class="doc">` or not), so reports can tell a heading, an
  inline anchor, a tab and a template id apart. Links to chrome ids pass but
  are reportable.
- Text fragments (`#:~:text=…`) aren't checked and are recorded as unchecked.

### D13: Extracted references
- Extract `<a href>`, `<img src>`/`srcset`, `<link href>` (stylesheet,
  canonical, alternate…), `<script src>`, `<iframe src>`, `<source>` and
  similar. Each link carries a **`kind`** column.
- Each unique target URL is fetched once per run, however many pages reference
  it.
- Assets get a HEAD request with GET fallback; they never need a fragment
  check.
- Reports default to `kind = anchor`. `rel=canonical` is kept, since old
  versions point theirs at `latest`.
- `mailto:`, `tel:`, `javascript:` and `data:` are recorded as unchecked.

### D14: Page discovery
- Seed from a **per-root sitemap URL** (docs.fp.o: `/en-US/sitemap.xml`, which
  is an index) *and* from root URLs, then follow links within the roots.
- Each page records how it was found: sitemap, links, or both.
- That gives two reports: **orphans** (in the sitemap, with no inbound internal
  link) and **unlisted** pages (reachable by links, missing from the sitemap).

### D15: Page data stored
- URL, status, final URL, content type, size, fetch time, duration.
- **Content hash**, used for change detection, since ETag is unreliable.
- `<title>`, plus Antora **component/version/module**, so reports can group by
  component (that is, by source repo).
- HTML bodies are **not** stored by default. An opt-in flag writes them to a
  separate, prunable table.

### D16: Result classification
- Outcomes:
  - `ok`;
  - `broken_url`: 404/410 and other 4xx (except below), NXDOMAIN, redirect
    loop or too many hops;
  - `broken_anchor`: page OK, fragment missing;
  - `error`: timeout, connection reset, TLS failure, 5xx after retries, and
    **401/403/429**, with the status recorded;
  - `unchecked`.
- `redirected` is a non-failing modifier on the outcomes above.
- Retry 3 times with exponential backoff before settling on `error`.
- **`error` alone never fails CI.** It's reported separately from the broken
  stats.

### D17: Page source mapping
- Extract and store each page's `source_url` (the "Edit this Page" link) and
  `source_repo`.
- Uses: routing broken links to the right repo and file, mapping CI builds to
  repos, and grouping reports by source repo (more exact than component).
- Markup confirmed (Antora 3.1.15, `<meta name="generator">`):
  - edit link:
    `<a href="https://forge.fedoraproject.org/<org>/<repo>/src/branch/<branch>/<start_path>/modules/<module>/pages/<path>.adoc" title="Edit this Page">`;
  - `<div class="nav-container" data-component="fedora" data-version="f44">`;
  - content lives in `<article class="doc">`.
- From the edit link we get the **module** and **page path** without
  guessing. From the nav-container we get the component and version.
  Unversioned components report a placeholder version, e.g. `data-version="master"`
  for packaging-guidelines, while their URL has no version segment.

### D18: Default politeness for docs.fedoraproject.org
- The shipped config obeys `Crawl-delay: 1`. That's ~40 min for an en-US
  crawl, which is fine for the nightly baseline.
- Going faster is an explicit per-root override for whoever has permission.
- "Maximum performance" targets the local-dir mode and the processing pipeline.

### D19: Antora resource IDs
- Each page stores `(component, version, module, relative path)`, taken from
  the nav-container and the edit link (D17).
- In CI, `xref unresolved` hrefs (`#<refspec>`) are parsed with Antora's rules,
  relative to the containing page's own component, version and module:
  - `page.adoc` = same module;
  - `mod:page.adoc` = other module, same component;
  - `comp::page.adoc` = ROOT module of comp;
  - `ver@comp:mod:page.adoc` = explicit version.
- The parsed ID is looked up in the baseline DB.
- With no version given, use the component's latest/only version in the DB
  (unversioned components show placeholder versions such as `master`).
- A live crawl uses the same index to report which source xref broke.

### D20: Classifying unresolved xrefs
- **Live crawl**: `broken_url` with reason `unresolved_xref`, keeping the raw
  refspec. It's never treated as a same-page fragment.
- **CI**: resolve against the baseline DB. If it resolves, check the page and
  then the fragment as normal. Otherwise `broken_url`.
- **"Did you mean" hint**, run only on failures: a fuzzy match over
  component/module/page names, catching cases like `packaging-guidelines:Debuginfo.adoc`
  where `::` was meant.
- Ordinary same-page `href="#foo"` links are checked against the page's own
  anchors.

### D21: CI baseline artifact
- An `export-baseline` command writes **one run** (pages, resource IDs,
  anchors, links) to a compact **single-run DuckDB file**, estimated at a few
  MB.
- CI takes `--baseline <path-or-https-url>`.
- The nightly crawl is meant to run as a **scheduled Forgejo Action** that
  publishes the baseline as a generic package or release asset.
- Hosting is deployment config, not code, and the exact location is still
  open.

### D22: Local-dir (CI) link resolution
- A configured prefix maps the build dir to production URLs
  (`public/` → `https://docs.fedoraproject.org/en-US/`).
- For each link:
  1. resolve relative hrefs against the page URL;
  2. under the prefix and present locally (`foo/` → `foo/index.html`) →
     check locally;
  3. under the prefix but not built → **baseline** lookup, with a live fetch
     if it's missing or stale;
  4. `xref unresolved` → resource-ID lookup (D19), then as above;
  5. outside the prefix → allowlist rules.
- `--externals=skip|baseline|live`, **default `baseline`**: reuse the
  nightly's verdict for external URLs, so CI stays fast and deterministic.

### D23: Toolchain and distribution
- `duckdb` crate with the **`bundled`** feature, for a single self-contained
  binary.
- Stable Rust pinned in `mise.toml`, with dev tasks as mise tasks.
- Distributed as a **release binary on the forge**, fetched by CI jobs, which
  run in the Fedora-minimal docs-fp-o image. A container image or RPM may come
  later.

### D24: Testing
- Unit tests on **saved real docs.fp.o fixture pages** (ReviewGuidelines, the
  tabs page, an unresolved-xref page): extraction, anchors, resource-ID
  parsing.
- HTTP integration tests against a **wiremock** server: redirects incl. the
  scheme-downgrade chain, 429 + Retry-After, robots.txt.
- **End-to-end**: a tiny 2–3-component Antora site built in a container,
  exercising CI mode against a baseline. It's slow, so it sits behind a
  feature flag or mise task.
- **criterion** bench comparing html5gum and lol_html (D11).

### D25: Name: `anchorage`, superseded by D30
- Dropped because **`anchorage` is taken on crates.io** by an unrelated
  Lavalink wrapper (v0.1.1, created 2025-09). That only matters if we publish
  to crates.io.
- Free on crates.io as of 2026-10-17: `fragcheck`, `anchorcheck`,
  `anchor-check`, `docs-anchors`, `linkgraph`, `fraglink`.

### D26: DB schema
- Every URL gets a stable **`url_id` = u64 hash of the canonical URL**, so
  cross-run joins are cheap.

```
runs     (run_id, started_at, finished_at, mode[crawl|local], tool_version, config_hash, baseline_run?)
targets  (run_id, url_id, url, status, final_url, redirect_chain[], outcome, reason, checked_at, duration_ms)
           -- one row per unique URL fetched/resolved this run (internal + external)
pages    (run_id, url_id, title, content_type, size, content_hash,
          component, version, module, rel_path, source_url, source_repo,
          in_sitemap, via_links)
           -- HTML pages inside roots only
anchors  (run_id, url_id, anchor, tag, in_content)
links    (run_id, src_url_id, target_url_id, fragment, raw_href, kind, xref_refspec?,
          outcome, reason, redirected)
           -- one row per occurrence (not deduplicated); outcome = combined URL+fragment verdict
bodies   (run_id, url_id, html)  -- opt-in (D15)
```

- Views stored in the DB:
  - `page_stats`: per page, counts of broken_url, broken_anchor, error,
    redirected and unchecked;
  - `broken_links`;
  - `inbound`: distinct source pages per target;
  - `sitemap`;
  - `orphans`;
  - `unlisted`;
  - `anchors_removed(run_a, run_b)`, with inbound links, which drives the D4
    inverse check;
  - `run_diff`.

### D27: CLI surface

```
bruce crawl          [--config f] [--db f]
bruce check-dir DIR  --prefix URL --baseline PATH|URL [--externals skip|baseline|live]
bruce report KIND    [--run N] [--component C] [--repo R] [--format text|json]
   KIND = broken | stats | inbound | sitemap | orphans | unlisted | redirects | anchors-removed | diff
bruce export-baseline --out f [--run N]
bruce prune          --keep N | --older-than DUR
```

- `check-dir` also writes its results to a local DB, so reports work on CI
  results. Its exit code follows D9 and D16.

### D28: Pipeline architecture
- **tokio** runtime, with a URL frontier deduplicated by `url_id`.
- **Fetch tasks** gated per host by a governor rate limiter plus a semaphore
  (D8).
- **Parsing** on a rayon / `spawn_blocking` pool: a single streaming pass over
  the bytes extracts hrefs, ids, title, nav-container and edit link. No DOM.
- **One DB writer task**, fed by a bounded channel (for backpressure), batches
  rows into DuckDB **Appenders** and commits every N rows.
- **Fragment verdicts are computed after the crawl, in SQL** (join `links` to
  `anchors`). The crawler holds no anchor state, and the same join feeds the
  reports.
- Local-dir mode reuses the parser and writer, with a parallel (rayon)
  directory walk instead of HTTP.

### D29: Milestones
1. Scaffold: Cargo, mise, config loader, DB schema and views, saved fixture
   pages, parser bench spike.
2. Extractor: links, anchors, page metadata and resource IDs, with fixture
   tests.
3. Crawler: frontier, sitemap seeding, rate limits, redirects, retries, robots.
   Wiremock tests.
4. Post-crawl SQL verdicts and `report` subcommands. **First real en-US
   crawl.** This milestone already delivers the sitemap, inbound ranking and
   both broken stats.
5. `export-baseline` and `check-dir`: local and resource-ID resolution, inverse
   check, exit codes. Antora e2e test.
6. Release binary, and a scheduled Forgejo Action for the nightly crawl.

### D30: Name: `bruce`
- **The binary and crate are `bruce`**, after the Bruce claw anchor (Peter
  Bruce, 1970s, derived from North Sea oil-rig moorings; known for setting fast
  and resetting when the boat swings). It was free on crates.io on 2026-10-17.
- Candidates considered, with crates.io availability as of 2026-10-17:
  - free: `marley`, `aweigh`, `kedger`, `windlass`, `deadlight`, `ahab`,
    `bitter-end`;
  - taken: `kedge`, `fluke`, `grapnel`, `holdfast`, `fathom`, `capstan`,
    `hawser`, `mooring`, `ozymandias`, `ariadne`.
- Trade and role terms:
  - free: `anchorsmith`, `boatswain`, `anchorwatch`/`anchor-watch`,
    `provinghouse`/`proving-house`, `cathead`;
  - taken: `bosun`, `forecastle`.
- Fleetwood Mac, "The Chain" (*Rumours*, 1977): "never break the chain". Free:
  `neverbreak`, `never-break`, `thechain`, `breakthechain`, `rumours`.
- Anchor types:
  - free: `bruce`, `danforth`, `rocna`, `mantus`, `herreshoff`, `northill`,
    `stockless`, `admiralty`, `stevin`, `plough`;
  - taken: `cqr`, `spade`, `fortress`, `claw`, `vulcan`.

## Open questions

- **Baseline hosting location** (D21): which forge repo or package registry
  the nightly job publishes to. This is deployment config only and doesn't
  block implementation.
- **Forge repo** this project will live in.
