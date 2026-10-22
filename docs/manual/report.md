# `bruce report`

```sh
bruce report REPORT [--db FILE] [--run N | --from N --to N] [--repo REPO]
                    [--fact KEY=VALUE] [--kind KIND] [--top N] [--format text|json]
```

Prints one report over one **run** of a database that `bruce crawl` wrote
([`crawl.md`](crawl.md)). A crawl records what each link's target answered; a
report is where a link is judged: a link is `broken_url` where its target
answered 4xx or does not exist, `broken_anchor` where the page is there and
the `#fragment` is not, and `error` where the target could not be asked or
refused access (a timeout, a 5xx after retries, a 401, 403 or 429).

## Arguments

| Argument | Default | Meaning |
|---|---|---|
| `REPORT` | required | `broken`, `errors`, `stats`, `inbound`, `sitemap`, `orphans`, `unlisted`, `redirects`, `diff`, `introduced` or `inverse`. |
| `--db FILE` | `bruce.duckdb` | The database to read. It must exist; a report never creates one. |
| `--run N` | the latest finished run | The run to read. See [Which run](#which-run). Not for `diff`. For `introduced` and `inverse`, a [`check-dir`](check-dir.md) run. |
| `--from N` | the finished run before `--to` | `diff` only: the run to compare against. |
| `--to N` | the latest finished run | `diff` only: the run to compare. |
| `--repo REPO` | none | Only pages whose source file is in this repo, as the page records it (`https://forge.fedoraproject.org/fedora/docs`); for `broken`, `errors`, `stats`, `redirects`, `diff`, `introduced` and `inverse`, the links on those pages (for `inverse`, the baseline's pages outside the build). |
| `--fact KEY=VALUE` | none | Only pages with this fact, such as `antora.component=fedora-docs`, or the links on them. Repeat to require several. |
| `--kind KIND` | `anchor`; every kind for `introduced` and `inverse` | Count links of this kind. Repeat to add kinds; `all` is every kind. `sitemap` and `unlisted` ignore it. |
| `--top N` | all rows | Print the first N rows, in the report's order. `introduced`'s and `inverse`'s counts are then counted over those rows alone. |
| `--format` | `text` | `text`, or `json`. |

The kinds are `anchor` (`<a>` and `<area>`), `image`, `canonical`,
`stylesheet`, `icon`, `link`, `script`, `frame`, `media`, `refresh`,
`sitemap` and `sitemap_file`. A report counts `anchor` links alone unless
`--kind` says more, since a broken image or script is a different job from a
broken link. `introduced` and `inverse` are the gate's, which judge every kind unless
`--kind` narrows them.

## Which run

A report reads **the latest finished run**. `--run N` reads any run, one
that never finished included: `bruce report` then says on stderr that the
report is incomplete, since a run stopped by Ctrl-C holds only the targets
it reached. A database with no finished run, and no `--run`, is an error.
Where a later run never finished, `bruce report` says on stderr that it
stopped before the end and which run it is reporting. If that later run is a
[`check-dir`](check-dir.md) run checked against the run the report would read,
the report is refused, since those rows are the baseline's and not the
build's; name a run with `--run`. `diff`'s default `--to` does the same.
[`bruce prune`](prune.md) deletes the runs a database no longer needs.

`diff` reads two runs. With neither option it compares the latest two
finished runs; `--to N` alone compares run N with the finished run before it,
and `--from N` alone compares run N with the latest finished run. Named runs
may be unfinished, which `bruce report` says on stderr. `--run` with `diff`,
and `--from` or `--to` with any other report, are command-line errors.

## The reports

Rows are sorted worst first, `orphans` and `unlisted` by root and page,
`inverse` by the linking page's group and page, and
`--top N` keeps the first N.

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

**`inbound`** ranks the pages a root's scope covers by how many other pages
link to them, most first: columns `page`, `inbound` and `title`. A page counts
once however many links it has to the target; its link to itself and a
sitemap's listing do not count, and a link to a URL that redirected counts for
the page it reached. A page nothing links to is a row of 0. `--kind` chooses
the links counted.

**`sitemap`** has a row for each page a root's scope covers, and for each
sitemap entry that no page was fetched for: columns `page`, `title`,
`in_sitemap`, `via_links` (whether a link, not a sitemap or the root, claimed it first: false for every page the sitemap lists, which is read before any page), `outcome` and `reason`
(what its URL answered, as `errors` gives them), `status`, `source_repo` and
`source_path`. The entries come first, with no title, `via_links` or source
(a page robots.txt kept the crawl from shows here, never as broken), then the pages absent from the sitemap, then the rest. `--repo` and
`--fact` leave only pages, an entry having none to match.

**`orphans`** has the pages a sitemap lists that no other page links to, by
the links `--kind` chooses: `page`, `title`, `root`, `source_repo` and
`source_path`.

**`unlisted`** has the pages a root's scope covers that its sitemap does not
list: `page`, `root`, `title`, `source_repo` and `source_path`. A root read a
sitemap when a sitemap file reached from it (those it names or robots.txt
does, and the files an index lists) answered with a page and no note: a file
that failed, was refused by robots.txt or ran past the size cap listed
nothing, and one that failed shows in `broken` or `errors` with `--kind
sitemap_file`. A root that read none has nothing to be absent from, so the
text output names it below the table and JSON does not. The `root` column is
the root's URL in canonical form.

**`redirects`** has a row for each URL that links landed on after a
redirect, the one with the most links first: `landing`, `links` (how many
links reached it), `urls` (how many distinct URLs they named) and `chain`,
the hops the first of those URLs took, each a status (`refresh` for a page's
own redirect) and the URL asked, joined by `>`. No link is left out for
being on another host. With `--format json` each row also has `pages`, an
array of the pages whose links they are.

**`diff`** has a row for each link that is broken (as `broken` has it) in
one run and not in the other: `change` is `new` where the link is broken in
`--to` and not in `--from`, and `fixed` where it was broken in `--from` and
is not in `--to`, and the other columns are `broken`'s, read from the run
the link is broken in. A link is the same link in both runs when its page,
its target and its fragment are, however it was written, and links sharing
those are one row. Not broken includes absent: a link removed with its page
or its text counts as `fixed`, and a broken link on a page the first run did
not reach as `new`. `new` comes first, then `broken_url` before
`broken_anchor`. `--repo` and `--fact` keep the rows whose page matches in
the run the row is read from, and `--kind` chooses the links compared in
both. Below the text table a line names the two runs.

**`introduced`** reads a run that [`bruce check-dir`](check-dir.md) wrote,
and has a row for each link that is `broken_url` or `broken_anchor` in it and
that the run's baseline does not have broken, the failures of `check-dir`'s
exit status: `broken`'s columns with one more, `baseline`, after `status`. It
is `new` where the baseline has no such link (the same page, target and
fragment), else the outcome the baseline had (`ok`, `unchecked` or `error`).
Below the text table are the baseline run and its day, the inverse breakage
(`inverse`'s rows), the redirected links the baseline did not have redirected, the links whose outcome is `error` and
the count of breakage the baseline already had, as `check-dir` prints them.
A run that names no baseline is an error. The report's exit status is 0
whatever it finds.

**`inverse`** reads the same run and lists what the change removed that pages
outside the build link to, the second failure of `check-dir`'s exit status:
`page`, `raw_href`, `target`, `fragment`, `kind`, `removed`, `component`,
`source_repo` and `source_path`, a row for each linking page, target and
fragment of the baseline, grouped
by the linking page's component (its repo where it has none). `removed` is
`page` where the target, or the page it redirected to, is a page of a
component-version the build holds and the build lacks; `file` where it is an
image or an attachment of such a component-version that the build lacks; and `anchor` where
the page is built and has lost the anchor the link named. Only links the baseline had `ok` on a page the build does not
hold are read: a page of a component-version the build holds is the build's own, and
its links are `introduced`'s or gone with it. Below the text table are the
baseline run and its day, and the count of links, inbound pages and
components. `--repo` and `--fact` filter the linking page in the baseline.

## Output

The text format is a table with one row a line. A missing value is `-`, and
a report with no rows prints its header and `(none)`.

`--format json` prints one array of objects, a row each, whose field names
are the columns above and do not change. A missing value is `null`, a count
or status a number:

```json
[{"page":"https://docs.example.org/en-US/a/","raw_href":"../b/#nope","target":"https://docs.example.org/en-US/b/","fragment":"nope","kind":"anchor","outcome":"broken_anchor","reason":null,"status":200,"source_repo":"https://forge.example.org/docs","source_path":"modules/ROOT/pages/a.adoc"}]
```

`stats` prints its page rows alone as JSON, `unlisted` its pages alone and `introduced` and `inverse` their links alone; the totals, the roots without a sitemap and `introduced`'s other sections are in text only (`check-dir --format json` has them all).

## Querying the database directly

Opening a database with `bruce` installs the views the reports read, so
`duckdb bruce.duckdb` finds them after a crawl alone: `report_broken`,
`report_errors`, `report_stats` (counts by page and kind) and
`report_pages`, `report_inbound`, `report_roots`, `report_sitemap` and
`report_redirects`, and beneath them `link_verdicts`, every link with its
outcome. Each holds every run, so filter on `run_id`.

## Exit status

| Status | Meaning |
|---|---|
| 0 | The report ran. Finding broken links does not change this. |
| 1 | The database could not be read, or has no such run or no finished one, `diff` has no finished run to compare, the run a bare report would read is the baseline of a later unfinished `check-dir` run, or an `introduced` or `inverse` run names no baseline; the message says which. |
| 2 | The command line was not understood. |
