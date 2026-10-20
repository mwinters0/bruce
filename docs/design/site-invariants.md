# Site invariants

Properties of the sites we check, and of the tools that build them, that a decision treats as
guaranteed. Rules: `../process.md`, "The assumptions register". Each `I<n>` is allocated on
discovery and never reused. Walk this file whenever Antora's version changes (I8's generator tag
names it), or when docs.fp.o's build or hosting moves: a change here surfaces as wrong verdicts,
not as an error.

<!-- invariant-watermark: I14 -->

### I1 docs.fp.o's sitemap is an index at `/en-US/sitemap.xml`, and lists no translated page
- **Claim.** `/sitemap.xml` is 404, and robots.txt names no `Sitemap:`. `/en-US/sitemap.xml` is
  a `<sitemapindex>` of per-component `sitemap-<component>.xml` files (68 on 2026-10-17), each with
  `<loc>` and `<lastmod>`. Locale sitemaps repeat the en-US index.
- **Proof.** Observed behaviour.
- **Scope limit.** Says nothing about pages reachable only by links.
- **Verified against.** docs.fp.o, 2026-10-17.
- **Relied on by.** D14, D10.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/sitemap.xml | grep -c '<loc>'`

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
- **Relied on by.** D7, D10.
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
- **Re-verify.** `for p in packaging-guidelines/ReviewGuidelines arm-sbc; do curl -s https://docs.fedoraproject.org/en-US/$p/ | grep -oE 'data-(component|version)="[^"]*"|<a href="[^"]*" title="Edit this Page">|<meta name="generator"[^>]*>'; done; curl -s https://gitlab.com/fedora/docs/docs-website/ui-bundle/-/raw/HEAD/src/partials/body.hbs`

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
  playbook can add sibling sources (quick-docs adds three). Docs repos live on Forgejo at
  forge.fedoraproject.org. The Docs team's content repos run no CI today.
- **Proof.** Upstream source: `docs/docs-template` and `docs/quick-docs` `site.yml`.
- **Scope limit.** Repos following docs-template. Some sources are on GitHub or GitLab.
- **Verified against.** forge.fedoraproject.org, 2026-10-17.
- **Relied on by.** D1, D4, D22.
- **Re-verify.** `curl -s https://forge.fedoraproject.org/docs/quick-docs/raw/branch/main/site.yml`

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
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/arm-sbc/ | grep -o '<script[^>]*search-ui-script[^>]*>'; curl -s https://gitlab.com/antora/antora-lunr-extension/-/raw/main/data/partials/search-scripts.hbs | grep search-ui-script`

### I14 A docs.fp.o alias is a 200 HTML page whose meta refresh has delay 0
- **Claim.** An Antora alias, a component's root among them (`/en-US/fedora-docs/` →
  `team-info/`), answers HEAD and GET with 200 `text/html`, no `Location`, and a body of about
  500 bytes carrying `<meta http-equiv="refresh" content="0; url=…">` and a `rel=canonical` to
  the target.
- **Proof.** Observed behaviour: the grilling's probe (`runs/p3-grill/probe-20261017/alias.html`)
  and P3.8's crawl (`runs/p3.8-live-20261018/live.duckdb`, the root's `redirect_chain`).
- **Scope limit.** Two aliases observed. Were the site to switch Antora's `redirect_facility` from
  `static` to `httpd`, `nginx` or `netlify`, an alias would be a 3xx, already a hop.
- **Verified against.** docs.fp.o, 2026-10-18.
- **Relied on by.** D7, a zero-delay refresh being one of its hops, so an alias read as a page is
  a target whose `final_url` is the page and never a `pages` row.
- **Re-verify.** `curl -s https://docs.fedoraproject.org/en-US/fedora-docs/ | grep -o '<meta http-equiv="refresh"[^>]*>'`

### I15 Antora builds a local git repository's working tree, and a partial build renders no edit link
- **Claim.** Antora 3.1.15 refuses a local content source that is not a git repository ("must be a
  git repository"), and builds the working tree of one named with `branches: HEAD` (an uncommitted
  edit is built). With `CI` set the Fedora UI renders no edit link, and a source with no forge
  remote gets none either way. A page's nav container carries `data-component` and `data-version`
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
  `runs/e2e/repos/` and builds with `CI=true`.
- **Re-verify.** `mise run e2e -- --save-pages && git diff --stat fixtures/antora/pages`
