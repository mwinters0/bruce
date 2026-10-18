# `bruce crawl`

```sh
bruce crawl [URL] [--config FILE] [--db FILE]
```

Crawls a site and adds one **run** to a database: every URL the site's pages
and sitemaps name, whether it answered, and each page's title, anchors and
links. A run does not judge links; the reports read that from the database.

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `URL` | none | The site to crawl, an `http` or `https` URL. |
| `--config FILE` | none | A configuration file ([`config.md`](config.md)). |
| `--db FILE` | `bruce.duckdb` | The database the run is added to, created if it does not exist. |

**Where the crawl starts** depends on which of the two you give:

- **A URL alone**: that URL is the only root, every setting at its default.
- **A file alone**: the file's `[[root]]` tables.
- **A file naming no `[[root]]`, and a URL**: the URL is the only root, and
  the file's other settings apply, its rates, hosts and allowlist.
- **A file naming roots, and a URL**: an error. Add a `[[root]]` for the URL
  instead, so its settings are written down.
- **Neither**, or a file naming no root and no URL: an error.

A root's `profile` must name a profile `bruce` has; the error lists them.

A database belongs to one version of `bruce`. One written by another version
is refused, and the message names both versions; crawl into a new file.

## Output

The run's number and what it wrote are printed when it ends:

```text
run 3: 2412 targets, 2305 pages, 69 sitemap files
```

`no sitemap` there means no sitemap file was read, none being found or the
one found unreadable, and every page was found by following links. Sitemap entries outside every root are counted and dropped.

## Stopping a crawl

**Ctrl-C once** stops the crawl's requests within 30 seconds:

```text
bruce crawl: stopping: finishing requests in flight (30 s at most); Ctrl-C again to abandon them
```

No new request is sent, not even a retry or the next step of a redirect.
A request already sent may finish, and what it answered is written; a URL
still unanswered after 30 seconds is left out. The run is left marked
unfinished, and its line counts the URLs it found but did not write:

```text
run 3: 1210 targets, 1187 pages, 69 sitemap files, 1102 cut; stopped before the end, so the run is left unfinished
```

**Ctrl-C again** gives up on the requests still out at once. A URL whose
answer is already being written to the database is written in full first,
so no URL is left half recorded; the rest are left out.

An unfinished run is never resumed: the next `bruce crawl` starts a new run
from the beginning, and the unfinished one's rows stay in the database.

## Exit status

| Status | Meaning |
|---|---|
| 0 | The run finished. Broken links do not change this. |
| 1 | The configuration, the database or the crawl failed; the message says which. |
| 2 | The command line was not understood. |
| 130 | Stopped by Ctrl-C. |
