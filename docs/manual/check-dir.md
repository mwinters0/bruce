# `bruce check-dir`

```sh
bruce check-dir DIR --prefix URL --baseline FILE|URL [--config FILE] [--db FILE]
                    [--externals skip|baseline|live] [--inverse fail|warn]
                    [--format text|json] [--all]
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
| `DIR` | required | The build directory. Every `.html` or `.htm` file under it is a page checked, and every other file a target of the run. |
| `--prefix URL` | required | The production URL of the directory's root, such as `https://docs.example.org/en-US`. A file is the page at the prefix plus its path, and `foo/index.html` is `foo/` too. It must be inside a root's scope. |
| `--baseline FILE\|URL` | required | The baseline file, or an `http` or `https` URL to download it from. A gzipped file is decompressed, whatever its name. Nothing is cached. |
| `--config FILE` | none | The configuration the baseline was crawled under ([`config.md`](config.md)). With none, the prefix is the only root. |
| `--db FILE` | `bruce.duckdb` | The database the run is added to, created if it does not exist. |
| `--externals MODE` | `baseline` | How a link to an allowlisted host is answered; see below. |
| `--inverse MODE` | `fail` | Whether the [inverse check](#the-inverse-check) fails the check (`fail`) or is only listed (`warn`). |
| `--format` | `text` | `text`, or `json`. |
| `--all` | off | Print the breakage already in the baseline in full. |

Use the **same configuration the baseline was crawled under**, so `skip`, the
allowlist and the roots' scopes plan a URL as they did for the baseline: a
difference between the two reads as breakage.

## How a link is answered

A link to a page in the build is read from the directory. Any other the
configuration would have a crawl ask about is answered from the baseline where
it holds the URL, else by a request to the live site, at the configuration's
pace; one it would leave `unchecked` (`skip`, another locale's page, a host
not on the allowlist) stays `unchecked`:

- **A page of a component the build holds, but not built**, is `broken_url`,
  reason `not_built`: the build is the truth for the components it holds,
  where the baseline would pass a link to a page the change deletes. The build
  holds the components whose pages it carries, by their component and version.
- **A page of a component the build does not hold** is the baseline's answer;
  a URL the baseline never saw is asked of the live site.
- **An `xref:` into a component the build lacks**, which Antora leaves as
  `#component::page.adoc`, is resolved to the baseline's page for it, and
  judged as any link. Where the baseline has no page with that source file,
  Antora may still have resolved it through a page-alias, so the URL the
  reference names is answered as any other the build lacks: from the baseline,
  else by a request to the live site, and a URL that answers broken is an
  `unresolved_xref`. One into a component the build holds stays
  `unresolved_xref`.
- **A link whose scheme no reader can follow**, as `file:`, is `broken_url`,
  reason `bad_scheme`, as in a crawl
  ([which schemes](digest.md#why-a-link-is-unchecked)).
- **A link to a URL whose path doubles a slash** (`a//b`), or that redirects
  to one, is `broken_url`, reason `doubled_slash`, whatever the URL answered
  and whether it was asked, as in a crawl.
- **A link to a URL that answers 404** where the same URL with its final `/`
  added or dropped answers 2xx is `broken_url`, reason `wrong_trailing_slash`,
  as in a crawl ([`bruce crawl`](crawl.md#output)).
- **A link starting `/`** resolves against the prefix, the way a static server
  serving the directory at `/` shows it.
- **A link to an allowlisted host**, or to a root's host outside every root's
  scope, is answered by `--externals`: `baseline` takes
  the baseline's answer where it has one and asks otherwise, `live` always
  asks, and `skip` never asks: the baseline's answer where it has one, else
  `unchecked`. `--externals` does not reach a URL in a root's scope. Another
  locale's page stays `unchecked` ([Roots](config.md#roots)); the locales are
  those the built pages' language menus name, and a build whose UI has none,
  as a content repo's usually has not, answers its links to them this way.

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

### The inverse check

A change can break a page it does not contain: removing a page, a file or an
anchor that other components link to. The check reads the baseline's links on
pages **outside the build**, in component-versions the build does not hold, and
fails where one was `ok` and what it reached is now gone from a component-version
the build holds: the page is not built, an image or attachment is not built, or
the page is built and has lost the anchor. The same page, file or anchor in a
component-version the build lacks is the baseline's, unchanged. The output lists each link with
the component it is on, and the count of inbound pages across components. `--inverse=warn` lists them and
leaves the exit status alone, for a repo whose CI is not yet ready to be that
strict; the same list is `bruce report inverse --run N` ([`report.md`](report.md)).

## Output

Text lists, in order:

1. **Introduced breakage**: a row for each link, with the columns
   [`report.md`](report.md)'s `broken` has and one more, `baseline`, naming
   what the baseline had for it: `new` where it had no such link, else the
   outcome it had (`ok`, `unchecked`, `error`).
2. **Inverse breakage**: the baseline's links from pages outside the build to
   what the change removed, with the columns [`report.md`](report.md)'s
   `inverse` has. They fail the check unless `--inverse=warn`.
3. **Introduced redirected links**: links through a redirect that the
   baseline did not have redirected: `page`, `raw_href`, `target`,
   `fragment`, `kind`, `landing`, `reason`, `chain` and the source columns. They
   never fail the check. `reason` is `pedantic_trailing_slash` where the
   landing differs from the target only by a final `/` added or removed, and
   those links are listed last.
4. **Errors**: the links whose outcome is `error` in this run, with `broken`'s
   columns.
5. **Breakage already in the baseline**: a count, and with `--all` the links,
   which never fail the check.

```text
run 7: 23 targets, 3 pages
against baseline run 1, crawled on 2026-10-19

introduced breakage: 1
page                         raw_href  target                       fragment  kind    outcome        reason  status  baseline  source_repo  source_path
https://docs.example.org/a/  b/#gone   https://docs.example.org/b/  gone      anchor  broken_anchor  -          200  ok        -            -

inverse breakage: none

introduced redirected links: none

errors: none

breakage already in the baseline: 4 (--all lists them)
```

The first line is the run's number and counts and the second names the
baseline run and the day it was crawled, whose answers stand whatever their
age.

`--format json` prints one object on stdout, and the first line goes to
stderr, prefixed `bruce check-dir: `:

```json
{"run":7,"baseline_run":1,"baseline_date":"2026-10-19","introduced":[],"inverse":[],"redirected":[],"errors":[],"already_broken":{"count":4,"links":null}}
```

`introduced`, `inverse`, `redirected` and `errors` are arrays of objects whose field
names are the columns above; `already_broken.links` is `null` unless `--all`
is given.

The same sections are `bruce report introduced --run N` over the run in the
database ([`report.md`](report.md)).

## Exit status

| Status | Meaning |
|---|---|
| 0 | No link is introduced, and no inverse breakage is found or `--inverse=warn`. |
| 1 | A link is introduced, or the inverse check finds breakage and `--inverse=fail`. |
| 2 | The check could not run: the command line was not understood, the configuration or the baseline could not be read, the baseline or the database is of another schema version, the prefix is outside every root's scope, or the build could not be read. |
| 128 plus the signal | Stopped by a signal before the build was judged. |

An `error` never moves the status, nor does a redirected link or a warned inverse breakage.

## Stopping a check

A SIGINT (Ctrl-C) or SIGTERM, as a CI cancel sends, stops the run as it does
[`bruce crawl`](crawl.md#stopping-a-crawl): a second signal of either kind
abandons what is still out. A check that was stopped is **not judged**: no
section is printed, stdout stays empty, and stderr says the build was not
judged. Its run stays in the database unfinished, so a `bruce report` with no
`--run` refuses, naming both runs, rather than read the baseline this check
imported ([`report.md`](report.md#which-run)). The exit status is 128 plus the first signal, 130
for SIGINT and 143 for SIGTERM.

## Not beside a crawl

DuckDB lets one process write a file. `bruce check-dir` on a database a crawl
has open fails at once with status 2.
