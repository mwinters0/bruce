# Site invariants

Properties of the sites we check, and of the tools that build them, that a decision treats as
guaranteed. Rules: `../process.md`, "The assumptions register". Each `I<n>` is allocated on
discovery and never reused. Walk this file whenever Antora's version changes (I8's generator tag
names it), or when docs.fp.o's build or hosting moves: a change here surfaces as wrong verdicts,
not as an error.

<!-- invariant-watermark: I25 -->

### I1 docs.fp.o's sitemap is an index at `/en-US/sitemap.xml`, and lists no translated page
- **Claim.** `/sitemap.xml` is 404, and robots.txt names no `Sitemap:`. `/en-US/sitemap.xml` is
  a `<sitemapindex>` of per-component `sitemap-<component>.xml` files (68 on 2026-10-17), each with
  `<loc>` and `<lastmod>`. Locale sitemaps repeat the en-US index.
- **Proof.** Upstream source: `@antora/site-mapper` 3.1.15 `lib/map-site.js` writes no sitemap without
  `site.url`, stamps every `<lastmod>` with the build time, and publishes the index at the output root
  alone, production's being `./public/en-US` (docs-fp-o `site.yml`). The rest is observed behaviour.
- **Scope limit.** Says nothing about pages reachable only by links. A root narrower than `/en-US/`
  (`/en-US/quick-docs/`) finds no sitemap by D40's probes, robots.txt naming none and `<root>/sitemap.xml`
  and `/sitemap.xml` being 404; it sets `sitemap` to `/en-US/sitemap-<component>.xml`. `<lastmod>` is the
  build's time on every entry, not the page's.
- **Verified against.** docs.fp.o, 2026-10-17; the probes and Antora 3.1.15, 2026-10-21.
- **Relied on by.** D14, D10, D40.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/sitemap.xml | grep -c '<loc>'; for u in sitemap.xml en-US/quick-docs/sitemap.xml; do curl -s -o /dev/null -w "%{http_code} $u\n" https://docs.fedoraproject.org/$u; done; curl -s https://docs.fedoraproject.org/robots.txt | grep -ci '^sitemap:'`

### I2 docs.fp.o's robots.txt asks for `Crawl-delay: 1`
- **Claim.** `User-agent: *`, `Crawl-delay: 1`, `Disallow: /*/Fedora*/`,
  `Disallow: /*/Community_Services_Infrastructure/`.
- **Proof.** Observed behaviour.
- **Scope limit.** Only docs.fp.o. Other roots and allowlisted hosts are read at run time.
- **Verified against.** docs.fp.o, 2026-10-17.
- **Relied on by.** D8, D18.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/robots.txt`

### I3 docs.fp.o's validators differ between backend proxies
- **Claim.** `ETag` and `Last-Modified` for one URL vary with the proxy that answers, and
  `If-None-Match` with a just-received ETag can return 200. Conditional revalidation is unreliable.
- **Proof.** Observed behaviour. It may only reflect a build that finished minutes earlier.
- **Scope limit.** docs.fp.o behind Fedora's proxies (`x-fedora-proxyserver`).
- **Verified against.** docs.fp.o, 2026-10-17.
- **Relied on by.** D8, D15.
- **Re-verify.** `for i in 1 2 3 4; do curl -sI https://docs.fedoraproject.org/en-US/quick-docs/ | grep -iE '^(etag|last-modified|x-fedora-proxyserver)'; done`

### I4 A slashless docs.fp.o URL redirects through `http://`
- **Claim.** `/en-US/quick-docs` gives a 301 to `http://…/quick-docs/`, then a 302 to https: two
  hops. `http://` gives a 302 to https everywhere.
- **Proof.** Observed behaviour.
- **Scope limit.** Antora's own xrefs always carry the slash (I10). This affects hand-written links.
- **Verified against.** docs.fp.o, 2026-10-17.
- **Relied on by.** D7.
- **Re-verify.** `curl -sIL -o /dev/null -w '%{num_redirects} %{url_effective}\n' https://docs.fedoraproject.org/en-US/quick-docs`

### I5 The latest fedora version is served at `latest`, and its own number redirects there
- **Claim.** `/en-US/fedora/latest/` is the real copy. `/en-US/fedora/f44/…` gives a 302 to
  `/fedora/latest/…`, via a generated `.htaccess`. Older versions are full copies whose
  `rel=canonical` points at `latest`. Only the `fedora` component is versioned.
- **Proof.** Observed behaviour, plus `docs-fp-o/build-scripts/rebuild-site.py` (upstream source).
- **Scope limit.** The number moves at each Fedora release.
- **Verified against.** docs.fp.o, 2026-10-17 (latest = f44).
- **Relied on by.** D7, D10, D55 (`profile::antora::resolve::newest`).
- **Re-verify.** `curl -sI https://docs.fedoraproject.org/en-US/fedora/f44/ | grep -iE '^(HTTP|location)'`

### I6 Antora renders an xref it cannot resolve as `href="#<raw refspec>" class="xref unresolved"`
- **Claim.** When the target page is not in the content catalog, the href is `#` plus the author's
  literal refspec, fragment kept (`#comp::page.adoc#frag`), with role `xref unresolved`, and an
  error is logged. Production pages carry these because prod logs at `fatal`.
- **Proof.** Upstream source, Antora v3.1.x: `packages/asciidoc-loader/lib/xref/convert-resource-ref.js`
  and `lib/converter/html5.js`. Observed live on `/en-US/quick-docs/bugzilla-file-a-bug/`.
- **Scope limit.** Antora 3.1. Not diffed against 3.2 or main.
- **Verified against.** Antora 3.1.15 output, 2026-10-17.
- **Relied on by.** D19, D20, D22.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/quick-docs/bugzilla-file-a-bug/ | grep -o '<a href="#[^"]*" class="xref unresolved">'`

### I7 Antora never validates fragments
- **Claim.** Antora checks that an xref's target page exists, and appends the fragment unchecked.
- **Proof.** Upstream source (the fragment is split off before `resolveResource`), and Antora
  issues #1077 and #704.
- **Scope limit.** Antora 3.1. A future Asciidoctor-level validator would change this.
- **Verified against.** Antora v3.1.x source, 2026-10-17.
- **Relied on by.** The project's premise, D4, D11.
- **Re-verify.** `curl -s 'https://gitlab.com/api/v4/projects/antora%2Fantora/issues/1077' | grep -o '"state":"[a-z]*"' | head -1`

### I8 A docs.fp.o page names its source file, and on a menu layout its component and version
- **Claim.** Every page has
  `<a href="https://forge.fedoraproject.org/<org>/<repo>/src/branch/<branch>/<start_path>/modules/<module>/pages/<path>.adoc" title="Edit this Page">`,
  and its content is in `<article class="doc">`. A page on the `with_menu` or `category` layout
  also has `<div class="nav-container" data-component="…" data-version="…">`. Unversioned
  components report a placeholder version (`master` for packaging-guidelines, empty for others)
  that their URLs omit. `start_path` is not the component name (`docs` lives in `homepage`).
- **Proof.** Upstream source: the UI bundle's `src/partials/body.hbs` renders the nav partial only
  for the `with_menu`, `404` and `category` layouts, and `src/partials/nav.hbs` sets the data
  attributes only when `page.component` is set. The edit link and article are observed.
- **Scope limit.** Pages from the Fedora UI bundle, Antora 3.1.15. A `:page-layout: without_menu`
  page has no nav container: `arm-sbc`, `docs`, `engineering`, `mindshare`, `spins-labs` and
  `tools`, each a `site-frontpage` component's index. The 404 page has one with no data
  attributes, and no edit link. Source repos on GitHub or GitLab have other edit-URL shapes.
- **Verified against.** docs.fp.o and the UI bundle's `HEAD`, 2026-10-17.
- **Relied on by.** D12, D17, D19, D34, D36.
- **Re-verify.** `for p in packaging-guidelines/ReviewGuidelines arm-sbc; do curl -s https://docs.fedoraproject.org/en-US/$p/ | grep -oE 'data-(component|version)="[^"]*"|<a href="[^"]*" title="Edit this Page">|<meta name="generator"[^>]*>'; done; curl -s 'https://gitlab.com/api/v4/projects/fedora%2Fdocs%2Fdocs-website%2Fui-bundle/repository/files/src%2Fpartials%2Fbody.hbs/raw?ref=HEAD'`

### I9 Anchors on docs.fp.o are static `id`s, and every page carries template chrome ids
- **Claim.**
  - Headings: `<h2 id="_auto_id">`, or an explicit id with no prefix.
  - `[[id]]` renders as an empty `<a id>`.
  - `@asciidoctor/tabs` ids (`_tabs_1_tab_a`…) are static, with JS only toggling visibility.
  - Content carries no `name=` attribute.
  - Chrome ids (`search-input`, `preamble`, `themeSwitch`…) and an empty `id=""` appear on every
    page.
  - Auto-generated ids come from the heading text, so translated pages differ.
- **Proof.** Observed behaviour. The UI bundle's JS was not audited for injected ids.
- **Scope limit.** Antora 3.1 with the Fedora UI bundle.
- **Verified against.** docs.fp.o, 2026-10-17.
- **Relied on by.** D10, D12.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/packaging-guidelines/ReviewGuidelines/ | grep -oE ' (id|name)="[^"]*"' | sort | uniq -c | sort -rn | head -20`

### I10 Antora xrefs, cross-component ones included, render as relative hrefs
- **Claim.** An xref renders as `<a href="../legal/license-approval/" class="xref page">`, with
  the fragment appended when given. Hand-written URLs stay absolute. Output uses the indexify style
  (`page/index.html`).
- **Proof.** Observed behaviour, plus `html_extension_style: indexify` in `docs-fp-o/site.yml`.
- **Scope limit.** Antora 3.1, indexify.
- **Verified against.** docs.fp.o, 2026-10-17.
- **Relied on by.** D7, D22.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/packaging-guidelines/ | grep -oE '<a href="[^"]*" class="xref page">' | head -5`

### I11 A content repo's CI build is partial, and may span several components
- **Claim.** A repo builds with its own `site.yml` (docs-template's `docsbuilder.sh`, image
  `registry.gitlab.com/fedora/docs/docs-website/docs-fp-o:latest`, output `./public`, indexify). That
  playbook can add sibling sources (quick-docs adds three). Neither docs-template's nor quick-docs's
  playbook sets `site.url`, and `docsbuilder.sh` passes neither `--url` nor `URL`, so the build has no site
  URL: no sitemap, no canonical link, an empty `site.path` (I15). Docs repos live on Forgejo at
  forge.fedoraproject.org. The Docs team's content repos run no CI today.
- **Proof.** Upstream source: `docs/docs-template` and `docs/quick-docs` `site.yml`.
- **Scope limit.** Repos following docs-template. Some sources are on GitHub or GitLab.
- **Verified against.** forge.fedoraproject.org, 2026-10-17; the playbooks and `docsbuilder.sh`, 2026-10-21.
- **Relied on by.** D1, D4, D22, D53.
- **Re-verify.** `curl -s https://forge.fedoraproject.org/docs/quick-docs/raw/branch/main/site.yml; curl -s https://forge.fedoraproject.org/docs/docs-template/raw/branch/main/site.yml | grep -c '^ *url: http'; curl -s https://forge.fedoraproject.org/docs/docs-template/raw/branch/main/docsbuilder.sh | grep -n 'CMD='`

### I12 Antora names itself in a generator meta; edit links share no markup across generators
- **Claim.**
  - Antora's default UI emits `<meta name="generator" content="Antora x.y.z">` (`head-info.hbs`),
    and every Antora site sampled carries it. So do MkDocs Material and Docusaurus.
  - Docsy (Hugo) and Sphinx emit none.
  - `rel=edit` marks the edit link in MkDocs Material and Furo. Elsewhere it is told apart by a
    class (`edit-this-page`, `td-page-meta--edit`, `theme-edit-this-page`), a `title`, or its text.
  - Some Antora UIs (asciidoctor.org, couchbase.com) omit `data-component` on the nav container.
    Both search with DocSearch, not lunr (no `data-site-root-path`, I13), and put the version in
    the URL. Couchbase names its coordinates in `docsearch:component`/`docsearch:cversion` metas,
    asciidoctor.org its component in `<meta name="page-spec" content="<component>::<page>">`.
- **Proof.** Upstream source: antora-ui-default `0e38223a` and Docsy `6d6e98b7`. The rest is
  observed output.
- **Scope limit.** A sample of public sites. Any theme can change its markup. The MkDocs Material
  and Docusaurus sites sampled were not recorded by URL.
- **Verified against.** docs.antora.org, docs.asciidoctor.org, docs.spring.io, docs.couchbase.com,
  docsy.dev, kubernetes.io, pradyunsg.me/furo, docs.readthedocs.io, 2026-10-17.
- **Relied on by.** D35, D36, D19.
- **Re-verify.** `for u in docs.antora.org/antora/latest/ docs.asciidoctor.org/asciidoctor/latest/ docs.couchbase.com/server/current/introduction/intro.html; do curl -sL https://$u | grep -oE '<meta name="(generator|page-spec|docsearch:c[a-z]*)"[^>]*>|data-component="[^"]*"|data-site-root-path'; done`

### I13 Antora's search script names the site root, and a page's URL begins with its component
- **Claim.** With `@antora/lunr-extension`, every page has
  `<script id="search-ui-script" data-site-root-path="…">`: the path from the page's directory to
  the site root (`..`, `../../..`), or the site URL's absolute path (`/en-US`) on a page with no
  publish root, such as the 404. A page's URL under that root begins with its component's name.
- **Proof.** Upstream source: lunr-extension `e5973ed7` `data/partials/search-scripts.hbs` writes
  `siteRootPath`, which Antora v3.1.x `packages/page-composer/lib/build-ui-model.js` sets to
  `file.pub.rootPath || site.path`. Observed on all 30 docs.fp.o fixtures.
- **Scope limit.** A site without the lunr extension has no such script, and a page there with no
  nav container (I8) gets no component. A UI may render the partial differently.
- **Verified against.** docs.fp.o fixtures, lunr-extension `main` and Antora v3.1.x, 2026-10-17.
- **Relied on by.** D19.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/arm-sbc/ | grep -o '<script[^>]*search-ui-script[^>]*>'; curl -s 'https://gitlab.com/api/v4/projects/antora%2Fantora-lunr-extension/repository/files/data%2Fpartials%2Fsearch-scripts.hbs/raw?ref=main' | grep search-ui-script`

### I14 A docs.fp.o alias is a 200 HTML page whose meta refresh has delay 0
- **Claim.** An Antora alias, a component's root among them (`/en-US/fedora-docs/` →
  `team-info/`), answers HEAD and GET with 200 `text/html`, no `Location`, and a body of about
  500 bytes carrying `<meta http-equiv="refresh" content="0; url=…">` and a `rel=canonical` to
  the target.
- **Proof.** Observed behaviour: the grilling's probe (`runs/p3-grill/probe-20261017/alias.html`)
  and a crawl of `/en-US/fedora-docs/` on 2026-10-18 (`runs/p3.8-live-20261018/live.duckdb`, the root's
  `redirect_chain`).
- **Scope limit.** Two aliases observed. Were the site to switch Antora's `redirect_facility` from
  `static` to `httpd`, `nginx` or `netlify`, an alias would be a 3xx, already a hop.
- **Verified against.** docs.fp.o, 2026-10-18.
- **Relied on by.** D7, a zero-delay refresh being one of its hops, so an alias read as a page is
  a target whose `final_url` is the page and never a `pages` row.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/fedora-docs/ | grep -o '<meta http-equiv="refresh"[^>]*>'`

### I15 Antora builds a local git repository's working tree, and renders an edit link only for a hosted remote
- **Claim.** Antora 3.1.15 refuses a local content source that is not a git repository ("must be a
  git repository"), and builds the working tree of one named with `branches: HEAD` (an uncommitted
  edit is built). With `CI` set the Fedora UI renders no `file://` link, and a source with no
  github.com, gitlab.com, bitbucket.org or pagure.io `origin` gets no edit link either; one with
  such an `origin` gets `https://<host>/<owner>/<repo>/edit/<branch>/<path>`, as the e2e whole
  build's pages do. A page's nav container carries `data-component` and `data-version`
  (empty for a `version: ~` component) regardless, and a page-alias is a meta-refresh page (I14).
  An xref out of the build logs an error, exits 0 and renders `class="xref unresolved"` (I6). A
  build with no `site.url` writes no sitemap or canonical link, and the UI's footer writes
  root-relative hrefs (`/legal/`) and one `../../../en-US/<component>/<page>/` language link;
  with `site.url` ending `/en-US`, as production's does, the footer writes `/en-US/legal/`.
- **Proof.** Observed behaviour: `mise run e2e` on `fixtures/antora/`, the saved pages
  `fixtures/antora/pages/`; upstream source for the git rule, `content-aggregator` v3.1.15.
- **Scope limit.** Antora 3.1.15 and the UI bundle of `fixtures/antora/ui-bundle.json`. A later UI
  may render these differently.
- **Verified against.** Antora 3.1.15, UI bundle `7f5fb3d`, 2026-10-20.
- **Relied on by.** D24: `scripts/e2e.py` commits each component into a repository under
  `runs/e2e/repos/`, gives each a github.com `origin`, and builds with `CI=true`.
- **Re-verify.** `mise run e2e -- --save-pages && git diff --stat fixtures/antora/pages`

### I16 An Antora page's URL is its component, its version unless bare, its module unless `ROOT`, and its page
- **Claim.** Under the site root, indexified, a page is at `<component>/[<version>/][<module>/]<page>/`. The
  version segment is omitted when the version is empty (`~`) or `master`, and the `ROOT` module's always is.
  A `latest_version_segment` replaces the latest version's own segment (docs.fp.o serves it at `latest`, I5).
- **Proof.** Upstream source: `@antora/content-classifier` 3.1.15, `lib/content-catalog.js`,
  `computeVersionSegment` and `computePub`. Observed on the fixture site: `guide/1.0/…` against `tools/…` and
  `ops/…`, whose versions render `data-version=""` and `master`.
- **Scope limit.** Antora 3.1.15, indexify. A build setting `latest_version_segment` writes the latest
  version under another name, which `crawl::baseline::Held` reads off the segment each built page sits at.
- **Verified against.** Antora 3.1.15, 2026-10-20.
- **Relied on by.** D54, D56, D57, KD17.
- **Re-verify.** `grep -n -A12 'function computeVersionSegment' runs/e2e/antora/node_modules/@antora/content-classifier/lib/content-catalog.js; ls runs/e2e/whole/guide runs/e2e/whole/tools`

### I17 The forge sends an anonymous request for any repository's `issues/new` to `/user/login`
- **Claim.** An anonymous GET or HEAD of forge.fedoraproject.org's `/<owner>/<repo>/issues/new`, with or
  without a query, answers `303` with `Location: /user/login`, no query (the return path is a
  `redirect_to` cookie), for a real repo and for one that does not exist. `/user/login` answers 200, a
  false `ok`. The group's other routes (`issues/new/choose`, `_edit/<ref>/<path>`) do the same.
- **Proof.** Upstream source: Forgejo `v15.0.0` `routers/web/web.go`, the group "endpoints that do require
  authentication" closing with `reqSignIn, context.RepoAssignment`, so sign-in is checked before the
  repo is looked up; `ctx.Redirect` defaults to 303 (`services/context/base.go`). Observed on the forge.
- **Scope limit.** Anonymous requests to Forgejo. GitHub's `/edit/` (302 to `/login?return_to=…`) and
  GitLab's `/-/edit/` (302 to `/users/sign_in`) do the same for their edit routes.
- **Verified against.** forge.fedoraproject.org (`v15.0.10-6450d`) and Forgejo `v15.0.0`, 2026-10-21.
- **Relied on by.** D50, and the guide's `skip` globs (`/user/login*` matching, the Location having no query).
- **Re-verify.** `for p in docs/quick-docs zz-no-org/zz-no-repo; do curl -s -o /dev/null -w "%{http_code} %{redirect_url}\n" https://forge.fedoraproject.org/$p/issues/new; done`

### I18 The forge's robots.txt asks for `Crawl-delay: 1` and nothing else
- **Claim.** forge.fedoraproject.org's `/robots.txt` is exactly `User-agent: *` and `Crawl-delay: 1`: no
  `Disallow`, no `Sitemap:`.
- **Proof.** Observed behaviour.
- **Scope limit.** Only the forge. The file is read at run time (D18), so a change shows as another rate,
  not a wrong verdict.
- **Verified against.** forge.fedoraproject.org, 2026-10-21.
- **Relied on by.** D18 (no root starts on the forge, so the slower rate wins), and the guide's forge
  `per_second = 1` (`docs/manual/deploy.md`).
- **Re-verify.** `curl -s https://forge.fedoraproject.org/robots.txt`

### I19 Every docs.fp.o page carries one `Last build:` line in its footer
- **Claim.** Every docs.fp.o page, the 404 included, has one `<p class="text-center text-xs
  text-fp-gray-dark">Last build: YYYY-MM-DD HH:MM:SS UTC…</p>` inside `<footer>`, after `</article>`, one
  value per build; it may go on ` | Last content update: <date>`, the source file's last commit. Three
  pages fetched again four days after their fixtures differ in that value alone, and asset URLs carry no
  query.
- **Proof.** Upstream source: the UI bundle's `partials/footer-content.hbs` writes `Last build:
  {{{env.ANTORA_DATE}}}`, which docs-fp-o `prod` `build-scripts/rebuild-site.py` sets once a run. That
  nothing else changes between builds of unchanged source is observed behaviour.
- **Scope limit.** Production builds through `rebuild-site.py`; a content repo's `docsbuilder.sh` sets no
  `ANTORA_DATE`. Not covered: a UI bundle change (fetched with `snapshot: true` every build), an Antora
  upgrade (the image installs `@antora/cli@3.1`), another component's nav changing, `{localdate}` in a page.
- **Verified against.** `fixtures/docs-fp-o/` (2026-10-17), UI bundle `7f5fb3d`, docs.fp.o, 2026-10-21.
- **Relied on by.** D15, and `tests/extract.rs`'s `a_new_build_of_a_docs_fp_o_page_keeps_its_hash`.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/a11y/ | diff fixtures/docs-fp-o/en-US/a11y/index.html -; unzip -p fixtures/antora/ui-bundle.zip partials/footer-content.hbs | grep -n 'Last build'` (one changed line)

### I20 Antora orders a component's versions empty first, then names, then semantic versions; latest is the first not a prerelease
- **Claim.** Antora 3.1.15 sorts versions newest first: the empty version (`~`); then names, in reverse
  `localeCompare(…, 'en', {numeric: true})` order (`rawhide` > `f44` > `f10` > `f9`, case-folded); then
  semantic versions (a digit, or `v` and a digit, then a `.` or an integer), numerically on three parts, a
  `-` suffix below its release and compared with the same collation, a non-numeric part low. A component's
  `latest` is the first whose `antora.yml` is not `prerelease`: docs.fp.o's fedora lists `rawhide`
  (`prerelease: True`) first, and `f44` is latest.
- **Proof.** Upstream source: `@antora/content-classifier` 3.1.15 `lib/util/version-compare-desc.js` and
  `lib/content-catalog.js` (`latest`, `latestPrerelease`). Observed: the rawhide page's version menu, and
  `docs/release-docs-home`'s `antora.yml`.
- **Scope limit.** Antora 3.1.15. Prerelease is a descriptor flag no page carries.
- **Verified against.** Antora 3.1.15 and docs.fp.o, 2026-10-21.
- **Relied on by.** D19, D55 (`profile::antora::resolve::compare`, `collate`), KD14.
- **Re-verify.** `sed -n '22,31p' runs/e2e/antora/node_modules/@antora/content-classifier/lib/util/version-compare-desc.js; sed -n '101,105p' runs/e2e/antora/node_modules/@antora/content-classifier/lib/content-catalog.js; curl -s https://forge.fedoraproject.org/docs/release-docs-home/raw/branch/main/antora.yml | grep -E '^(version|prerelease)'`

### I21 Antora's resource ID is `version@component:module:family$relative`, defaulted from the referring page
- **Claim.** The grammar is `^(?:([^@:$]+)@)?(?:(?:([^@:$]+):)?(?:([^@:$]+))?:)?(?:([^@:$]+)\$)?([^:$][^@:$]*)$`:
  one colon names a module, two a component. With a component, no module is `ROOT` and no version the
  component's latest (I20); an unknown component is unresolved. Without one, the component, the module
  and, unless given, the version are the referring page's. `_` is the empty version. No family is `page`
  for an xref, which permits every family. `./` and `..` are normalised; a spec the regex refuses is
  unresolved; a miss in `page` is retried as an alias.
- **Proof.** Upstream source: `@antora/content-classifier` 3.1.15 `lib/util/parse-resource-id.js` and
  `lib/util/resolve-resource.js`; `@antora/asciidoc-loader` `lib/xref/convert-resource-ref.js`.
- **Scope limit.** Antora 3.1.15.
- **Verified against.** Antora 3.1.15, 2026-10-21.
- **Relied on by.** D19, D55, KD10 (`profile::antora::resolve::parse`).
- **Re-verify.** `sed -n '5p;42,56p' runs/e2e/antora/node_modules/@antora/content-classifier/lib/util/parse-resource-id.js; sed -n '30,35p' runs/e2e/antora/node_modules/@antora/content-classifier/lib/util/resolve-resource.js`

### I22 Antora builds a page whose image is missing, logging an error and rendering the target as written
- **Claim.** An `image:` target that is no URL and resolves to no published image logs `target of image not
  found: <target>` at `error`, gains role `unresolved`, and renders `<img src="<target>">` as written. The
  build exits 0 unless `runtime.log.failure_level` (`--log-failure-level`) is `error` or `warn`, the default
  being `fatal`; `docsbuilder.sh` passes none and neither content playbook sets one.
- **Proof.** Upstream source: `@antora/asciidoc-loader` 3.1.15 `lib/converter/html5.js` (`$image_uri`,
  `logUnresolved`); `@antora/playbook-builder` `lib/config/schema.js`. Observed: `runs/e2e/partial-changed/`,
  a build `scripts/e2e.py` requires to exit 0.
- **Scope limit.** Antora 3.1.15. A repo whose CI sets a failure level of `error` fails its build first.
- **Verified against.** Antora 3.1.15, docs-template `main`, 2026-10-21.
- **Relied on by.** D9, D13 (the gate, not Antora, fails such a page).
- **Re-verify.** `grep -n -A4 'function logUnresolved' runs/e2e/antora/node_modules/@antora/asciidoc-loader/lib/converter/html5.js; grep -n -A4 'failure_level: {' runs/e2e/antora/node_modules/@antora/playbook-builder/lib/config/schema.js`

### I23 Forges spell a file's URL as repo, marker, ref and path
- **Claim.** Forgejo: `/<o>/<r>/src/branch|tag|commit/<ref>/<path>`, its edit page `/_edit/<ref>/<path>`
  behind sign-in. GitLab: `/<group…>/<r>/-/blob|edit/<ref>/<path>`; a bare `/blob/` is served as it is, a
  bare `/edit/` is a 301 to `/-/edit/`. GitHub: `/<o>/<r>/blob|edit/<ref>/<path>`. Pagure:
  `/[<ns>/]<r>/blob|edit/<ref>/f/<path>`, also under `/fork/<user>/`. Everywhere ref and path are one glob,
  so a ref holding `/` is not told apart by the URL (KD1). Antora's default edit URL is `edit` on github.com
  and gitlab.com for a branch (`blob` for a tag) and `/<r>/blob/<ref>/f/<path>` on pagure.io.
- **Proof.** Upstream source: Forgejo `v15.0.0` `routers/web/web.go`; GitLab `config/routes/repository.rb`
  and `repository_deprecated.rb`; Pagure `pagure/ui/repo.py`; `@antora/content-aggregator`
  `lib/compute-origin.js`. GitHub's is observed only.
- **Scope limit.** pagure.io was sunset in 2026 and is a static snapshot, its blob URLs 404. Bitbucket's
  `/src/<ref>/<path>` and Forgejo's deprecated `/src/<ref>/…` are no shape `forge::parse` reads.
- **Verified against.** forge.fedoraproject.org, gitlab.com, github.com, pagure.io and the sources above, 2026-10-21.
- **Relied on by.** D36, KD1, KD2 (`forge::parse`).
- **Re-verify.** `for u in forge.fedoraproject.org/docs/quick-docs/src/branch/main/site.yml gitlab.com/fedora/docs/docs-website/ui-bundle/-/blob/HEAD/src/partials/body.hbs gitlab.com/fedora/docs/docs-website/ui-bundle/edit/HEAD/src/partials/body.hbs github.com/rust-lang/rust/blob/main/README.md; do curl -s -o /dev/null -w "%{http_code} %{redirect_url} $u\n" https://$u; sleep 1; done`

### I24 A CI runner cancels a job with SIGINT then SIGTERM (GitHub Actions) or SIGTERM alone (GitLab), and kills it soon after
- **Claim.** GitHub Actions sends SIGINT to the step's entry process, SIGTERM 7500 ms later if it has not
  exited, and kills the process tree 2500 ms after that. GitLab Runner's shell executor sends SIGTERM to the
  process and its children, and SIGKILL after 10 minutes.
- **Proof.** Documentation: GitHub's "Workflow cancellation" reference and GitLab Runner's shell executor
  page. Not observed.
- **Scope limit.** Unix runners; GitLab's other executors and a configurable grace period are not read.
- **Verified against.** Both pages, 2026-10-21.
- **Relied on by.** D58 (each signal is a press; GitHub's 10 s is shorter than the stop's 30 s grace, so its
  SIGTERM must abandon).
- **Re-verify.** `curl -s https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-cancellation | grep -o '7500 ms\|2500 ms\|SIGINT\|SIGTERM' | sort -u; curl -s https://docs.gitlab.com/runner/executors/shell/ | grep -o 'SIGTERM[^.]*SIGKILL[^.]*'`

### I25 A docs.fp.o page's language menu links the same page under every locale, one list site-wide
- **Claim.** Every page docs.fp.o's build renders carries `<a class="language" href="{siteRootPath}/../<locale>{page.url}">`
  for `en-US` and each translated locale, in one list the same on every page: the link leaves the page's own URL at
  the locale segment alone (`../../ar/a11y/` on `/en-US/a11y/`), so that segment, as a directory, is the locale's
  prefix on the host. The 404, which has no generator meta, writes the root-relative `/en-US/../ar`.
- **Proof.** Upstream source: docs-fp-o `prod` `8b647a46e5` `build-scripts/rebuild-site.py`, `generate_lang_switch_ui`,
  called with `["en-US"] + list(languages.keys())` and written over the UI's `partials/page-languages.hbs`, whose own
  (ui-bundle `main`) links `en-US` alone as `language is-active`. Observed on all 30 docs.fp.o fixtures: 68 links each.
- **Scope limit.** docs.fp.o's production build; a content repo's CI build uses the UI's partial, naming no other
  locale. Another site's UI may name its locales otherwise, or not at all.
- **Verified against.** docs-fp-o `prod` and the fixtures (fetched 2026-10-17), 2026-10-24.
- **Relied on by.** D67.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/a11y/ | grep -o '<a[^>]*class="language"[^>]*>' | head -3; curl -s 'https://gitlab.com/api/v4/projects/fedora%2Fdocs%2Fdocs-website%2Fdocs-fp-o/repository/files/build-scripts%2Frebuild-site.py/raw?ref=prod' | grep -n 'class="language"\|lang_switch_ui('`
