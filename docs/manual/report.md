# `bruce report`

```sh
bruce report REPORT [--db FILE] [--run N] [--repo REPO] [--fact KEY=VALUE]
                    [--kind KIND] [--top N] [--format text|json]
```

Prints one report over one **run** of a database that `bruce crawl` wrote
([`crawl.md`](crawl.md)). A crawl records what each link's target answered; a
report is where a link is judged: a link is `broken_url` where its target
answered 4xx or does not exist, `broken_anchor` where the page is there and
the `#fragment` is not, and `error` where the target could not be asked
(a timeout, a 5xx after retries).

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `REPORT` | required | `broken`, `errors` or `stats`. |
| `--db FILE` | `bruce.duckdb` | The database to read. It must exist; a report never creates one. |
| `--run N` | the latest finished run | The run to read. See [Which run](#which-run). |
| `--repo REPO` | none | Only links on pages whose source file is in this repo, as the page records it (`https://forge.fedoraproject.org/fedora/docs`). |
| `--fact KEY=VALUE` | none | Only links on pages with this fact, such as `antora.component=fedora-docs`. Repeat to require several. |
| `--kind KIND` | `anchor` | Count links of this kind. Repeat to add kinds; `all` is every kind. |
| `--top N` | all rows | Print the first N rows, the worst first. |
| `--format` | `text` | `text`, or `json`. |

The kinds are `anchor` (`<a>` and `<area>`), `image`, `canonical`,
`stylesheet`, `icon`, `link`, `script`, `frame`, `media`, `refresh`,
`sitemap` and `sitemap_file`. A report counts `anchor` links alone unless
`--kind` says more, since a broken image or script is a different job from a
broken link.

## Which run

A report reads **the latest finished run**. `--run N` reads any run, one
that never finished included: `bruce report` then says on stderr that the
report is incomplete, since a run stopped by Ctrl-C holds only the targets
it reached. A database with no finished run, and no `--run`, is an error.

## The reports

Rows are sorted worst first, and `--top N` keeps the first N.

**`broken`** has a row for each link whose outcome is `broken_url` or
`broken_anchor`, `broken_url` first, then by page:

| Column | Holds |
|---|---|
| `page` | The page the link is on. |
| `raw_href` | The link as written on the page. |
| `target` | The URL it resolves to, without the fragment. |
| `fragment` | The `#fragment`, or none. |
| `kind` | The link's kind. |
| `outcome` | `broken_url` or `broken_anchor`. |
| `reason` | Why, where the crawl knew: `unresolved_xref` for an Antora `xref:` that names no page, a redirect loop, an unparseable URL. None for a plain 404 or a missing fragment. |
| `status` | The HTTP status the target answered, where it answered. |
| `source_repo`, `source_path` | The repo and file the page was built from, where the page links its edit page. |

**`errors`** has the same columns, for the links whose outcome is `error`. An
error says the target could not be asked, not that the link is wrong, so it
is kept apart from `broken`.

**`stats`** has a row for each page a root's scope covers, with the number of
`links` it holds and how many are `broken_url`, `broken_anchor`, `error` and
`unchecked`, the most broken page first. Below the table are the run's
totals and the `unchecked` links by reason (`out_of_scope`,
`host_not_allowlisted`, `skipped`, `robots`, `scheme` and the others), each
with the number of links and of distinct URLs they name, so a link the crawl
chose not to ask about is shown and never counted as broken. The filters
narrow both counts.
Pages fetched only because a page linked to them (the allowlist) have no row.

## Output

The text format is a table with one row a line. A missing value is `-`, and
a report with no rows prints its header and `(none)`.

`--format json` prints one array of objects, a row each, whose field names
are the columns above and do not change. A missing value is `null`, a count
or status a number:

```json
[{"page":"https://docs.example.org/en-US/a/","raw_href":"../b/#nope","target":"https://docs.example.org/en-US/b/","fragment":"nope","kind":"anchor","outcome":"broken_anchor","reason":null,"status":200,"source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/a.adoc"}]
```

`stats` prints its page rows alone as JSON; the totals are in text only.

## Querying the database directly

Opening a database with `bruce` installs the views the reports read, so
`duckdb bruce.duckdb` finds them after a crawl alone: `report_broken`,
`report_errors`, `report_stats` (counts by page and kind) and
`report_pages`, and beneath them `link_verdicts`, every link with its
outcome. Each holds every run, so filter on `run_id`.

## Exit status

| Status | Meaning |
|---|---|
| 0 | The report ran. Finding broken links does not change this. |
| 1 | The database could not be read, or has no such run; the message says which. |
| 2 | The command line was not understood. |
