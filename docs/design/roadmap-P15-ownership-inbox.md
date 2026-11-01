# P15 inbox

Facts addressed to the phase giving every site what a repo owns before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## The baseline names each page's repo, though a CI build's pages do not

- **Fact.** The held set is recorded only from a built page's
  `antora.component` and `antora.version` (`crawl::Shared`, at KD6's
  marker; `crawl::baseline::Held`), and `report::inverse` reads it back the
  same way. D56 rejected touched pages by `source_repo` because a CI build's
  pages carry no edit link (I15); the baseline's crawled pages do
  (`pages.source_repo`, D17), and the inverse check already reads the
  baseline's pages.
- **Why P15 cares.** A held set of the baseline's pages naming the repo a
  CI run builds needs no fact from the build, which answers D56's ground for
  refusing it, and holds by repo where KD5 holds a component whole; whether
  it replaces the component-version for Antora or sits beside it is the
  call.
- **Origin.** P15's sketching, 2026-11-01, read off the source at `9fb39b1`.
  Contingent on the held set still being read from `antora.*` facts.
