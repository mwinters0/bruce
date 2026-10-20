# P5.12 notes: Resolve through the URL a refspec names

Outcome: where `antora`'s Resolve finds no baseline page with a refspec's
coordinates, it returns the URL I16 names (D57), and the crawl answers it as it
answers any URL the build lacks: the baseline's rows, else a request. KD7 and
KD8 are struck, index lines and markers together. 5.11's as-published check,
re-run, exits 0 with nothing introduced or inverse.

- **Evidence**: `runs/p5.12-quick-docs/` (`published.json`, `ci.duckdb`), the
  command of `roadmap-P5.11-quick-docs-notes.md` run with the debug binary
  against the same `public/` and baseline: exit 0, 1482 targets, 236 pages, 91
  already broken (unchanged), 0 introduced, 0 inverse, 239 redirected. The
  three new redirected rows are the three xrefs 5.11 listed as introduced:
  `project::objectives.adoc` (a crawled alias, the baseline's rows) and the two
  `package-maintainers::Packaging_Tutorial_GNU_Hello.adoc` (an uncrawled alias,
  answered live: `ok`, landing `Packaging_Tutorial_2_GNU_Hello/`). The fourth,
  `releases::spins.adoc`, is the baseline's `ok` page at the shared URL. One
  request left the machine.
- **Tests**: `profile::antora::resolve::tests` (the URL for each layout case,
  a page with no edit link, a latest served at another segment),
  `crawl::baseline::tests::an_xref_no_page_has_the_coordinates_of_resolves_by_the_url_it_names`
  (crawled alias, shared URL, uncrawled alias live, nothing there),
  `store::views::tests::a_resolved_xref_whose_url_is_broken_is_unresolved_and_one_not_known_to_be_is_not`.
  `mise run e2e` plants `xref:ops::disks.adoc`, the fixture's alias: it is
  absent from `introduced` and present in `redirected`, where the nonexistent
  `guide::nonexistent.adoc` stays an introduced `unresolved_xref`.

## What the next work inherits

- **Every xref no baseline page carries is now a request** when the baseline
  holds no target for its URL: a PR with many xrefs to pages that do not exist
  asks production for each, at the config's pace.
- **A resolved link's verdict can now be `unresolved_xref` with a target that is
  not its own page**: the URL tried, which `report` rows name. The gate key
  for it is that URL and the refspec's fragment, as for a facts-resolved one.
- **The P6 guide** says a CI job reaches the live site for these, and that the
  config's allowlist is not what lets it (the URL is in a root's scope).

## Limits met, not removed

- **A component-version whose page the baseline holds is not under its own
  version segment** (the latest of docs.fp.o's `fedora`, served at `latest`,
  I5) names no prefix, so `f44@fedora::x.adoc` with no page carrying
  `x.adoc` stays unresolved. Older versions, and every versionless component,
  are named.
- **A component-version with no baseline page carrying a `version` fact**
  (all its pages without a nav container) names no prefix either.
- **The layout lives in three places** now: `crawl::baseline::Held::holds`,
  `report::inverse::prefix` and `profile::antora::resolve::prefix`. A fourth
  reader earns a shared helper, which D39's layering puts below `profile`.

## Negative results

- **"Not `ok`" as the rule for an unresolved xref** (the review's wording) fails
  the gate on an `error` or an `unchecked` URL; rule 1 takes `broken_url`
  alone (STATUS, "Decisions worth another look").
- **Resolving before planning the URL** (asking the chain inside Resolve) is
  out: Resolve sees only the baseline (D38), and the crawl owns the request.
