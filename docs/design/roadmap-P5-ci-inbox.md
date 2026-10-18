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
- **Origin.** P3's grilling, 2026-10-17 (`../wip/roadmap-P3-crawler.md`,
  "Redirects").
