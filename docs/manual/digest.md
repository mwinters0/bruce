# `bruce report digest`

```sh
bruce report digest --out DIR [--name STEM] [--db FILE] [--run N] [--from N | --since FILE|URL] [--format json|html|both]
```

Writes one **digest** of a crawl run: the run's figures, the broken links
ranked so that the fix removing the most links comes first, and the housekeeping
a maintainer tidies, in files named for the day the run started: an
[HTML document](#the-html) to read and [JSON](#the-json) to load. It is the
report to keep: a crawl database is pruned ([`bruce prune`](prune.md)) and
truncated, and the digests it leaves behind hold every broken and error row,
so a database loading them can say how long a link has been broken after the
runs that knew are gone.

The digest reads a database that [`bruce crawl`](crawl.md) wrote, as
[`bruce report`](report.md) does, and counts the links on the pages a root
covers, of every kind. A page fetched only because another page linked to it
is read for its anchors and holds no links. A sitemap's entry is in no section
but the redirects, and the housekeeping's list of the entries no page was
fetched for.

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `--out DIR` | required | The directory the files are written to, created where it is missing. |
| `--name STEM` | `bruce` | The file name's first part, so the digests of an en-US crawl and an every-locale crawl share a directory. It holds no `/`. |
| `--db FILE` | `bruce.duckdb` | The database to read. It must exist. |
| `--run N` | the latest finished run | The run to digest, as [`bruce report`](report.md#which-run) chooses it. A run that never finished is digested with a warning on stderr. A run that [`bruce check-dir`](check-dir.md) made has no digest. |
| `--from N` | the finished crawl run before `--run` | The run to [compare with](#the-comparison): a crawl run of the database, finished or not. |
| `--since FILE\|URL` | none | Compare with the run of a baseline file, as [`bruce check-dir`](check-dir.md)'s `--baseline` takes one: a path or an `http(s)` URL, gzipped or not. It holds one finished run, which [`bruce export-baseline`](export-baseline.md) writes. Not with `--from`. |
| `--format` | `both` | `json`, `html` or `both`: the files to write. |

## The comparison

The digest compares its run with another, to say what changed: the header names
that run, each figure shows its change since it, and the breakage new and the
breakage fixed since are sections, each left out where it would list nothing. **By default** the other run is the finished crawl run before the
digest's in the database; a `check-dir` run is never one. **`--from`** names
another run of the database. **`--since`** names a baseline file, for a
deployment that keeps no history: the digest imports its run, reads it, and
removes it again, so the database holds the runs it held before and a bare
[`bruce report`](report.md) still reads its latest finished run. A digest of a
database that holds no earlier run, given neither, says so in the header, has
neither section, and has `null` for `compared_with`. The comparison is of two runs
of one site: a run of other roots shows as pages gained and lost.

## The files

`DIR/STEM-YYYY-MM-DD.json` and `DIR/STEM-YYYY-MM-DD.html`, the date being **the day the run started, in UTC**,
so a digest made again later is named for the same day. A second run on the
same day overwrites the first's files: the files are the day's report, and the
`run` inside them says which run that was. `bruce` prints the path of each file
it wrote.

## The HTML

One page with no script and no outside file, so it reads the same opened from
a disk, from an archive or from a web server. It is the JSON's data laid out
for a reader, and is dark on a screen and light in print, where every
`<details>` is opened. At a phone's width its tables scroll sideways. Its
times are UTC and its links absolute, those into the page excepted. The
header and the figures are open; every section after them is closed under its
heading, a click opening it. A dot marks each finding, a row that names a
link or a target, in a column with no heading, or first in a drill-down's
summary; a heading has none. It is red
and filled for a link to a broken URL or one that errored, amber and filled for
a link to a missing fragment or a redirect to update, and a grey ring for a
redirect that differs from its landing only by a final `/`. The ring differs
from the others in shape, so the three read apart in print and without
colour, and every dot is named for a screen reader. The figures' legend names
the three. Only the JSON's `outcome` and `reason` decide a level; it has no
field for it. The lists that are not links (orphans, unlisted pages, sitemap
entries, the unchecked counts) have no dot. **A bar at the left edge** marks a
row or a drill-down that is new since the compared run, as the JSON's `new`
says ([below](#the-json)), and a drill-down that is not new counts its new
links in its summary, "● 4 broken links, 1 new"; the figures' legend says
so. A digest with no compared run marks nothing, and the unchecked counts
and the hosts checked are never marked, nor the two sections that are all
new or all fixed. In this order:

1. **The header**: under the title, the run [compared with](#the-comparison)
   as the day it started and its number, "Compared with 2026-10-22 (run #4)",
   the time too where another crawl run of the database started that day, a
   baseline file's run as "(baseline file)", and "never finished" added
   where it did not; or that there was none. Then the roots, the run's id,
   start, finish and duration, and the `bruce` that crawled it. Its links go
   to each section's heading.
2. **Figures**: the pages checked and the links on them, and among those
   links the ones to a broken URL, in red, to a missing fragment, in amber
   as their dots are, and that errored.
   The first two of those are the broken links. Beneath each is its change since
   the compared run, as `+12`, `−3` or `no change`, in red where what is
   broken rose and green where it fell.
3. **Broken since the previous run**: the links broken in this run and not in
   the compared one, each group under its cause ([below](#why-a-link-is-newly-broken)) a
   closed drill-down of the links, with the page each is on, what it is written as
   and names, and the source of the page answering for it (the target's, as the compared
   run recorded it, for a target removed, moved or without the anchor, else the linking
   page's). Left out, with its link in the header, where no link broke since,
   or there was no compared run.
4. **Fixed since the previous run**: the links the compared run had broken that are
   not broken in this one, each with what it is now and the linking page's source.
   Left out, as section 3 is, where none was fixed.
5. **Pages, by the broken links on them**: each page that holds one, or a
   link that errored, as a drill-down of those links, with its live page and
   its source. Its summary reads, for a page of 181 links to a broken URL or
   that errored and 2 to a missing fragment, "● 181 broken links, ● 2 missing
   fragments", the dots red and amber, a level with no links left out. The pages rank by their red links, then their amber ones,
   and the drill-down lists the red before the amber. A link is counted on the
   page it is written on. A link to a URL whose path doubles a slash is red,
   reason `doubled_slash`, whatever that URL answered and whether it was
   asked, and the page that writes it also counts, as "● 579 broken through
   it", the broken relative links (`../`, `committee/`, `?query`) on the
   doubled-slash pages the link reaches, which the doubled slash breaks, each
   page once and none that is broken on the page's twin too: "● 1 broken
   link, ● 579 broken through it", ranked with the red. A
   page at such a URL counts none of its links where the URL with its
   slashes collapsed is a page of the run, and else all but its broken
   relative ones. The doubled slash is marked in the link, and where it is
   the URL a redirect lands on that doubles it, not the link's, that landing
   is under *Resolves to*, "→ lands on fedoraproject.org//".
   A link to a URL that answered 404 where the same URL with its final `/`
   added or dropped answers 2xx (asked once, for its status alone) is
   broken all the same, reason `wrong_trailing_slash`, and *Resolves to*
   names that URL, "→ try docs.example.org/page/". Where a redirect lands
   on the 404, the landing is under *Resolves to* with its final `/` marked,
   "→ lands on docs.example.org/page/", and the URL named is the landing's
   with its final `/` toggled.
   Its *Written
   as* is the link as the page has it, *Resolves to* the URL it names, *Kind*
   the element holding it (`anchor` is an `<a href>`, as [`bruce report`](report.md)'s `--kind` names it),
   and *Problem* and *Status* what the target answered: `URL broken` or
   `fragment missing`. A page with an unresolved `xref:` that looks like a
   typo has a Hint column, the nearest page ID.
6. **Repos, by the broken links on their pages**: the repos whose pages hold
   broken links, ranked and counted as the pages are, each with links to its pages in section 5,
   which open that section where the browser opens a closed section for a
   link into it: Chrome 97, Firefox 139 and Safari 26.2 on. In an older one,
   open the pages first. A link counts against the repo of the page it is
   written on, which may not be the repo section 7 names for its target.
7. **Broken targets, by the links to them** ("fix once, fix many"): the
   targets that more than one broken link names, from whatever pages, the
   most links first, each with the pages linking to it and the target's own
   source, where restoring a missing fragment mends every link in its row.
   The targets one link names are not listed; their links are in section 5.
8. **Links that errored**, apart from the breakage.
9. **Housekeeping**: the redirected links by where their targets land, each
   landing a closed drill-down of its own (its dot, the landing, its links and
   redirecting URLs, and inside the hops the first took and the pages the links are
   written on, listed one to a line so a phone has no wide table to scroll), those
   differing from their landing only by a final `/` last, the sitemap entries no page was fetched for, the orphans, the unlisted pages
   and the unchecked links, each closed in its turn. The unchecked links are
   listed by reason, each with what its links are in plain words, the setting
   in the [configuration](config.md) that would check them where there is
   one, and where its links point most: the hosts, the parts of a root's
   host, or the schemes, the first five and a count of the rest. After them
   are the hosts no root starts on whose links were checked, the allowlist as
   far as the pages link to it. [Why a link is unchecked](#why-a-link-is-unchecked)
   says what each reason means precisely.

The JSON holds every row the HTML lists and the targets it leaves out, and
the by-component counts it has no section for.

## The JSON

One object. Every time is UTC, an RFC 3339 string. A missing value is `null`,
and a count or status a number.

```json
{
  "format_version": 1,
  "tool_version": "0.1.0",
  "run": {"id":1,"tool_version":"0.1.0","started_at":"2026-10-23T20:04:11Z","finished_at":"2026-10-23T21:14:44Z","duration_seconds":4233,"roots":["https://docs.example.org/en-US/"]},
  "compared_with": {"id":0,"from_file":false,"tool_version":"0.1.0","started_at":"2026-10-22T20:03:58Z","finished_at":"2026-10-22T21:12:09Z","roots":["https://docs.example.org/en-US/"],"totals":{"pages":5,"links":9,"broken_url":2,"broken_anchor":0,"error":1,"unchecked":1}},
  "totals": {"pages":5,"links":10,"broken_url":3,"broken_anchor":1,"error":1,"unchecked":1},
  "broken_targets": [{"target":"https://docs.example.org/en-US/b/","landing":null,"fix":null,"fragment":"nope","outcome":"broken_anchor","reason":null,"status":200,"links":1,"pages":1,"source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/b.adoc","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/b.adoc","new":true}],
  "pages": [{"page":"https://docs.example.org/en-US/a/","title":"A","source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/a.adoc","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/a.adoc","links":6,"broken_url":2,"broken_anchor":1,"error":1,"through":0,"new":false,"new_links":1}],
  "broken": [{"page":"https://docs.example.org/en-US/a/","raw_href":"../b/#nope","target":"https://docs.example.org/en-US/b/","landing":null,"fix":null,"fragment":"nope","kind":"anchor","outcome":"broken_anchor","reason":null,"hint":null,"status":200,"new":true}],
  "by_repo": [{"repo":"https://forge.example.org/docs","pages":2,"broken_pages":1,"links":6,"broken_url":2,"broken_anchor":1,"error":1,"through":0,"new":false,"new_links":1}],
  "by_component": [{"component":"handbook","pages":2,"broken_pages":1,"links":6,"broken_url":2,"broken_anchor":1,"error":1,"through":0}],
  "new_breakage": [{"page":"https://docs.example.org/en-US/a/","raw_href":"../b/#nope","target":"https://docs.example.org/en-US/b/","fragment":"nope","kind":"anchor","outcome":"broken_anchor","reason":null,"status":200,"cause":"anchor_removed","source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/b.adoc","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/b.adoc"}],
  "fixed_breakage": [{"page":"https://docs.example.org/en-US/a/","raw_href":"../old/","target":"https://docs.example.org/en-US/old/","fragment":null,"kind":"anchor","outcome":"broken_url","reason":null,"status":404,"became":"ok","source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/a.adoc","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/a.adoc"}],
  "anchors_removed": [{"page":"https://docs.example.org/en-US/b/","anchor":"nope","links":1,"pages":1,"linking_pages":["https://docs.example.org/en-US/a/"],"source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/b.adoc"}],
  "errors": [{"page":"https://docs.example.org/en-US/a/","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/a.adoc","raw_href":"https://slow.example.net/","target":"https://slow.example.net/","fragment":null,"kind":"anchor","reason":"server_error","status":503,"new":false}],
  "housekeeping": {
    "redirects": [{"landing":"https://docs.example.org/en-US/b/","links":1,"urls":1,"chain":"301 https://docs.example.org/en-US/old/","reason":null,"pages":["https://docs.example.org/en-US/a/"],"new":false}],
    "orphans": [{"page":"https://docs.example.org/en-US/c/","title":"C","root":"https://docs.example.org/en-US/","source_repo":null,"source_path":null,"new":false}],
    "unlisted": [{"page":"https://docs.example.org/en-US/d/","root":"https://docs.example.org/en-US/","title":"D","source_repo":null,"source_path":null,"new":false}],
    "sitemap_entries": [{"page":"https://docs.example.org/en-US/gone/","outcome":"broken_url","reason":null,"status":404,"new":false}],
    "roots_without_sitemap": ["https://docs.example.net/"],
    "unchecked": [{"reason":"skipped","links":1,"targets":1}],
    "unchecked_groups": [{"reason":"skipped","group":"forge.example.org/…/issues/new","links":1,"targets":1}],
    "checked_hosts": [{"host":"slow.example.net","links":1,"targets":1}]
  }
}
```

`format_version` changes only when a field is renamed, removed or given
another meaning, never when one is added, so a loader reading version 1 may
ignore a field it does not know. `tool_version` is the `bruce` that wrote the
file, and `run.tool_version` the one that crawled. `compared_with` is the run
this one was compared with, or `null` where there was none.

**What is new since it**: each row of `broken_targets`, `pages`, `broken`,
`by_repo`, `errors` and the housekeeping's `redirects`, `orphans`, `unlisted`
and `sitemap_entries` has `new`, true where `compared_with`'s same list had
no such row: the same page, target and fragment, or for a broken link, one
`new_breakage` lists. A repo is new where its pages hold a broken link or an
error and held none then. `pages` and `by_repo` also have `new_links`, the
new rows of `broken` and `errors` on the page or the repo's pages. Both are
`null` where `compared_with` is, nothing being known to be new.

Every list below is **already ranked**; a loader needs to sort none of them.

| Field | Holds |
|---|---|
| `run` | The run digested: its `id`, start and finish (`finished_at` and `duration_seconds` are `null` for a run that never finished) and the `roots` it crawled. |
| `compared_with` | The run compared with: its `id` (`null` for a baseline file's run, `from_file` true, whose number is that of a run the database held only while the digest was made), its start and finish (`finished_at` `null` for a run that never finished), the `bruce` that crawled it, its `roots` and its `totals`, which a loader subtracts to give a figure's change. |
| `totals` | The pages the roots cover and the links on them, and the links by outcome, over every kind. `unchecked` links are not broken: `housekeeping.unchecked` says why. |
| `broken_targets` | Each target a broken link names, with its `fragment` when the link names one, the target named by the most `links` first, then by `pages`: one anchor restored in the target's repo removes that many links. `outcome` is `broken_url` or `broken_anchor`, and `reason` and `status` are the target's answer. `landing` is, for a target whose `reason` is `doubled_slash` and whose own URL doubles no slash, the URL it redirected to, which does, and for a target whose `reason` is `wrong_trailing_slash` and which redirected, the URL it landed on; else `null`. `fix` is, for a target whose `reason` is `wrong_trailing_slash`, its URL (its `landing` where it has one) with the final `/` added or dropped, which answered when the crawl asked it; else `null`. `source_repo`, `source_path` and `source_url` are those of the target page, where the run holds it: the repo that would restore a missing anchor. |
| `pages` | The pages that hold a broken link or one that errored, the most `broken_url` plus `error` plus `through` first, then the most `broken_anchor`. `links` counts every link on the page that counts, `error` its links that could not be checked, and `through` the broken relative links (a `?query` is one) on the pages at doubled-slash URLs that its `doubled_slash` links reach, each page once, less those broken on the page's twin too (zero for nearly every page). A page at a doubled-slash URL counts no link where the URL with its slashes collapsed is a page of the run, and else all but its broken relative links. `source_url` is the forge's view of the page's file. |
| `broken` | Every broken link on the pages of `pages`, a page's links together and the pages in that order; a link that errored is in `errors`, where the HTML's drill-down finds it. `page` joins to `pages`. `hint` is, for an Antora `xref:` that names no page, the nearest page ID in the run, as `component:module:path` with an empty module for `ROOT`, where one is close enough to be a typo of it, else `null`. `landing` and `fix` are as in `broken_targets`. |
| `by_repo` | Each repo the pages were built from, ranked as `pages` are, `through` the sum of its pages'; `repo` is `null` for the pages that record none. `pages` counts the repo's pages, `broken_pages` those holding a broken link or an error. |
| `by_component` | The same by Antora component. A site with no Antora page has none. |
| `new_breakage` | The links broken in `run` and not in `compared_with`, a link to a row: `outcome` is `broken_url` or `broken_anchor`, and `cause` is what the compared run held, [below](#why-a-link-is-newly-broken), the rows in that order and then `broken_url` first. `source_repo`, `source_path` and `source_url` are those of the page answering for it: the target page's, as the compared run recorded it, for `target_removed`, `target_moved` and `anchor_removed`, else the linking page's. Empty where `compared_with` is `null`. |
| `fixed_breakage` | The links broken in `compared_with` and not in `run`, as `new_breakage`'s rows without a `cause`, with what each is in `run` as `became`: `ok`, `error` or `unchecked`; `link_removed`, the page no longer holding it; `page_removed`, the page not among the pages the roots cover; or `uncounted`, held on a page whose twin at a URL without the doubled slash answers for its links. The source is the linking page's, `run`'s where it holds the page, else the compared run's. Empty where `compared_with` is `null`. |
| `anchors_removed` | The pages of `run` that lost an anchor since `compared_with`, links still naming it, one row to a `page` and `anchor`, the most `links` first and then the most `pages`: the links to it, the distinct pages they are on and those pages as `linking_pages`, over every kind of link. A page that landed a redirect is the page it landed on. Empty where `compared_with` is `null`. |
| `errors` | Every link whose outcome is `error`, apart from breakage: the target could not be asked, not necessarily wrong. `source_url` is the linking page's. |
| `housekeeping.redirects` | The redirected links by landing URL, the most `links` first: `urls` distinct URLs reached it, `chain` is the hops the first took, and `pages` the pages whose links they are. `reason` is `pedantic_trailing_slash` on a landing whose links name a URL that differs from it only by a final `/`, which comes after every other row, else `null`: such a landing has two rows where other links reach it too. |
| `housekeeping.orphans` | The pages a sitemap lists that no other page links to. |
| `housekeeping.unlisted` | The pages absent from the sitemap of a root that read one. |
| `housekeeping.sitemap_entries` | The sitemap entries no page was fetched for, those whose `outcome` is not `ok` first: what the URL answered. Antora writes a sitemap from what it publishes, so such an entry is a build or deploy fault. |
| `housekeeping.roots_without_sitemap` | The roots that read no sitemap, whose pages cannot be unlisted. |
| `housekeeping.unchecked` | The `unchecked` links by `reason` ([below](#why-a-link-is-unchecked)), and the distinct `targets` they name. |
| `housekeeping.unchecked_groups` | The same links by `reason` and by `group`, where they point, every group listed: the reasons in `unchecked`'s order, a reason's groups the most `links` first. A `group` is the scheme, as `mailto:`, for a URL that is not `http` or `https`; for `skipped`, the host and the path's last two segments, as `forge.example.org/…/issues/new`, the part a `skip` glob usually matches; on a root's host, the URL up to the path's first segment, as `https://docs.example.org/de/`, with the `/` after it where the path goes on; and else the host. It is `null` for a link the run stopped before asking about. |
| `housekeeping.checked_hosts` | The hosts no root starts on whose links' targets were asked about, the most `links` first: the [`allowlist`](config.md#allowlist) as far as the pages link to it. The run records no configuration, so a host the allowlist names that no page links to is in no row. |

## Why a link is newly broken

A link is the same link in both runs when its page, its target and its fragment
are. The digest reads the compared run for the first of these that holds, and
never the pages' hashes, which would blame a page for a break its edit did not
make:

| `cause` | The link |
|---|---|
| `target_removed` | Was `ok` in the compared run, and its target now answers with an error, the page it redirects to included. |
| `target_moved` | Was `ok` in the compared run, and its target now redirects somewhere else, where the link breaks. |
| `anchor_removed` | Was `ok` in the compared run, whose page at the target had the fragment then and does not now: the links behind a row of `anchors_removed`. |
| `link_added` | Is on a page the compared run checked, which did not hold the link. |
| `new_page` | Is on a page the compared run did not check. |
| `unexplained` | Was in the compared run, which judged it neither `ok`, `error` nor `unchecked`, or `ok` where nothing the comparison held explains the break, usually a rule that judges it differently since, as the doubled slash. |
| `first_judged` | Was `error` or `unchecked` in the compared run, so is no regression: it is judged now. |

## Why a link is unchecked

An `unchecked` link is not broken: the run did not learn whether it is. Its
`reason` says why, as the HTML says in plainer words.

| `reason` | The link |
|---|---|
| `out_of_scope` | Names another locale's page: a URL on a host a root starts on, outside every root's scope ([Roots](config.md#roots)), under the path where a link in the language menu of a root's `antora` page leaves that page's own URL, as `/ar/` for the menu's `/ar/a11y/` on `/en-US/a11y/`. It is not requested. Any other URL on such a host outside every scope is asked once, as an allowlisted one is, and judged as any link. |
| `host_not_allowlisted` | Names a URL on a host no root starts on and the [`allowlist`](config.md#allowlist) does not name. It is not requested. |
| `skipped` | Names a URL a [`skip`](config.md#skip) glob matches, or one whose redirect leads to such a URL. |
| `scheme` | Names a URL whose scheme is not `http` or `https` but one a public web page uses: `mailto`, `tel`, `irc`, `ircs`, `xmpp`, `appstream`, `ftp`, `news`, `magnet` or `sms`, or `data` or `javascript`, which work in the page itself, as an embedded image or a script's link. A link with any other scheme, as `file:` or a mistyped `hhttps:`, cannot work for a reader and is `broken_url`, reason `bad_scheme`. |
| `robots` | Names a URL its host's `robots.txt` forbids, where that is obeyed ([Hosts](config.md#hosts)). |
| `robots_unavailable` | Names a URL on a host that answered its `robots.txt` request with no file to read (a server error, a 429 that outlasts the retries, or redirects that lead nowhere), which leaves that scheme and port of the host unrequested for the rest of the run ([Hosts](config.md#hosts)). A host that gives no answer at all has its links `broken_url` or `error` instead. |
| `externals_skipped` | In a `check-dir` run given `--externals skip`, names a URL on an allowlisted host that the baseline has no answer for ([`bruce check-dir`](check-dir.md)). |
| `too_deep` | Is a sitemap that an index lists more than three files below the first, which is not read ([Roots](config.md#roots)). |
| `no_anchors` | Has a fragment, and its target answered with no HTML page read to look for it in: an asset, a page too large to read, or a URL on an allowlisted host first named by a link other than an `<a href>`. |
| `text_fragment` | Has a text fragment (`#:~:text=`), which names words rather than an anchor, and its target answered. |
| `cut` | Names a target the run stopped before asking about. |

## Exit status

| Status | Meaning |
|---|---|
| 0 | The files were written. Finding broken links does not change this. |
| 1 | The database could not be read or has no such run or no finished one, the run or the run compared with is a `check-dir` run, `--since` names no baseline (a file that is not one, or of another schema version), the run a bare digest would read is the baseline of a later unfinished `check-dir` run, or a file could not be written; the message says which. |
| 2 | The command line was not understood. |
