# `bruce check-dir`

```sh
bruce check-dir DIR --prefix URL --baseline FILE|URL [--config FILE] [--db FILE]
                    [--externals skip|baseline|live] [--format text|json] [--all]
```

Checks a content repo's partial build against a **baseline**, and fails only
on the breakage the change introduced. `DIR` is the directory an Antora build
wrote, the baseline is a file `bruce export-baseline` wrote
([`export-baseline.md`](export-baseline.md)) from a scheduled full crawl, and
the exit status says whether the change broke a link the baseline did not
have broken. The run is added to a database like a crawl's ([`crawl.md`](crawl.md)), so
[`bruce report`](report.md) reads it.

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `DIR` | required | The build directory. Every `.html` file under it is a page checked. |
| `--prefix URL` | required | The production URL of the directory's root, such as `https://docs.example.org/en-US`. A file is the page at the prefix plus its path, and `foo/index.html` is `foo/` too. It must be inside a root's scope. |
| `--baseline FILE\|URL` | required | The baseline file, or an `https` URL to download it from. A `.gz` file or URL is decompressed. Nothing is cached. |
| `--config FILE` | none | The configuration the baseline was crawled under ([`config.md`](config.md)). With none, the prefix is the only root. |
| `--db FILE` | `bruce.duckdb` | The database the run is added to, created if it does not exist. |
| `--externals MODE` | `baseline` | How a link to an allowlisted host is answered; see below. |
| `--format` | `text` | `text`, or `json`. |
| `--all` | off | Print the breakage already in the baseline in full. |

Use the **same configuration the baseline was crawled under**, so `skip`, the
allowlist and the roots' scopes plan a URL as they did for the baseline: a
difference between the two reads as breakage.

## How a link is answered

A link to a page in the build is read from the directory. Any other is
answered from the baseline where it holds the URL, else by a request to the
live site, at the configuration's pace:

- **A page of a component the build holds, but not built**, is `broken_url`,
  reason `not_built`: the build is the truth for the components it holds,
  where the baseline would pass a link to a page the change deletes. The build
  holds the components whose pages it carries, by their component and version.
- **A page of a component the build does not hold** is the baseline's answer;
  a URL the baseline never saw is asked of the live site.
- **An `xref:` into a component the build lacks**, which Antora leaves as
  `#component::page.adoc`, is resolved to the baseline's page for it, and
  judged as any link. One into a component the build holds stays
  `unresolved_xref`.
- **A link starting `/`** resolves against the prefix, the way a static server
  serving the directory at `/` shows it.
- **A link to another host** is answered by `--externals`: `baseline` takes
  the baseline's answer where it has one and asks otherwise, `live` always
  asks, and `skip` never asks: the baseline's answer where it has one, else
  `unchecked`. `--externals` does not reach a URL on the site itself.

## What fails

A link is the page it is on, the URL it points at and its fragment, of every
kind: a broken image or script fails the check as a broken link does. It is
**introduced** when it is `broken_url` or `broken_anchor` in this run and the
baseline has the same link as anything else: absent, `ok`, `unchecked` or
`error`. A link the baseline never asked about can then fail CI, since it is
new to what the baseline knows. Only the links on pages the build holds count:
the baseline's links on a page not built are not this change's.

An `error` (a timeout, a 5xx, a refusal to answer) never fails the check, and
a redirect never does.

## Output

Text lists, in order:

1. **Introduced breakage**: a row for each link, with the columns
   [`report.md`](report.md)'s `broken` has and one more, `baseline`, naming
   what the baseline had for it: `new` where it had no such link, else the
   outcome it had (`ok`, `unchecked`, `error`).
2. **Introduced redirected links**: links through a redirect that the
   baseline did not have redirected: `page`, `raw_href`, `target`,
   `fragment`, `kind`, `landing`, `chain` and the source columns. They never
   fail the check.
3. **Errors**: the links whose outcome is `error` in this run, with `broken`'s
   columns.
4. **Breakage already in the baseline**: a count, and with `--all` the links,
   which never fail the check.

```text
run 7: 23 targets, 3 pages
against baseline run 1, crawled on 2026-10-19

introduced breakage: 1
page                         raw_href  target                       fragment  kind    outcome        reason  status  baseline  source_repo  source_path
https://docs.example.org/a/  b/#gone   https://docs.example.org/b/  gone      anchor  broken_anchor  -          200  ok        -            -

introduced redirected links: none

errors: none

breakage already in the baseline: 4 (--all lists them)
```

The first line is the run's number and counts and the second names the
baseline run and the day it was crawled, whose answers stand whatever their
age.

`--format json` prints one object on stdout, and the first line goes to
stderr:

```json
{"run":7,"baseline_run":1,"baseline_date":"2026-10-19","introduced":[],"redirected":[],"errors":[],"already_broken":{"count":4,"links":null}}
```

`introduced`, `redirected` and `errors` are arrays of objects whose field
names are the columns above; `already_broken.links` is `null` unless `--all`
is given.

The same sections are `bruce report introduced --run N` over the run in the
database ([`report.md`](report.md)).

## Exit status

| Status | Meaning |
|---|---|
| 0 | No link is introduced. |
| 1 | A link is introduced. |
| 2 | The check could not run: the command line was not understood, the configuration or the baseline could not be read, the baseline or the database is of another schema version, the prefix is outside every root's scope, or the build could not be read. |

An `error` never moves the status.

## Not beside a crawl

DuckDB lets one process write a file. `bruce check-dir` on a database a crawl
has open fails at once with status 2.
