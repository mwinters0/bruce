# P4 — Verdicts, reports, and the first en-US crawl

What P4 does and why, settled in its grilling on 2026-10-19, which drained
its inbox. Progress is [`../status/STATUS.md`](../status/STATUS.md)'s
checklist, never this file. The decisions are in
[`../design/decisions.md`](../design/decisions.md) and are cited here, never
restated.

## Intent

Post-crawl fragment verdicts in SQL (D28), outcome classification of links
(D16), `bruce report` (D6, D27) and `bruce prune` (D2), ending with a real
en-US crawl that delivers the sitemap, inbound ranking and both broken stats
before any CI work, run from the simulated deployment (D41, D42).

## A link's verdict is a view

Settled 2026-10-19. **A link's outcome, reason and whether it went through a
redirect are a view, `link_verdicts`**, joining `links` to its target, the
target's `final_url` target, and that page's `anchors`, computed whenever it
is read (D28). **`links.outcome`, `links.reason` and `links.redirected` are
dropped**, a schema change to version 3 with no migration (D43), which
closes KD3. P4.5 times the view on P4.4's en-US DB; that
measurement is the evidence for materialising it if it proves slow.
Rejected: a `verdicts` table written once at a run's end, which leaves a
stopped run without verdicts unless a second path computes them, the logic
then living twice; keeping the columns under a same-named view, columns that
lie to an ad-hoc `duckdb` user.

### Where the fragment cannot be checked

Settled 2026-10-19, placing each `targets.reason` a crawl leaves
(`src/crawl/`):

- **A target that is not `ok`** gives its link the target's outcome and
  reason; the fragment is not looked at.
- **An `ok` target with no page to read** (`too_large`, an asset, an
  allowlisted URL first named by a non-anchor link, KD4) gives `unchecked`,
  reason `no_anchors`: never broken, since nothing is known. KD4 therefore
  surfaces as `unchecked`, never as a false break.
- A fragment checks against the landing page's `anchors` where the target
  was redirected (D7, D46). `#`, `#top` and an empty fragment are `ok`; a
  text fragment (`:~:`) is `unchecked` (D12).
- **A link with `xref_refspec` set is `broken_url`, reason
  `unresolved_xref`** (D20). D20's "did you mean" hint over resource IDs
  waits for P5's Resolve (D19).

## Which reports

Settled 2026-10-19. P4 delivers every single-run report and one over two
runs:

- **Single run:** `broken` (`broken_url` and `broken_anchor`), `errors`
  (D16 keeps `error` apart), `stats` (each page's broken-URL and
  broken-anchor counts, the goals' per-page stat), `inbound`, `sitemap`,
  `orphans`, `unlisted`, `redirects`.
- **Two runs:** `diff`, the links newly broken and newly fixed.

**`anchors-removed` and `changed` (pages whose content hash moved) are
P8's** (`../design/roadmap.md`). Nothing in P4 then needs a run's hash
scope recorded, and the question of recording it is in P8's inbox,
[`../design/roadmap-P8-history-inbox.md`](../design/roadmap-P8-history-inbox.md).

### Filters and output

Settled 2026-10-19. A report takes `--repo R` (`pages.source_repo`, D17)
and **`--fact key=value`**, repeatable, over `page_facts` (so
`--fact antora.component=fedora-docs`): the core names no profile's facts
(D32, D38). Output is a text table by default; `--format json` is one array
of objects per report with stable field names, documented in the manual.
Rejected: `--component`, which puts Antora in the core; filters declared
by a profile on D38's Views hook, until one needs more than equality.

### What each report holds

Settled 2026-10-19. Text output sorts worst first, and `--top N` limits any
report.

- **`broken`**: a row per link whose outcome is `broken_url` or
  `broken_anchor`: source page, raw href, reason, `source_repo` and
  `source_path`. `kind = anchor` by default (D13); `--kind` widens it.
- **`errors`**: the same shape, for `error`.
- **`stats`**: a row per crawled page with its links, `broken_url`,
  `broken_anchor` and `error` counts, and a run totals line.
- **`inbound`**: crawled pages ranked by the distinct other pages linking
  to them by an `anchor` link; self-links and the `sitemap` kinds do not
  count.
- **`sitemap`**: every crawled page, with title, `in_sitemap`, `via_links`
  and source.
- **`orphans`**: pages in their root's sitemap that no other page links to
  by an `anchor` link.
- **`unlisted`**: crawled pages absent from the sitemap of a root that read
  one; a root with no sitemap is listed as having none (D47).
- **`diff`**: links, keyed by source, target and fragment, broken in `--to`
  and not in `--from` ("new"), and the reverse ("fixed").

### A page's root is recorded

Settled 2026-10-19. **`pages.root`** is the root URL whose scope claimed the
page, NULL for an allowlisted page, and **`runs.roots`** lists the run's
roots, so a root that crawled nothing still shows as having no pages. Both
are written where the frontier claims, which knows the root, and are part
of schema 3. Before them the DB stored no roots, and an allowlisted page
was told from a crawled one only by having no links, as a crawled page
with none also has. Rejected: a `run_roots` table with a `pages.crawled`
flag, leaving a page's root to be matched by scope in SQL, which `include`
globs make unreliable; inferring it from links.

### The redirects report

Settled 2026-10-19. **`redirects` lists every link whose target was
redirected, grouped by landing URL** with a count, each group showing its
chain (3xx or refresh hops); `--format json` adds the linking pages. Nothing
is filtered: a same-host filter would drop cross-host moves (pagure.io to
the forge), the likeliest tasks, and an auth-wall heuristic surprises.
**A page whose canonical names another URL is not in it**: none was seen on
docs.fp.o, and it is a page's property, not a link to fix. Rejected: a
same-host filter; an auth-wall heuristic.

## Which run a report reads, and what `prune` deletes

Settled 2026-10-19.

- A report reads **the latest finished run** by default. `--run N` names
  any run, an unfinished one included, with a note on stderr that it is
  incomplete.
- `diff` compares **the latest two finished runs** by default; `--from N
  --to M` names them.
- `prune --keep N` keeps the N latest finished runs and deletes every
  older run, finished or not. `prune --older-than DUR` deletes the runs
  started before the cutoff. **An unfinished run newer than the latest
  finished one is kept** until a finished run passes it, so the last attempt
  stays inspectable. A `CHECKPOINT` follows the deletes, so the space is
  reclaimed. `prune` with neither flag is a usage error.
- DuckDB allows one writing process per file (D28), so `prune` never runs
  beside a crawl, and an unfinished run it finds is dead.

## The forge, and links skipped by config

Settled 2026-10-19. **The en-US crawl allowlists the forge**: its edit and
history links are what D17's tie-back exists for, and a moved source file
shows only if they are checked. **A config `skip` list of globs**, matching
the whole canonical URL as `include` does (D44) and applying to any host,
makes a matching link `unchecked`, reason `skipped`, with no request. The
guide's worked example skips the forge's `issues/new`, which anonymously
answers 303 to `/user/login` whether or not the repo exists (below), so it
is no longer a false `ok`. That takes the forge from four requests a page
to two (about 77 minutes at its `Crawl-delay: 1`, beside docs.fp.o's 40),
and D46's memo stays unbuilt. It is new crawl behaviour, so a slice ahead
of the crawl, with a `D<k>` entry. Rejected: login detection from a 401 or
a redirect to a login-like path, a heuristic that misfires on other sites;
learning a pattern from the first few URLs to land on a login; allowlisting
with neither, at over 2 hours; not allowlisting the forge.

**The frontier asks `skip` before scope and the allowlist**, so a skipped
URL is `skipped` even inside a root's scope on its own host, **and a
redirect hop onto a skipped URL ends the chain there**, `unchecked` /
`skipped`, a sitemap's hop included, as a hop robots.txt forbids does: the
deployer said not to ask, and the key means that everywhere. So the guide
also skips the forge's `/user/login*`, marking any link that redirects to
it whatever the page.

## The deployment guide and the sim

Settled 2026-10-19. P4 writes **`docs/manual/deploy.md` as far as its
config section**: docs.fp.o's worked example, with its roots, allowlist,
the forge's `skip`, `[host]` rates and `user_agent`. **`mise run sim`**
extracts the guide's tagged blocks into a gitignored `sim/` and checks that
the config loads (D42). **The crawl is not part of `mise run sim`**: at 1 to
2 hours it is launched by hand from `sim/` with `setsid` (`CLAUDE.md`,
"Long-running processes"), the command given in the guide. The scheduled
crawl and the baseline sections are P6's. Rejected: running the crawl in
`mise run sim`, which makes every sim a two-hour job.

## Where the code lives

Settled 2026-10-19, on the concept map (`../design/concepts.md`):

- **`link_verdicts` is `store`'s**, in a new `store/views` child: P5's CI
  gate reads it as `report` does, so it sits below both (D39).
- **The report views are `report`'s**, which becomes `report/`: the views,
  their rendering and choosing the run (above). `cli` hands them to
  `Store::open` beside the profiles' views, so every open installs every
  view with `CREATE OR REPLACE` and an ad-hoc `duckdb` user sees them after
  a crawl alone (D6).
- **`prune`'s deletes are a `store` method**; its command line is
  `cli/prune`, calling `store` directly.
- **`skip` is `config`'s**: a top-level `Patterns` compiled at load, so a
  bad glob is refused there, as scope is (`Root::in_scope`, D44).

Rejected: every view in `store`, which puts report logic there; `report`
installing its views only when `bruce report` runs, which hides them from
`duckdb` after a crawl.

## What the en-US crawls show

Settled 2026-10-19. P4 crawls en-US twice from `sim/`: **P4.4** with the
schema and `skip` in place, so every later slice builds against the real
site's shapes, and **P4.10** with the finished tool, so the pair is `diff`'s
real evidence and P4.10's reports are the phase's deliverable: the sitemap,
the inbound ranking and both broken stats. **A crawl passes when it
finishes** (`finished_at` set, exit 0) **with every sitemap page fetched**;
the rest is recorded, not judged: wall clock, peak RSS, DB size, the
outcome and reason counts, and `link_verdicts`' timing, as in P3.8's table
(`roadmap-P3-crawler-notes.md`). A wall clock or RSS well past the
estimates (77 minutes; about twice P3.8's 55 MiB) is a finding for the
notes, not a stop. **Broken links found on docs.fp.o are evidence only**:
reporting them upstream is the deployer's (D41). Rejected: one crawl at the
end, which builds every report on wiremock data and meets the site's shapes
last.

## Facts found while grilling

- `report.rs` holds only its module doc. The one view is the antora
  profile's `antora_pages` (`src/profile/antora.rs`, `PAGES`), and
  `Store::open` installs every profile's views with `CREATE OR REPLACE VIEW`
  at each open (`src/store.rs`), so an older DB gets the opening build's
  views.
- There is no `mise run sim` task and no deployment guide: `docs/manual/`
  holds `config.md` and `crawl.md`. D42's machinery starts in P4.
- D20's `unresolved_xref` is unbuilt: the crawl keeps `links.xref_refspec`
  (`src/crawl.rs`), and placing it is the join's.
- In P3.8's sitemap-set run (`runs/p3.8-live-20261018/live-sitemap.duckdb`)
  every redirected link, 26 of 26, is a forge `issues/new?title=…` link
  answering 303 to `/user/login`, a distinct URL per page; no docs.fp.o
  link was redirected. All 24 canonicals name their own page. Of 5402 links,
  4948 are anchors and 173 carry a fragment, 152 of those same-page.
- Probed 2026-10-19 with `curl` (no redirect following): a forge
  `issues/new` answers 303 to `/user/login` **for a repo that does not
  exist as well as for one that does**, so anonymously that link tells
  nothing; a missing file under `src/branch/main/` answers 404.

## The slices

In order; each makes the next one's mistakes visible. Nothing builds or
tests while a crawl runs (`CLAUDE.md`, "Long-running processes"), so the
slices after a launch land in later sessions.

### P4.1 Schema 3

Drop `links.outcome`, `reason` and `redirected`; add `pages.root` and
`runs.roots` (above). Closes KD3.

### P4.2 `skip`

The config key, the frontier's `skipped`, and its `D<k>` entry.

### P4.2.1 A skipped hop ends the chain

Earned: P4.2 followed a redirect onto a skipped URL. The hop ends the
chain `unchecked` / `skipped` (above), whatever the target's plan, a
sitemap's included, amending D50.

### P4.3 The deployment guide's config section and `mise run sim`

`docs/manual/deploy.md` as far as its config, docs.fp.o's worked example
allowlisting the forge and skipping `issues/new` and `/user/login*`, and the task extracting
it into `sim/` (D42).

### P4.4 The first en-US crawl

Launched detached from `sim/`; the notes name its log and DB under
`runs/` and record what "What the en-US crawls show" asks.

### P4.5 `link_verdicts`

The view in `store/views`, with the rules above, tested on fixtures and
wiremock and timed on P4.4's DB.

### P4.6 `bruce report`

Choosing the run, `--repo`, `--fact`, `--top`, `--kind`, both formats, the
`report/` views installed at every open, and `broken`, `errors` and
`stats`; the manual's reference for it.

### P4.7 The other single-run reports

`inbound`, `sitemap`, `orphans`, `unlisted` and `redirects`.

### P4.8 `diff`

### P4.9 `prune`

### P4.10 The second en-US crawl

With the finished tool; its reports, and `diff` against P4.4's run, are
the phase's deliverable.
