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
