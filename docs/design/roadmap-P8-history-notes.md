# P8 — Run-history reports: notes

P8 delivered `bruce report digest`, a crawl run's dated JSON and HTML. It is
compared with the crawl run before it, a named run or a baseline file. The
breakage new and fixed since is attributed by what the comparison held, and
what is new is marked in every section. Beside it P8 delivered `report
changed` and `report anchors-removed`, a hash that records its scope (schema
5), a release body naming its schema change and an unresolved xref's hint.
The two reviews of a real en-US digest also changed how the crawl classifies:
a scheme that cannot work is broken, an out-of-scope URL is asked unless it is
another locale's, the allowlist takes `*.<domain>`, a host giving no response
fails its links, robots.txt is read per origin, and a trailing or doubled
slash is judged. The decisions are D15, D43 and D64 to D75, with D3, D5, D7,
D8, D10, D13 and D16 amended; what a reader sees is `../manual/digest.md`. This keeps
what P9 and P7 inherit, the evidence, the reviews' findings the spec cites,
and what was tried and refused.

## The evidence

| Database | What it holds |
|---|---|
| `runs/p8.1-en-us-20261023-2004/en-us.duckdb` | en-US under schema 5 and P5.2's config: 6,436 pages in 1:10:33, 827 broken links on 324 pages |
| `runs/p8.14-en-us-20261024-1322/en-us.duckdb` | en-US under the deployment guide's config after 8.10 to 8.13 and 8.15: 8,990 pages in 1:11:00 |
| `runs/p8.6-review/` | copies of both, and the digests the first review read |
| `runs/en-us-20261025-1301/en-us.duckdb` | M21's crawl as run 2 (1:15:46), over a copy of 8.14's as run 1: the closing review's digest |
| `runs/p8.9-compare/` | Servo's and Chrome's renders of the fixture's digest |

- **8.14 against 8.1 is reclassification, not drift.** Every 8.1 broken link
  keeps its verdict, and all 671 new breaks are `first_judged` (D74). The
  widened allowlist accounts for most of them: 2,830 targets moved from
  `host_not_allowlisted` to `ok`, 306 to `broken_url` and 48 to `error`.
  Other locales' stray URLs were asked (8.11), 2 links became `bad_scheme`,
  and 529 broken links sit on the three `code-of-conduct//` pages that a
  302 doubling a slash leads into. `report broken` went from 827 links on
  324 pages to 1,973 on 588, and `report errors` from 0 to 90. The wall clock
  did not move: the allowlist's slowest host (lists.fedoraproject.org,
  `Crawl-delay: 10`, 29 minutes) runs beside the crawl under D61.
- **No real pair of runs has yet differed by content.** `changed` and
  `anchors-removed` are empty over 8.1 and 8.14, and `target_removed`,
  `target_moved`, `anchor_removed` and `unexplained` have been read only on
  fixtures (D74, Reopens). The 2026-10-25 crawl against 8.14's has no new
  break, one fixed link, two new errors and 16 new redirect landings.
- **`wrong_trailing_slash` on the 2026-10-25 crawl: 5 targets and 5 links**
  out of 764 broken 4xx targets probed. They are two wiki landings with a
  stray `/` (`fedoraproject.org/wiki/Taskotron/` and
  `…/wiki/Infrastructure/pdc-retirement/`) and the three URLs that redirect
  to them: the full-crawl yield D72's Reopens asks for.
- **The digest's cost**, release builds over 8.14's database: 3.1 s since
  8.18.1, which computes the page counts once (8.5 s before it), and 5.6 to
  7.2 s on the 2026-10-25 crawl with D75's marks. The JSON was 671 KB over
  8.1's run, against the spec's estimate of 300 KB: 494 broken targets and 61
  redirect landings with their linking pages make most of the difference.
  The HTML of 8.14 against 8.1 is 4.0 MB, of which section 3 is 0.47 MB.

## What P9 inherits

- **A baseline crawled before 8.10 and 8.11 holds a bad scheme, or a stray URL
  on a root's host, as `unchecked`.** A `check-dir` built after either calls
  such a link introduced, or under `--externals=baseline` copies the row and
  stays silent. P9 ships the binary beside the baseline it crawled
  (`../wip/roadmap-P9-deployment.md`, "CI fetches the binary from beside the
  baseline"). A deployer pinning a release against a newer baseline, or the
  reverse, meets the mismatch.
- **P9's spec says the stray first segments are "`out_of_scope` today"**
  ("Every locale is one root at the site's top", and its facts' `/` bullet).
  Under en-US's config they are asked now (D10, D67); the every-locale root
  still crawls them. This is for P9's re-grilling.
- **An every-locale digest is unmeasured** (`roadmap-P8-history.md`, "Every
  row, until every locale is measured"), and three things in it may grow:
  - the unchecked groups, about 1,300 rows a run of external hosts, which are
    listed whole;
  - section 3, where `first_judged` may want folding to a count;
  - the reads of `link_verdicts`, which each of thirteen sections makes
    again, 0.2 to 0.45 s each on en-US. The remedy is reading it once into a
    temporary table that the views read by name, a change to how the store
    installs them.
- **`robots_unavailable` now means a host answered with no file** (D8), so a
  run's count should be near zero. On 8.14's crawl it is 1 link,
  `jenkins.fedorainfracloud.org`, whose `/robots.txt` is a 302 onto a name
  that does not answer. A large count means a host failing with 5xx or 429.
  A host that is down at its first request has every link `error` for the
  whole run.
- **`digest --since FILE` imports into the database it reads** (D73). On
  8.14's database the file grew from 33.6 to 55.3 MB at the first import,
  and later imports reuse the freed blocks.
- **pagure.io is a read-only archive**: its 18 `new_issue` targets answer 404
  and are reported broken. src.fedoraproject.org's `new_issue` redirects to a
  login page that answers 200. The guide adds no `skip` for either, and
  robots.txt already keeps Bugzilla's `enter_bug.cgi` unasked.
- **The review tooling** is `mise run crawl | digest | serve` (M21,
  `scripts/review.py`). The digest's golden file, `fixtures/antora/digest.html`,
  is rewritten by `cd scripts && uv run e2e.py --save-digest`. A renderer
  change shows as a diff to read there; a change of layout shows only in a
  render.

## What P7 inherits

Its inbox, `roadmap-P7-rules-inbox.md`, holds both facts: severity
belongs to the profile, and a doubled slash's three possible modes.

## The renders

**Servo (`mise run shot`) is enough for a first look, not to check what the
digest is for** (8.9). Closed, at 1280 and 390 px, its renders agree with
Chrome's headless shell 156 line for line. Opened, Servo differs at a phone's
tables: it breaks at every `/`, scrolls at another column and paints no edge
shadow. It has no paged media, so print cannot be checked in it at all. It
also drops the underscores in an `<h3>`'s `<code>`, and does not reveal a
fragment's closed ancestors. Chrome stays the reference for a phone's tables
and for print:

```sh
T=$PWD/runs/headless-browser-trial
LD_LIBRARY_PATH=$T/archpkgs/root/usr/lib FONTCONFIG_FILE=$PWD/runs/shot/fonts.conf \
  $T/mise-data/installs/http-chrome-headless-shell/156/chrome-headless-shell \
  --no-sandbox --hide-scrollbars --user-data-dir=$T/chrome-profile \
  --window-size=390,5000 --screenshot=$OUT.png file://$PWD/$PAGE
```

- **Print to PDF** by putting `--print-to-pdf=$OUT.pdf --no-pdf-header-footer`
  in place of the last two flags. With no rasteriser installed, read the PDF's
  pages as text: `uv run --with pypdf`, then `PdfReader(f).pages[i].extract_text()`.
- **`FONTCONFIG_FILE` must be an absolute path.** A relative one is "not
  found", and the page renders in a fallback font.
- **`mise run shot PAGE` takes an absolute path too**, the task running in
  `scripts/`.
- **A `--screenshot` of a URL with a fragment** paints blank above the target,
  so read a reveal off `--dump-dom`.
- **A fragment naming an element inside a closed `<details>` opens its closed
  ancestors, not the element itself**, and then scrolls. This is HTML's
  "ancestor revealing algorithm", in Chrome 97, Firefox 139 (bug 1724299) and
  Safari 26.2 (WebKit `23cdbeb3cd`, on by default from `5bc51f2367`). An older
  browser does nothing visible. So the section ids sit on the `<section>`,
  outside its `<details>`, and a nav link lands on the closed heading in every
  browser.
- **Printing opens every `<details>` only through `::details-content`**
  (Chromium 131, Firefox 143, Safari 18.4). An older browser prints them
  closed.

## The reviews

8.6 reviewed the digest of 8.1's run, then of 8.14's, on 2026-10-24, and
approved it at `131225c`. 8.20 reviewed the digest of the 2026-10-25 crawl
against 8.14's, on 2026-10-25, and approved it once findings 1 to 6 had
landed; findings 7 to 9 came after the verdict. Each entry below gives the
maintainer's finding and what it earned. P8's spec, D71, D72, D75 and P7's
inbox cite them.

### 8.6's Finding 1: a count does not say which way it points

The digest did not say whether a count was of links *to* a target or links
*on* a page. On 8.1's run, "Fix once" sends the 7 links to
`fesco/Updates_Policy/#updates-testing-activation` to fesco's file, while
"By repo" counts them against four other repos: 674 of the 828 broken links
name a target in their own page's repo. The figures counted links under
target nouns, and `anchor` meant both the `<a>` element and a missing
fragment. It earned 8.4.2, the wording only; the JSON's names stay (D64).
"By repo" counts by the linking page's repo alone (D64, Rejected).

### 8.6's Finding 2: the sections' order and folding

Every section after the figures is a `<details>` that starts collapsed, in
the order pages, repos, targets, then housekeeping. It earned 8.4.3. The
session placed the rest: the header and figures stay open and first, section 3
comes after them and the errors come before the housekeeping.

### 8.6's Finding 3: the unchecked reasons are unexplained

A reader cannot know what `out_of_scope` means, or which hosts were
allowlisted. The run records neither its allowlist nor its scopes, so the
hosts checked are read off its rows (D64, Rejected). It earned 8.4.4, whose
first draft's definitions the maintainer called confusing and technical: they
said how bruce decides, not what the links are.

### 8.6's Finding 4: some unchecked links are broken

Every unchecked link that cannot work is broken. On 8.1's run these were a
mistyped `hhttps:`, `https:/kubernetes.io` and `https:/docker.io` (which a
browser resolves on docs.fp.o, to a 404) and an href pasted with an ellipsis
and a zero-width space. All of them were `out_of_scope` or `scheme`, because
bruce had no notion of a locale. It earned 8.10 and 8.11. 8.13 came from the
ruling of the same day that a host that does not resolve, or never answers,
must not hide its links as `unchecked`.

### 8.6's Finding 5: the allowlist names the Fedora and Red Hat sites

The allowlist gains the Fedora and Red Hat sites, every subdomain included,
with pagure.io, centos.org and the retired fedorahosted.org. That is 3,521
of 8.1's `host_not_allowlisted` targets, on 161 hosts. It earned 8.12. 8.15
came from the review of 8.13's call.

### 8.6's Finding 6: a trailing slash is pedantic, not broken

A slash the server supplies is `ok`, reason `pedantic_trailing_slash`: 496
links to 259 targets on 8.14's crawl, and 505 by the view, which also
compares URLs with a query. A slash the server does not supply stays broken,
with the fix named. The 529 links on the `code-of-conduct//` pages are one
redirect. It earned 8.16 to 8.18.

### 8.6's Finding 7: severity is marked

A small red, yellow or grey circle in the first column, pedantic the lowest,
with the design left to the session. It earned 8.19.

### 8.6's Finding 8: a dot marks a finding, and a missing fragment is amber

The dots mark individual findings, never headings. `broken_url` and `error`
are red; `broken_anchor` and a redirect that is not pedantic are amber. These
are defaults, each kind's classification being the profile's (P7's inbox).
It earned 8.19.1, 8.19 having shipped the wrong contract.

### 8.6's Finding 9: rows counted by severity; the redirects too wide

A page's and a repo's summary counts its red links, `error` included, and its
amber ones, and ranks by red then amber. A page whose only findings are errors
becomes a row: 35 such pages on 8.14's crawl. The redirects' table was so wide
that a phone scrolled across empty space, so each landing became a
`<details>`. It earned 8.4.5 and 8.4.6.

### 8.6's Finding 10: a doubled slash is the linking page's fault

The top pages were those whose URL doubles a slash, broken through their
relative links, and the page linking to such a URL is the one at fault. The
breakage this causes counts on the linking page's row, the whole chain
included. A doubled-slash page counts nowhere when its twin was reached; when
it was not, it counts its own broken absolute links. It rewrote 8.18; the
modes are in P7's inbox. The spec's 579 links were read before the view
existed, and the view gives 556 (8.18).

### 8.6's Finding 11: a doubled slash a redirect lands on is not shown

Neither *Written as* nor *Resolves to* showed the doubled slash: 35 of 8.14's
98 doubled-slash targets are doubled only by their landing. Where the landing
doubles it, *Resolves to* shows "→ lands on …", and the doubled slash is
marked wherever it is shown. It earned 8.18.2.

### 8.6's Finding 12: a row's summary carries no total

The parenthetical totals go. It earned 8.4.7, reversing that part of
Finding 9.

### 8.20's Finding 1: "Broken since" held the fixed links too

It became two sections, "Broken since the previous run" and "Fixed since the
previous run". It earned 8.8.2.

### 8.20's Finding 2: no section for the anchors removed

The HTML drops the section; the JSON keeps `anchors_removed`, whose links are
section 3's `anchor_removed` rows. It earned 8.8.2.

### 8.20's Finding 3: the compared run is a sub-title

The compared run moves under the title: its day, its time where another crawl
run started that day, and "(run #N)". The session placed the rest: a baseline
file's run reads "(baseline file)" (D73), and "never finished" is kept for a
compared run that did not finish. It earned 8.8.2.

### 8.20's Finding 4: each figure is coloured as its links' dots

The missing fragments' figure is amber and the errored links' figure red. It
earned 8.19.2.

### 8.20's Finding 5: what is new is marked in every section

New entries are marked in every section, taking as little room as possible,
and none on a digest with nothing to compare with. The maintainer chose the
bar, a drill-down counting its new links, and `new` in the JSON. It earned
8.21 (D75).

### 8.20's Finding 6: an empty two-run section is left out

A two-run section with nothing to list is hidden, along with its link in the
header. The session also hid both sections when there is no comparison, the
sub-title saying there was none. It earned 8.8.3.

### 8.20's Finding 7: a figure that did not change says nothing

A figure shows its change only where the change is not zero. It earned 8.4.8.

### 8.20's Finding 8: an errored link is not a broken one

The maintainer asked whether "External links that errored" would be the
right title. It would not: D16's `error` is any host's refusal, 5xx or
silence, docs.fp.o's included, though all 89 errors on the 2026-10-25 crawl
are off the site. The session, the maintainer having handed it the choice,
named an `error` link "could not be verified": it sits between broken
(answered that it is not there) and unchecked (not asked). The JSON's `error`
and the section's `#errors` stay. It earned 8.4.8.

### 8.20's Finding 9: a summary counts its unverifiable links apart

A summary reads "● 3 broken links, ● 1 unverifiable", red with the broken
links and ranked as before. It earned 8.4.9.

## Negative results

### The digest

- **A section is a `Table`**, not a type of its own: the JSON, the HTML and
  the tests all want the column names, and `Table` already prints D6's shape.
- **`serde_json` is a dev-dependency only.** The writer is a few `format!`s,
  and a production dependency would join `THIRD-PARTY-NOTICES.txt` for them.
- **There is no Rust golden file.** The structure tests hold the sections,
  the escaping and the links, and the e2e golden file holds the whole page; a
  second copy would be a second thing to bless. The golden file holds neither
  a marker nor a shadow, both being CSS.
- **Rejected: the reasons' meanings in the JSON**, which are prose for a
  reader. Also rejected: capping the JSON's unchecked groups at the HTML's
  five, which would need a remainder row of another shape; and grouping
  `skipped` by host and first segment, which names a forge's organisations
  rather than what the glob matched.
- **Rejected: showing Kind `anchor` as `<a>` or `hyperlink`**, which would
  part the HTML's word from `--kind`'s and the JSON's. Also rejected: a sixth
  figure for the broken links' total, which the sentence under the figures
  names.
- **Rejected: the section id inside its `<details>`**, which lands nowhere
  in an older browser. Also rejected: `details:has(:target)::details-content`
  as a fallback, which helps only Safari 18.4 to 26.1 and shows a
  `<details>` that a click cannot then close.
- **Rejected: keeping error-only pages out of the pages section**, which would
  leave a repo's red count holding links no listed page shows.
- **Rejected: a table of redirects with the linking pages dropped**, since
  the first hops are as long as the pages. Also rejected: a `<dl>` for the
  hops, which is the header's grid.
- **Joining the `doubled_slash` view to find a landing's doubled slash took
  the digest from 3.76 s to 4.2 s**, and joining every target by URL took it
  to 4.1 s. Joining only the 799 redirected targets that double no slash
  themselves is indistinguishable from the baseline.
- **Computing the page counts once saved only 0.6 s of 5.4 s.** The rest was
  `doubled_slash`'s pattern running over all 173,000 targets for 98 matches.
  A `contains` ahead of the pattern, and a join of the 98 alone, fixed it.
- **A key broken in both runs is no new break**, even where its cause
  changed (`broken_url` to `broken_anchor`), as in `diff`. `fixed_breakage`
  has no cause, its `became` saying what a link is now.
- **`changed` lists no page that is in only one run.** `anchors-removed` has
  no row for an anchor no link names; it reopens for a consumer that wants
  every lost anchor.
- **A `run_roots` table was not made for the hash scopes.** `prune`'s and
  `baseline`'s table lists would all change for it; a list of structs on
  `runs` holds the same rows. `copy_run` names `runs`' columns, so a column
  added to `runs` is added to `store/baseline.rs`' select too.

### Classification

- **The reason for an unworkable scheme is `bad_scheme`, not `scheme`**, so
  that one name never means two verdicts. A diff of two runs then shows
  (`unchecked`, `scheme`) moving to (`broken_url`, `bad_scheme`). A
  configurable list of schemes was rejected; a missing one is added to
  `WORKING_SCHEMES`.
- **Rejected: telling a locale by its segment's shape** (`pt_BR`), which takes
  `f28` or `infra` for one. A waiting URL keeps its first link's kind, as
  KD4 has it.
- **Rejected: glob matching over the allowlist's hosts**, whose cost grows
  with the list; a suffix lookup does not. A host with a trailing dot is not
  below `*.<domain>`.
- **`rtrim(url, '/')` is the wrong slash comparison.** It strips every
  trailing slash, and misses a slash before a query.
- **A same-page `#fragment` on a doubled page is not a doubled-slash link**,
  and a `data:` body's base64 holds `//`. Only `http(s)` URLs are read for a
  doubled slash (D71).
- **No target row for a toggled URL**, and a target redirected before its
  4xx was at first not asked about (D72, Rejected). 8.17.1 asks its landing's
  toggle. Two URLs landing on one 404 make two probes; a memo per landing
  was not worth its shared state for 3 of 34 such targets.
- **A pedantic link with a missing fragment is `broken_anchor`**, and on
  8.14's crawl 8 links go through a slash-only redirect onto a broken
  landing. Only an `ok` link takes the reason.

### Crawl and fetch

- **`Access` moved up into `fetch`**, since `fetch::http::host` naming
  `fetch::Failure` was a module cycle. A failure type of host's own would have
  been a second copy of D16's causes.
- **A loop between two origins' robots.txt would deadlock on their cells.**
  `Reads::waiting` breaks it; a chain-local visited set misses two tasks
  starting at once.
- **No test resolves a name to NXDOMAIN through the crawl**, or serves two
  schemes. The mock servers speak `http` alone, so the origins differ by
  port.
- **A 404's language menu is never read**: the 404 page is `generic`, and its
  body is not extracted.
- **The release body's schema step was never run against GitHub** before the
  next `-rc` tag. `-F body=@file` is `gh api`'s documented behaviour,
  asserted only as text, and `generate_release_notes=true` stays.
