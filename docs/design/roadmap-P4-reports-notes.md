# P4 Verdicts, reports, and the first en-US crawl: notes

What P4 left for later phases, and what it tried and refused. Its intent is
[`roadmap-P4-reports.md`](roadmap-P4-reports.md); what it built is the code,
[`../status/STATUS.md`](../status/STATUS.md)'s "What exists", the manual and
the concept map.

## Reproducing the evidence

- **Two en-US crawls**, each the release build launched detached from
  `sim/bruce.toml` (`mise run sim`, D42): one root
  `https://docs.fedoraproject.org/en-US/` with its sitemap set, the forge
  allowlisted and `per_second = 1`, its `issues/new*`, `issues/new*/**` and
  `/user/login*` skipped. Each directory holds `en-us.duckdb` (one run),
  `crawl.log`, `time.log` and `HANDOFF.md` (the launch command).
  `runs/p4.4-en-us-20261019-1633/` ran before the guide's two fixes (robots.txt
  obeyed on docs.fp.o, the single `issues/new*` glob);
  `runs/p4.10-en-us-20261019-2125/` with them, at `2547df1`. Beside the second,
  `combined.duckdb` holds the first as run 1 and the second as run 2, and
  `out/` each report's text on run 2.

  | | P4.4 | P4.10 |
  |---|---|---|
  | Summary | 172239 targets, 6425 pages, 69 sitemap files | 172452, 6433, 69 |
  | Wall clock | 2:20:42 | 1:48:22 |
  | Peak RSS | 168 MiB | 161 MiB |
  | DB size | 30 MiB; 587839 `links`, 93421 `anchors` | 27.5 MiB; 588551, 93521 |
  | `ok` | 7299, all 200; 61 through a redirect | 7307; 61 |
  | `broken_url` | 100: 68 docs.fp.o, 32 forge; all 404 | 109: 77 and 32 |
  | `error` | 0 | 2, `other`, forge `commits/` URLs `ok` in P4.4 |
  | `unchecked` | 154380 `out_of_scope`, 8397 `host_not_allowlisted`, 1925 `skipped`, 128 `scheme`, 10 `robots` | 154581, 8397, 1928, 128, none |
  | Requests | 3251 docs.fp.o, 6056 forge | 3263, 4155 |

  `combined.duckdb` is built by attaching the second DB and inserting its rows
  with `run_id` 2, table by table:

  ```sh
  cd runs/p4.10-en-us-20261019-2125
  cp ../p4.4-en-us-20261019-1633/en-us.duckdb combined.duckdb
  duckdb combined.duckdb "attach 'en-us.duckdb' as n (read_only);
    insert into runs select 2, * exclude (run_id) from n.runs;" # likewise pages,
  # page_facts, targets, anchors, links and bodies
  ../../target/release/bruce report diff --db combined.duckdb
  ```

- **Timing**, one process per report on `combined.duckdb`, `Store::open`
  installing every view: `sitemap` and `unlisted` 0.5 s and 66 MiB; `broken`,
  `inbound`, `redirects` and `orphans` 0.6 to 0.7 s and 276 to 484 MiB.
  `link_verdicts` over one run, grouped by outcome and reason: 0.1 s warm,
  0.37 s cold (`duckdb` 1.5.5 CLI, `.timer on`, the view created `TEMP` from
  `LINK_VERDICTS` in `src/store/views.rs`). D28 stays unmaterialised.
- **`prune` on a copy of P4.4's DB** (one run): `--keep 1` deleted nothing in
  0.13 s; `--older-than 0s` deleted run 1 in 0.19 s and left a 5 MB file.
- **The skip-glob check** was a throwaway program over `globset` with
  `literal_separator(true)`, as `Patterns::new` builds it, fed P4.4's
  `select url from targets where url like 'https://forge.%issues/new%'`:
  `issues/new*` matched 16 of 1921, `issues/new*/**` the other 1905 and
  nothing else of the 172239. Two `config::tests` read the guide's globs now.

## What P5, CI mode, inherits

- **`diff` reads not-broken as absent, and the first real `diff` shows the
  cost.** `new` is broken in `to` with no broken row in `from`, `fixed` the
  reverse (`src/report/diff.rs`), so a link on a page the base run did not
  reach is `new`. P4.10 against P4.4 printed 9 `new` and no `fixed`: the nine
  404s P4.4 left `unchecked` / `robots`, new only because the guide stopped
  obeying robots.txt. The two forge `error`s, `ok` in P4.4, are in no row:
  `diff` reads `broken` only. A column naming the link's state in the other
  run costs one join to `link_verdicts`. P5's gate reads a `diff` as
  introduced breakage (D9), so it decides whether the column is built and
  whether a link that was `unchecked` counts as introduced.
- **`report_broken`'s key across runs** is the linking page's `url_id`, the
  target's URL and the fragment (`''` for none), all stable across runs
  (D26). A baseline compared with a partial build's run has that key to hand.
- **`runs.baseline_run` of a kept run may name a pruned one.** Both `prune`
  rules delete a prefix of run numbers, so a number is reused only when
  `--older-than` deletes every run and the next crawl is run 1 again. Nothing
  reads the column yet.
- **A filter names a page, and `--repo` takes the stored URL**
  (`https://forge.fedoraproject.org/atomic/docs`), not `group/repo`. A link
  report filters on the page the link is on, a page report on the page the row
  lists. A CI run scoped to one repo filters the same way.

## What P6, release and the deployment guide, inherits

- **`sim.py` writes only the files the guide names**; one the guide stops
  naming stays in `sim/` until deleted by hand. The crawl's own files need
  not live there: both crawls wrote under `runs/`.
- **A crawl or `duckdb` session holding the file makes `prune` fail at open**
  with DuckDB's lock error, exit 1, so a scheduled `prune` cannot overlap the
  scheduled crawl.

## Calls beyond the spec

None touches a contract; each is a place a later reader may ask "why":

- `link_verdicts` has two reasons the spec did not name: `text_fragment`, a
  fragment holding `:~:` anywhere, and `cut`, a link whose target has no row,
  which only a stopped run leaves. `#top` matches in any case, HTML's rule;
  an id matches exactly, raw or percent-decoded.
- `reason` is NULL for most breakage: a 404 is `broken_url` with no reason
  (`outcome::answered`), as is every `broken_anchor`.
- A link's `redirected` is `targets.final_url IS NOT NULL`, true for a chain
  ending `skipped` or `robots` too.
- `report_inbound` resolves a link to the page it landed on, through
  `final_url`, so a page reached only by a redirect is no orphan.
- `redirects` shows one chain per landing URL, the first origin by name.
- An index that answered `ok` counts as a read sitemap even when every child
  failed: the rule is "any file reached".
- `diff`: `--from` alone reads against the latest finished run, `--to` alone
  against the finished run before it; a filter is read in the run a row comes
  from, so a page that moved repo is in both halves; `--kind` applies to both
  runs.
- `prune`: `--keep` takes at least 1; `--keep` and `--older-than` are
  exclusive; an age is a whole number with `s`, `m`, `h`, `d` or `w`; with no
  finished run, both rules keep every unfinished run.
- `stats`' totals are text only; JSON is the page rows (D6's `Rejected`).

## Not done

- **A set or discovered sitemap whose own URL a glob matches is still read**:
  `skip` ends a *hop*, as the spec says, and `sitemap_item` plans a sitemap
  without asking it.
- **A root with an escape** is tested by `root_urls`' unit test and
  `report_roots`' tests, not by a crawl: wiremock matches the request path,
  which the fetcher does not canonicalise for the mock.
- **Older DBs keep `runs.roots` and `pages.root` as configured**, so a root
  with an escape or a stripped query reads as having no sitemap there; no
  schema bump.
- **How `CHECKPOINT` fares pruning a database of many runs** is unmeasured.

## Negative results

- **The frontier's order runs the hosts nearly in turn.** In P4.4 the sitemap's
  pages took the first 47 minutes and the forge, at one start a second, the
  next 101; in P4.10 the forge began 39 minutes in. `targets.checked_at` is
  when a target's hops began, so its distribution is the schedule; the code
  was not read for the cause. `duration_ms` includes waiting for the host's
  turn (forge mean 61 s), so no column holds a response time.
- **Obeying docs.fp.o's robots.txt hid nine 404s** linked from live pages,
  and a sitemap page its own `Disallow: /*/Fedora*/` forbids (I2). That
  earned D18's rewrite.
- **A forge `issues/new` answers 303 to `/user/login` for a repo that does
  not exist as well**, so the link says nothing anonymously; `skip` replaced
  D46's memo for it (D50).
- **Something on docs.fp.o redirects**: 55 of the 61 `ok` redirected targets,
  a 302 or a `refresh`, in both runs.
- **Run-to-run drift is small.** Between the crawls, 10 pages only in the
  second, 2 only in the first; the 610 `broken_anchor`s the same count;
  `orphans` 74 and `unlisted` 13 in both. 138 pages changed hash, 137 of them
  the forge's (P8's inbox).
- **No `no_anchors`, `text_fragment` or `cut` link** in either crawl: KD4 and
  the stop's paths did not show.
- **The bundled DuckDB has no ICU**, so a `TIMESTAMPTZ` takes no interval
  arithmetic (`now() - to_seconds(?)` fails to bind); `prune` compares
  `epoch_us` values.
- **`ORDER BY` on an alias that is also a joined view's column is
  ambiguous** in DuckDB, so `stats` sorts an outer select; and a page report's
  `kind` filter sits in the join's `ON`, `report_stats` having no row for a
  page with no links.
- **`serde_json` is not a dependency**; JSON is written by hand
  (`src/report/table.rs`).
- **A join of `anchors` on `a.anchor IN (fragment, decoded)`** was not built;
  two `EXISTS` keep each an equality, which DuckDB plans as hash joins.

## Evidence only (D41)

The broken links found are the deployer's to report; none is filed from here.
On P4.10's run: `broken` 827 rows, 217 `broken_url` and 610 `broken_anchor`
among anchor links; the worst page `/legal/not-allowed-licenses/`, 112 broken
anchors and one broken URL, the legal pages holding most of the 610 (266 name
`_what_is_licenseref`). Two forge shapes recur among the 404s: `.../-/issues`
(GitLab's path) and `council/tickets/issue/N`.
