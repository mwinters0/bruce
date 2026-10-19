# P4.2.1 A skipped hop ends the chain: notes

`Frontier::hop_plan` leaves a skipped URL `Plan::Unchecked("skipped")`
(`frontier::SKIPPED`) where it turns every other `Unchecked` into a status
request, and `Crawl::hop` does the same for a sitemap's hop. `Crawl::walk`
breaks with `End::Blocked("skipped")` before it asks robots.txt, so the hop
makes no request of any kind. The rows are a robots-blocked hop's (D50).

## What the next slice inherits

- **P4.3's guide** may skip `/user/login*` beside `issues/new`: a link that
  redirects onto it is `unchecked` / `skipped`, with its `final_url` the login
  URL and the login URL a target of its own (`unchecked` / `skipped`),
  claimed once however many links lead there.
- **`link_verdicts` (P4.5)** needs nothing new: the link's target is not `ok`,
  so it gets the target's outcome and reason.
- **`checked_at` of the landing row** is set when the hop before it was
  requested, as for a robots-blocked landing (`asked`); the skipped landing
  was never requested itself.

## Negative results

- A set or discovered sitemap whose own URL a glob matches is still read:
  `sitemap_item` plans it `Sitemap` without asking `skip`, and only a *hop*
  onto a skipped URL ends there. The spec names hops only, so it is left.
- A probe (`sitemap.xml`, `/sitemap.xml`) that ends skipped is a miss and
  writes nothing, as any probe that is not a sitemap does.
