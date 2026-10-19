# P5 inbox

Facts addressed to the CI-mode phase before it is grilled. Drained when it
is, and then deleted (`../process.md`, "Inboxes: facts filed by
destination").

## Links through a redirect: both uses, no setting

- **Fact.** The maintainer settled that some users track every link through
  a redirect as a task to point it at its target, while others care only
  about broken links, and that both are supported **without
  configuration**. D7 makes `redirected` non-failing, and D9 fails CI only
  on introduced breakage.
- **Why P5 cares.** `check-dir`'s output serves both: introduced redirected
  links listed apart from the breakage that fails the gate, never failing it.
- **Origin.** P3's grilling, 2026-10-17 (`roadmap-P3-crawler.md`,
  "Redirects").

## D20's "did you mean" hint waits for Resolve

- **Fact.** P4 places a link with `xref_refspec` set as `broken_url`,
  reason `unresolved_xref` (D20), in its `link_verdicts` view, and builds
  no hint: D20's fuzzy match over resource IDs is left for the Resolve hook
  (D19, D38), which is empty in `antora` today.
- **Why P5 cares.** Building Resolve, it decides whether the same lookup
  gives the live report's hint, and where the hint lives: a column of the
  view, or the report's rendering.
- **Origin.** P4's grilling, 2026-10-19
  (`../wip/roadmap-P4-reports.md`, "Where the fragment cannot be checked").
