# `bruce report digest`

```sh
bruce report digest --out DIR [--name STEM] [--db FILE] [--run N] [--format json|html|both]
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
| `--format` | `both` | `json`, `html` or `both`: the files to write. |

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
heading, a click opening it. A dot marks the severity of each row that names a
link or a target, in a column with no heading, and each section's heading
carries the dot of the highest severity it holds, so a closed digest can be
skimmed: red and filled for a link to a broken URL or a missing fragment,
amber and filled for a link that errored or a redirect to update, a grey ring
for a redirect that differs from its landing only by a final `/`. The ring
differs from the others in shape, so the three read apart in print and without
colour, and every dot is named for a screen reader. The figures' legend names
the three. Only the JSON's `outcome` and `reason` decide a level; it has no
field for it. The lists that are not links (orphans, unlisted pages, sitemap
entries, the unchecked counts) have no dot. In this order:

1. **The header**: the roots, the run's id, start, finish and duration, the
   `bruce` that crawled it, and the run compared with. Its links go to each
   section's heading.
2. **Figures**: the pages checked and the links on them, and among those
   links the ones to a broken URL, to a missing fragment, and that errored.
   The first two of those are the broken links.
3. **Pages, by the broken links on them**: each page that holds one, the
   most first, as a drill-down of its broken links, with its live page and
   its source. A link is counted on the page it is written on. Its *Written
   as* is the link as the page has it, *Resolves to* the URL it names, *Kind*
   the element holding it (`anchor` is an `<a href>`, as [`bruce report`](report.md)'s `--kind` names it),
   and *Problem* and *Status* what the target answered: `URL broken` or
   `fragment missing`. A page with an unresolved `xref:` that looks like a
   typo has a Hint column, the nearest page ID.
4. **Repos, by the broken links on their pages**: the repos whose pages hold
   broken links, the most first, each with links to its pages in section 3,
   which open that section where the browser opens a closed section for a
   link into it: Chrome 97, Firefox 139 and Safari 26.2 on. In an older one,
   open the pages first. A link counts against the repo of the page it is
   written on, which may not be the repo section 5 names for its target.
5. **Broken targets, by the links to them** ("fix once, fix many"): the
   targets that more than one broken link names, from whatever pages, the
   most links first, each with the pages linking to it and the target's own
   source, where restoring a missing fragment mends every link in its row.
   The targets one link names are not listed; their links are in section 3.
6. **Links that errored**, apart from the breakage.
7. **Housekeeping**: the redirected links by where their targets land, those
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
  "compared_with": null,
  "totals": {"pages":5,"links":10,"broken_url":3,"broken_anchor":1,"error":1,"unchecked":1},
  "broken_targets": [{"target":"https://docs.example.org/en-US/b/","fragment":"nope","outcome":"broken_anchor","reason":null,"status":200,"links":1,"pages":1,"source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/b.adoc","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/b.adoc"}],
  "pages": [{"page":"https://docs.example.org/en-US/a/","title":"A","source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/a.adoc","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/a.adoc","links":6,"broken_url":2,"broken_anchor":1,"error":1}],
  "broken": [{"page":"https://docs.example.org/en-US/a/","raw_href":"../b/#nope","target":"https://docs.example.org/en-US/b/","fragment":"nope","kind":"anchor","outcome":"broken_anchor","reason":null,"hint":null,"status":200}],
  "by_repo": [{"repo":"https://forge.example.org/docs","pages":2,"broken_pages":1,"links":6,"broken_url":2,"broken_anchor":1,"error":1}],
  "by_component": [{"component":"handbook","pages":2,"broken_pages":1,"links":6,"broken_url":2,"broken_anchor":1,"error":1}],
  "errors": [{"page":"https://docs.example.org/en-US/a/","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/a.adoc","raw_href":"https://slow.example.net/","target":"https://slow.example.net/","fragment":null,"kind":"anchor","reason":"server_error","status":503}],
  "housekeeping": {
    "redirects": [{"landing":"https://docs.example.org/en-US/b/","links":1,"urls":1,"chain":"301 https://docs.example.org/en-US/old/","reason":null,"pages":["https://docs.example.org/en-US/a/"]}],
    "orphans": [{"page":"https://docs.example.org/en-US/c/","title":"C","root":"https://docs.example.org/en-US/","source_repo":null,"source_path":null}],
    "unlisted": [{"page":"https://docs.example.org/en-US/d/","root":"https://docs.example.org/en-US/","title":"D","source_repo":null,"source_path":null}],
    "sitemap_entries": [{"page":"https://docs.example.org/en-US/gone/","outcome":"broken_url","reason":null,"status":404}],
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
this one was compared with, and `null`.

Every list below is **already ranked**; a loader needs to sort none of them.

| Field | Holds |
|---|---|
| `run` | The run digested: its `id`, start and finish (`finished_at` and `duration_seconds` are `null` for a run that never finished) and the `roots` it crawled. |
| `totals` | The pages the roots cover and the links on them, and the links by outcome, over every kind. `unchecked` links are not broken: `housekeeping.unchecked` says why. |
| `broken_targets` | Each target a broken link names, with its `fragment` when the link names one, the target named by the most `links` first, then by `pages`: one anchor restored in the target's repo removes that many links. `outcome` is `broken_url` or `broken_anchor`, and `reason` and `status` are the target's answer. `source_repo`, `source_path` and `source_url` are those of the target page, where the run holds it: the repo that would restore a missing anchor. |
| `pages` | The pages that hold a broken link, the most `broken_url` plus `broken_anchor` first. `links` counts every link on the page, `error` its links that could not be checked. `source_url` is the forge's view of the page's file. |
| `broken` | Every broken link on the pages of `pages`, a page's links together and the pages in that order. `page` joins to `pages`. `hint` is, for an Antora `xref:` that names no page, the nearest page ID in the run, as `component:module:path` with an empty module for `ROOT`, where one is close enough to be a typo of it, else `null`. |
| `by_repo` | Each repo the pages were built from, the most broken links first, then by errors; `repo` is `null` for the pages that record none. `pages` counts the repo's pages, `broken_pages` those holding a broken link. |
| `by_component` | The same by Antora component. A site with no Antora page has none. |
| `errors` | Every link whose outcome is `error`, apart from breakage: the target could not be asked, not necessarily wrong. `source_url` is the linking page's. |
| `housekeeping.redirects` | The redirected links by landing URL, the most `links` first: `urls` distinct URLs reached it, `chain` is the hops the first took, and `pages` the pages whose links they are. `reason` is `pedantic_trailing_slash` on a landing whose links name a URL that differs from it only by a final `/`, which comes after every other row, else `null`: such a landing has two rows where other links reach it too. |
| `housekeeping.orphans` | The pages a sitemap lists that no other page links to. |
| `housekeeping.unlisted` | The pages absent from the sitemap of a root that read one. |
| `housekeeping.sitemap_entries` | The sitemap entries no page was fetched for, those whose `outcome` is not `ok` first: what the URL answered. Antora writes a sitemap from what it publishes, so such an entry is a build or deploy fault. |
| `housekeeping.roots_without_sitemap` | The roots that read no sitemap, whose pages cannot be unlisted. |
| `housekeeping.unchecked` | The `unchecked` links by `reason` ([below](#why-a-link-is-unchecked)), and the distinct `targets` they name. |
| `housekeeping.unchecked_groups` | The same links by `reason` and by `group`, where they point, every group listed: the reasons in `unchecked`'s order, a reason's groups the most `links` first. A `group` is the scheme, as `mailto:`, for a URL that is not `http` or `https`; for `skipped`, the host and the path's last two segments, as `forge.example.org/…/issues/new`, the part a `skip` glob usually matches; on a root's host, the URL up to the path's first segment, as `https://docs.example.org/de/`, with the `/` after it where the path goes on; and else the host. It is `null` for a link the run stopped before asking about. |
| `housekeeping.checked_hosts` | The hosts no root starts on whose links' targets were asked about, the most `links` first: the [`allowlist`](config.md#allowlist) as far as the pages link to it. The run records no configuration, so a host the allowlist names that no page links to is in no row. |

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
| 1 | The database could not be read or has no such run or no finished one, the run is a `check-dir` run, the run a bare digest would read is the baseline of a later unfinished `check-dir` run, or a file could not be written; the message says which. |
| 2 | The command line was not understood. |
