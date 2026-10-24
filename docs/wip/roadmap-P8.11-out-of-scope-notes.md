# P8.11 — An out-of-scope URL is asked unless another locale's: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"A link that cannot work is broken", its first bullet) and 8.6's Finding 4
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 4"): the `antora` profile's language-menu reader
(`profile::antora::other_locale`), `Sink::other_locale`, `Settled::resolve`
and `Extracted::other_locales`; the frontier's locales, `Plan::Outside`,
`Frontier::learn`, `wait` and `judge_outside`; the crawl's `settle_outside`;
D67, D10 and D3 amended, I25; the digest's `out_of_scope` meaning and the
unchecked fold's opening; the manual (`digest.md`, `config.md`'s Roots,
`check-dir.md`, `deploy.md`); and the e2e site's whole build given
production's language menu for `en-US` and `fr` (`e2e.py`, `LANGUAGES`),
with its golden digest and saved pages written anew.

## Order

A page's links are planned in `Shared::write_page`, after the page is
extracted, so `write_page` hands the frontier the page's locales first: no
link on a page with the menu waits. **The URLs planned before any menu is
read** cannot be judged on the locales then known, and the frontier makes
them wait instead:

- **A root's URL** is in its own scope (D44, refused at load otherwise), so
  never reaches the rule.
- **A page sitemap's entry** outside every scope is dropped before it is
  claimed (D10), so a link meets it later as any URL; an index's entry is a
  sitemap wherever it is.
- **A `check-dir` seed** is a built page; an xref is resolved after the built
  pages are read.
- **A hop** is asked wherever it goes (`hop_plan`), as before.
- **Anything else**: a URL on a root's host outside every scope that no
  locale learned so far covers is claimed as `Plan::Outside` and held by the
  frontier; after the run's last drive (`run`, and `run_seeded` before the
  copies) `settle_outside` judges each by the locales learned by then and
  drives what it asks, looping while those drives read a page a hop landed on
  that names more. The locales only grow, so one covered when met is covered
  at the end, and no verdict depends on which page was read first (D67).
  The `crawl::tests` test reads a menu-less page naming `/fr/docs/x/` before
  the page whose menu names `fr`.

On en-US this holds about 12 URLs, not the 154,569 locale targets: every
Antora page's menu is the same site-wide list (I25), so after the first page
the locales are whole.

## Where the locales live

The profile reads, the crawl decides. `profile::antora` reads the menu's
hrefs in a handler and, in its finish step, resolves each through
`Settled::resolve` (the core's own resolution, `<base href>` and D53
included) and keeps the prefix where the link leaves the page's URL. The
locales leave the pass as `Extracted::other_locales`, and `crawl::frontier`
holds them by host in an `RwLock`, read on every out-of-scope plan and
written only when a page brings a prefix not yet known. Only a page read
whole (`Plan::Page`) teaches it; a one-hop page's links are never planned.
No module was added, and the edges stay one way (`module_graph.py`).

## `check-dir`

A waiting URL is judged by `Frontier::external`, the allowlisted URL's
path: the baseline's rows where it holds the target, else asked, and
`--externals` decides (`crawl::baseline::tests::an_out_of_scope_url_is_answered_as_an_external_unless_another_locales`,
every mode). Another locale's page stays `out_of_scope` whatever the mode.
The locales come from the built pages: a content repo's CI build uses the
UI's own `page-languages.hbs`, which links `en-US` alone (I25), so it learns
none and answers a link to another locale as any other out-of-scope URL,
copying the baseline's `out_of_scope` row where it has one. A hop onto such
a URL is asked as before, even under `--externals=skip`, whose end-of-chain
(D54) stays the allowlisted host's.

## Robots and rate

The fetcher keys robots.txt, pace, in-flight cap and `[host]` settings by
host (`fetch::http::host`, D8), so these requests take the root host's own
rate and robots.txt, and a root host's `robots = false` covers them too. No
code changed for it.

## What the next slices inherit

- **On en-US's 8.1 crawl this asks 12 URLs** (2,387 links) and leaves
  154,569 `out_of_scope` targets, every one under a locale the a11y fixture's
  menu names (`duckdb -readonly runs/p8.1-en-us-20261023-2004/en-us.duckdb`,
  targets with reason `out_of_scope` split by `split_part(url, '/', 4)`
  against those 67 names). The 12, by links: the slashless home link
  `https://docs.fedoraproject.org/en-US` (2,320), the host's `/` (26),
  `fedora-project/project/fedora-overview.html` and
  `http://docs.fedoraproject.org/en-US/index.html` (16 each),
  `/kubernetes.io` (2), and seven with one link each, `/docker.io` and the
  pasted `infra/sysadmin_guide/…​..` among them. 8.6 counted 11 URLs and
  2,371 links, one URL of 16 links fewer. 8.8's second crawl shows them asked, its notes counting
  them as reclassified, not new.
- **The e2e site shows both halves**: its slashless home link
  (`http://127.0.0.1:8099/en-US`, 8 links) moved from `out_of_scope` to the
  housekeeping's redirects (301, then the root's refresh), and the `fr`
  menu link of each of the 8 pages is `out_of_scope`. The gate's sections
  did not change.
- **`crawl.rs` is at 996 code lines and 40 functions**, the soft limits being
  1,000 and 40: the next slice that grows it turns the seams meter red.
- **For 8.12**: `Frontier::plan` tests the allowlist before a root's host, so
  an allowlist entry matching a root's host makes its other locales asked.
  8.12's rule that `*.<domain>` never matches a root's host keeps them
  unasked; a root's host named outright is refused at load (D68).
- **A baseline crawled before this rule** holds these URLs as `unchecked`,
  so a `check-dir` under `--externals=baseline` copies that row and the gate
  stays silent where the URL is broken, until the baseline is crawled anew.
  P9 ships the binary beside its baseline, as 8.10's notes say for schemes.
- **P9's spec says the stray first segments are "`out_of_scope` today"**
  (`roadmap-P9-deployment.md`, "Every locale is one root at the site's top",
  and its facts' `/` bullet). Under en-US's config they are now asked; the
  every-locale root still crawls them. Left to P9's grilling, its spec not
  being this slice's to edit.

## Negative results

- **A waiting URL keeps its first link's kind** (KD4), though it waits long
  enough for an anchor link's to be known; KD4's remedy is the crawl-wide
  one its entry names, not one for this class alone.
- **The 404's menu is never read**: the page has no generator meta, so it is
  `generic`, and a 404's body is not extracted. Its root-relative hrefs
  (`/en-US/../ar`) still give `/ar/` should a profile read them.
- **Rejected: telling a locale by its segment's shape** (`ar`, `pt_BR`,
  `zh_Hant`), which would take `f28` or `infra` for one and miss a locale
  named otherwise; the menu says which exist.
