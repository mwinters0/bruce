# P6 inbox

Facts addressed to the release and deployment-guide phase before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## A full en-US crawl takes over two hours, hosts in turn

- **Fact.** P4.4's crawl of docs.fp.o `/en-US/` with the forge allowlisted took
  2:20:42 wall clock and 168 MiB. The hosts ran one after the other, the
  sitemap's 2317 pages in the first 47 minutes and the forge's 6056 requests
  at one a second after, so the time is nearly their sum. 1908 of the forge's
  requests were links a `skip` glob should have caught (fixed in the guide
  before P4.10).
- **Why P6 cares.** The guide's scheduled-crawl section sets how long a night
  the crawl may take and what a deployer is told to expect; whether the hosts
  could overlap (the frontier's order, not read) changes that figure.
- **Origin.** P4.4, 2026-10-19 (`../wip/roadmap-P4.4-first-crawl-notes.md`).
  Contingent on P4.10's run, with the glob fixed, taking about 47 + 69
  minutes.
