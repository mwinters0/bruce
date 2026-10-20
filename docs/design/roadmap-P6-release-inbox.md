# P6 inbox

Facts addressed to the release and deployment-guide phase before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## A full en-US crawl takes nearly two hours, hosts nearly in turn

- **Fact.** P4.10's crawl of docs.fp.o `/en-US/` with the forge allowlisted,
  from the guide's config, took 1:48:22 wall clock and 161 MiB. The hosts ran
  nearly one after the other: the sitemap's pages took the first 39 minutes,
  and the forge's 4155 requests at one a second began then, so the time is
  close to their sum. P4.4's run, before the guide's skip glob was fixed, took
  2:20:42.
- **Why P6 cares.** The guide's scheduled-crawl section sets how long a night
  the crawl may take and what a deployer is told to expect; whether the hosts
  could overlap (the frontier's order, not read) changes that figure.
- **Origin.** P4.4 and P4.10, 2026-10-19 (`roadmap-P4-reports-notes.md`,
  "Reproducing the evidence"). Contingent on the forge staying at one request
  a second and about two requests a docs.fp.o page.
