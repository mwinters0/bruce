# P14 — Redirects as facts

What P14 does and why, settled in the grilling of 2026-11-02. The decisions
are in [`decisions.md`](decisions.md) and are cited here,
never restated.

## Intent

A page's discovery and a link's redirect are read from the facts the store
holds: who names a URL (`links`, a sitemap's entries by kind `sitemap`) and
where it leads (`targets.final_url` and `redirect_chain`, D7, D46), not from
the flags of whichever target claimed the page first (KD38). And the length
past which a redirect chain is broken is the site's to configure
(maintainer, 2026-11-01), since some owners hold every redirect a problem to
solve. P9 follows P14: its guide sets docs.fp.o's limit.

## A chain over the limit is broken

Settled 2026-11-02 (maintainer). **A link whose target's chain is longer than
the limit is `broken_url`, reason `too_many_redirects`**, the reason the
crawl already writes past D7's ceiling, one name for more hops than allowed.
It is judged in `link_verdicts` at read time from the recorded policy (D77,
D90), so a trial `--config` (D84) and a comparison (D81) see it with no new
crawl; the gate fails on it through `gate.fail`'s default (D83), and a site
softens its dot through the reason's `[severity]` key (D82). A target that is
itself not `ok` keeps its own outcome and reason, the worse fault; a link over
the limit is broken whatever its fragment, its anchor not looked at, as with
`doubled_slash` (D71). The row still names its landing, which is the fix.
Rejected: a new reason apart from the crawl's ceiling, the reader seeing one
thing; `ok` with a level alone, which the gate could fail only by `gate.fail`
taking reasons, and which is not broken.

## The limit counts every hop

Settled 2026-11-02 (maintainer). **The limit counts every entry of
`redirect_chain`**: each 3xx and each delay-0 refresh (D7), an `http` to
`https` upgrade included. A slash-only chain counts like any other, and D70's
`pedantic_trailing_slash` stays the verdict of an `ok` link, so it holds only
for a slash-only chain within the limit. Why: the limit is what the reader
sits through; a slashless docs.fp.o URL bounced through `http://` (I4) is two
round trips whoever is at fault, and a site disagreeing raises its limit or
its host's. Rejected: a slash-only chain exempt at any length; the slash hop
alone uncounted.

## The setting is `max_redirects`, per host, default 10

Settled 2026-11-02 (maintainer). **`default.max_redirects`, an integer from 0
to 10, and `host."…".max_redirects` over it**, keyed by the linked URL's host,
which answered the first hop and owns the chain (D79, as `doubled_slash` is).
0 makes any redirect broken; past 10 is refused, the crawl breaking those
already (`crawl::hops::MAX_HOPS`). **The default is 10**, D7's ceiling, so a
file without the key judges as before (D90) and every row stays an integer
(D78). Why per host: a site tolerates a forge's login walls (four hops from
`src.fedoraproject.org` to `id.fedoraproject.org`) while holding its own
pages to one. Rejected: a tighter default, changing every existing run's
verdicts; top-level only under `[judge]`; keyed by the landing's host.

## An over-limit link stays in the redirect listings

Settled 2026-11-02 (maintainer). **`report_redirects` keeps every counted
redirected link whatever its verdict**, so a link over the limit is listed
among the broken by its verdict and among the redirects (the report, the
digest's section, the gate's introduced redirected links) by its chain and
landing, the fix, as a redirect onto a 404 is now. Rejected: dropping links
not `ok` from the view, one list each, which changes the redirect-onto-404
rows too and loses their landing.

## A target that redirects differently has moved

Settled 2026-11-02 (maintainer). **The digest's `target_moved` holds where
the target's landing or its chain differs from the compared run's** (D74), so
a link newly over the limit at the same landing names the target's change, its
source shown the target's, not `unexplained`. A tightened limit is no new
break, both runs judged by the digest's run's policy (D81). Rejected: leaving
it `unexplained`; a cause of its own, D74 refusing a cause per reason.
Between run 1 and run 2 of the 2026-11-01 database, no target kept its landing
with another hop count, and 17 changed landing.

## Listed and linked are derived, and the flags dropped

Settled 2026-11-02 (maintainer). **`pages.in_sitemap` and `pages.via_links`
are dropped** (a schema change, D43), and `report_pages` derives **`listed`
and `linked`, each `'direct'`, `'redirect'` or NULL**, direct winning where
both hold: `listed` from `sitemap` links, `linked` from the links
`report_inbound` counts (no sitemap kind, no page's link to itself). A seeded
page that is neither is told apart by `root`, as now. Why: `via_links` is no
fact, meaning a link claimed the page before a sitemap did, so it is false on
2,148 listed pages other pages link to; its readers only show it, sort by it,
or read its NULL as a sitemap entry with no page (`report_sitemap`, the
digest's sitemap entries), and those read the derived columns. Rejected: four
booleans; the flags written at the run's end, a claim still.

## Through a redirect is listed and linked

Settled 2026-11-02 (maintainer). **A page a sitemap entry reaches only by a
redirect is not `unlisted`**, a reader of the sitemap getting there, and **a
page linked only through a redirect is no orphan**, as `report_inbound`
already counts it. `report sitemap` shows `listed` and `linked` in place of the
flags. The redirect itself is the finding: a `sitemap`-kind row in
`redirects`, or broken past the limit. Rejected: a redirecting entry not
listing its landing, which reports one defect twice.

## D47's ordering stays, for other reasons

Settled 2026-11-02 (maintainer). **A run still reads its sitemaps before any
page**, D47's Why rewritten: every listed URL is claimed from the start, so a
stopped run's "cut" (D48) counts every listed page it lacks, and a listed
page's plan is the sitemap's, never a link's. Rejected: reading the files
beside the pages, saving about a minute of a multi-hour crawl at docs.fp.o's
pace for a cut that undercounts.

## What docs.fp.o's discovery is

Read 2026-11-02 off run 2 of `runs/en-us-20261101-0005/en-us.duckdb`, the
derivation as SQL over the stored rows: listed directly is exactly
`in_sitemap`, 2,310 pages, and through a redirect none (no sitemap entry
redirects). Linked through a redirect only: 412 allowlisted pages and 1
crawled. These are what the derived columns must reproduce.

## What docs.fp.o's chains are

Read 2026-11-01 off run 2 of `runs/en-us-20261101-0005/en-us.duckdb`: 798
redirected targets, by hops 646, 131, 14, 6 and 1 (one to five). Slash-only:
241 of one hop, 31 of two (I4's, through `http://`). The limit breaks only a
target that is `ok`, 716 of them (577, 127, 8, 3, 1): a limit of 1 breaks 139,
31 of them slash-only; a limit of 0, all 716.

## Where it lives

Settled 2026-11-02 (maintainer). **P14 adds no module**; each part widens a
concept already placed (`concepts.md`): the setting in
`config::policy`, the rule in `store::views`' `link_verdicts`, the derived
columns in `report::views` and the reach reports, the cause in
`report::digest::cause`, the dropped columns in `store::schema` and the
crawl's claim (`crawl`).

## The slices

Settled 2026-11-02 (maintainer). Two, **discovery first**, the store's change
landing before the judging's, each its own review:

1. **Discovery as facts**: the flags dropped, `listed` and `linked` derived,
   the reach reports and the digest's sitemap entries, orphans and unlisted
   reading them, D14 and D47 amended, KD38 struck, and `review.py crawl`
   starting a new database where the newest kept one is of another schema.
   Its evidence is the view's SQL over the kept 2026-11-01 database,
   reproducing "What docs.fp.o's discovery is".
2. **The chain limit**: `max_redirects` and its rows, the manual's row and
   Housekeeping, the `link_verdicts` rule, `target_moved` widened, D7 amended
   and an entry of its own.

No slice crawls: a fresh en-US crawl under the new schema is P9's, with the
guide's limit. The schema change makes D43 refuse every kept database, and
`review.py crawl` only copies the newest, so 14.1 gives it a fresh start
(maintainer, 2026-11-02): D43 tells a user to start a new database, and the
review tooling does the same at this change and every later one.
