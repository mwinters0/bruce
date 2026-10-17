# P2 inbox

Facts addressed to the extractor phase before it is grilled. Drained when it
is, and then deleted (`../process.md`, "Inboxes: facts filed by destination").

## Pages with no component

- **Fact.** Six docs.fp.o pages (`arm-sbc`, `docs`, `engineering`, `mindshare`,
  `spins-labs`, `tools`) have no `nav-container`, yet carry the three
  edit-link `title`s. Their edit link points into `docs/site-frontpage`, with
  the component's directory first in the path.
- **Why P2 cares.** The `antora` Extract hook (D19, D38) must record no
  `antora.component` for them and still return the source tie-back (D36), and
  D19's Resolve cannot assume every page has coordinates.
- **Origin.** P1.1, 2026-10-17; the fixture is `fixtures/docs-fp-o/en-US/arm-sbc/`.
  Contingent on the site-frontpage build staying a separate source from the
  component builds.
