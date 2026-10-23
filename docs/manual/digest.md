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
`<details>` is opened. Its times are UTC and its links absolute, those into the
page excepted. In this order:

1. **The header**: the roots, the run's id, start, finish and duration, the
   `bruce` that crawled it, and the run compared with.
2. **Figures**: the pages and links checked, and the broken URLs, broken anchors
   and errors among them.
3. **Fix once, fix many**: the broken targets that more than one link names,
   the most links first, each with the source of its page, the repo that would
   restore a missing anchor. The targets one link names are not listed; their
   links are in the next section.
4. **Pages by broken links**: each page that holds one, the most first, as a
   drill-down of its broken links, with its live page and its source.
5. **By repo**: the repos whose pages hold broken links, the most first, each
   with links to its pages in section 4.
6. **Errors**, apart from the breakage.
7. **Housekeeping**, closed: the redirected links, the sitemap entries no page
   was fetched for, the orphans, the unlisted pages and the unchecked links.

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
  "broken": [{"page":"https://docs.example.org/en-US/a/","raw_href":"../b/#nope","target":"https://docs.example.org/en-US/b/","fragment":"nope","kind":"anchor","outcome":"broken_anchor","reason":null,"status":200}],
  "by_repo": [{"repo":"https://forge.example.org/docs","pages":2,"broken_pages":1,"links":6,"broken_url":2,"broken_anchor":1,"error":1}],
  "by_component": [{"component":"handbook","pages":2,"broken_pages":1,"links":6,"broken_url":2,"broken_anchor":1,"error":1}],
  "errors": [{"page":"https://docs.example.org/en-US/a/","source_url":"https://forge.example.org/docs/src/branch/main/modules/ROOT/pages/a.adoc","raw_href":"https://slow.example.net/","target":"https://slow.example.net/","fragment":null,"kind":"anchor","reason":"server_error","status":503}],
  "housekeeping": {
    "redirects": [{"landing":"https://docs.example.org/en-US/b/","links":1,"urls":1,"chain":"301 https://docs.example.org/en-US/old/","pages":["https://docs.example.org/en-US/a/"]}],
    "orphans": [{"page":"https://docs.example.org/en-US/c/","title":"C","root":"https://docs.example.org/en-US/","source_repo":null,"source_path":null}],
    "unlisted": [{"page":"https://docs.example.org/en-US/d/","root":"https://docs.example.org/en-US/","title":"D","source_repo":null,"source_path":null}],
    "sitemap_entries": [{"page":"https://docs.example.org/en-US/gone/","outcome":"broken_url","reason":null,"status":404}],
    "roots_without_sitemap": ["https://docs.example.net/"],
    "unchecked": [{"reason":"skipped","links":1,"targets":1}]
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
| `broken` | Every broken link on the pages of `pages`, a page's links together and the pages in that order. `page` joins to `pages`. |
| `by_repo` | Each repo the pages were built from, the most broken links first, then by errors; `repo` is `null` for the pages that record none. `pages` counts the repo's pages, `broken_pages` those holding a broken link. |
| `by_component` | The same by Antora component. A site with no Antora page has none. |
| `errors` | Every link whose outcome is `error`, apart from breakage: the target could not be asked, not necessarily wrong. `source_url` is the linking page's. |
| `housekeeping.redirects` | The redirected links by landing URL, the most `links` first: `urls` distinct URLs reached it, `chain` is the hops the first took, and `pages` the pages whose links they are. |
| `housekeeping.orphans` | The pages a sitemap lists that no other page links to. |
| `housekeeping.unlisted` | The pages absent from the sitemap of a root that read one. |
| `housekeeping.sitemap_entries` | The sitemap entries no page was fetched for, those whose `outcome` is not `ok` first: what the URL answered. Antora writes a sitemap from what it publishes, so such an entry is a build or deploy fault. |
| `housekeeping.roots_without_sitemap` | The roots that read no sitemap, whose pages cannot be unlisted. |
| `housekeeping.unchecked` | The `unchecked` links by `reason`, and the distinct `targets` they name. |

## Exit status

| Status | Meaning |
|---|---|
| 0 | The files were written. Finding broken links does not change this. |
| 1 | The database could not be read or has no such run or no finished one, the run is a `check-dir` run, the run a bare digest would read is the baseline of a later unfinished `check-dir` run, or a file could not be written; the message says which. |
| 2 | The command line was not understood. |
