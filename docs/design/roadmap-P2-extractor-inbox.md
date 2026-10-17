# P2 inbox

Facts addressed to the extractor phase before it is grilled. Drained when it
is, and then deleted (`../process.md`, "Inboxes: facts filed by destination").

## Pages with no nav container still have a component

- **Fact.** A page on the `without_menu` layout renders no nav container, so
  no `data-component` or `data-version` (I8). It still belongs to an Antora
  component, and still has its edit link. Six such pages exist today, each a
  `site-frontpage` component's index; `arm-sbc/` is the fixture. The component
  is the URL's first segment under the site root, which the page itself gives
  (`data-site-root-path` on the search script). The edit link's `start_path`
  is not the component (`docs` lives in `homepage`).
- **Why P2 cares.** The maintainer settled the `antora` Extract hook's answer
  (D19, D38): with no nav container, take `antora.component` from that URL
  segment, module and path from the edit link as always, and record no
  version, which D19 already reads as the component's only or latest one.
  Without it, every xref to `docs::index.adoc` and the like in a partial CI
  build (I11, I6) is a false breakage. Still open for the grilling: whether a
  derived component is marked as derived in `page_facts` (D37). Reasoning:
  `../status/history/2026-10-17.md`, "I8 reviewed: the nav container follows
  the layout".
- **Origin.** P1.1, and the review of its call, 2026-10-17. Contingent on
  docs.fp.o keeping Antora's URL scheme with no per-component URL override.

## docs.fp.o's 404 page has no generator meta

- **Fact.** The site's not-found page
  (`fixtures/docs-fp-o/en-US/no-such-page-bruce-fixture/404.html`) carries the
  Antora chrome but no `<meta name="generator">`, so under `auto` (D35) it is
  profiled `generic`, its region drawn by `<main class="article">` though it
  has an `article.doc` as every other page does.
- **Why P2 cares.** The extractor decides whether a non-2xx body is extracted
  at all (D16 already makes the page `broken_url`); if it is, its facts come
  from the wrong profile. A local build's `404.html` (D1, P5) reaches the
  extractor as an ordinary page.
- **Origin.** P1.2, 2026-10-17. Contingent on the Fedora UI bundle's 404
  layout omitting the meta.
