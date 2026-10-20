# P5 CI mode: baseline and `check-dir`: notes

What P5 left for later phases, and what it tried and refused. Its intent is
[`roadmap-P5-ci.md`](roadmap-P5-ci.md); what it built is the code,
[`../status/STATUS.md`](../status/STATUS.md)'s "What exists", the manual
(`check-dir.md`, `export-baseline.md`, `report.md`'s `introduced` and
`inverse`) and the concept map. Its decisions are D51–D57.

## Reproducing the evidence

- **The en-US crawl under schema 4**, launched detached from `sim/bruce.toml`
  (`mise run sim`, D42) under `runs/p5.2-en-us-20261020/` (`HANDOFF.md` holds
  the launch command; `en-us.duckdb` one run, 33042432 bytes). Its export is
  `runs/p5.3-baseline/baseline.duckdb`: `target/release/bruce export-baseline
  --db runs/p5.2-en-us-20261020/en-us.duckdb --out <file>`, 2.3 s and 207 MB.
  The size and its gzip are P6's inbox. `report stats` and `broken` read the
  file and give the source's rows.
- **The end-to-end test** is `mise run e2e` (`scripts/e2e.py`, its docstring
  the mechanism), writing under `runs/e2e/`: the fixture site built whole and
  served under `/en-US/`, crawled and exported, then four builds of `tools`
  checked against it, each asserting the exit status and every section of the
  JSON (`UNCHANGED_*`, `PLANTED_*`, `IMAGE_ONLY_INVERSE`). Nothing leaves the
  machine.
- **quick-docs**, under `runs/p5.11-quick-docs/`: quick-docs `70cd18e` (the
  forge's clone; `pagure.io`'s holds only a README), team-docs `e896044`,
  user-documentation `ca833e4`, operations/docs `98ef1c0`, all `main` on
  2026-10-20, built by Antora 3.1.15 from `runs/e2e/antora` with `CI=true`,
  `playbook.yml` (`site.yml`'s, the vendored UI bundle, `indexify`): 25 s, 299
  `.html` files in `public/`. `planted.patch` applied to a copy is
  `public-planted/`. The check, the deployment guide's config:

  ```sh
  target/release/bruce check-dir public --prefix https://docs.fedoraproject.org/en-US/ \
    --baseline ../p5.3-baseline/baseline.duckdb --config ../../sim/bruce.toml \
    --db ci.duckdb --format json
  ```

  | | As published (5.11) | As published (5.12, `runs/p5.12-quick-docs/`) | Planted (5.11) |
  |---|---|---|---|
  | Exit | 1 | 0 | 1 |
  | Introduced | 4, KD8's xrefs | 0 | 12: KD8's 4 and the 8 below |
  | Inverse | 1, KD9's | 0 (re-read by `report inverse` after 5.11.1) | 23 links on 22 pages in 7 components; 22 after 5.11.1 |
  | Redirected | 236, every page's home link | 239, the three xrefs added | |
  | Already broken | 91 | 91 | |
  | Requests | none | one, an uncrawled alias, `ok` | one, a planted 404 |

  The planted introduced rows: `not_built` for the `link:` to the deleted page
  and for `fedora-server/server-communicating`'s link to its page-alias;
  `broken_anchor` for `quick-docs/` linking the removed anchor;
  `unresolved_xref` for the held-component xref to a missing page and the two
  quick-docs pages that `xref:` the deleted one; a live 404 for `../nowhere/`;
  `not_built` for the missing image. The valid xref out of the build resolved.
  The deleted-image case is not plantable: no other component embeds an image
  of quick-docs's.
- **The inverse check's cost** with that baseline (588k links) and 284
  quick-docs pages as the build: 2.5 s in a debug build.

## What P6, release and the deployment guide, inherits

Beside its inbox ([`roadmap-P6-release-inbox.md`](roadmap-P6-release-inbox.md)):

- **A repo's CI job** is Antora from npm with `CI=true` and the playbook's UI,
  then `check-dir` with the deployment's config, published beside the baseline
  (spec, "The config is the deployment's"). `check-dir.md` is the command's
  page; the guide's CI section is P6's.
- **A CI job reaches production** for an in-scope URL the build lacks that the
  baseline holds no target for, an xref no baseline page carries included
  (D57): the URL is in a root's scope, so the allowlist is not what lets it,
  and `--externals=skip` does not stop it (D22's `Rejected` line).
- **Ctrl-C is not handled by `check-dir`**: a killed run leaves
  `runs.finished_at` NULL, and `report` with no `--run` then picks the latest
  finished run, which is the imported baseline.
- **Every error `check-dir` returns is exit 2**, a crawl or writer failure
  midway included; the gate then does not run.
- **Whether the gate's `error` section is noise is unanswered.** It lists every
  row in full; the e2e fixture and quick-docs gave none, the config allowlisting
  only the forge. A repo linking a host that refuses every request would show
  it repeating on each page; the count alone is a one-line change in
  `Gate::rest`.

## Calls beyond the spec

None touches a contract; each is a place a later reader may ask "why":

- **The baseline file keeps its run's number** and the source's `started_at`
  and `config_hash`, with `runs.baseline_run` NULL; six tables are copied,
  `bodies` not (D15); the reports' views are installed at open, not exported.
  `--force` writes `<out>.partial` and renames, so a refused run keeps the old
  file.
- **The import's reuse covers the run the file was exported from**: importing
  into the history DB returns the original run. It refuses before copying a
  file with no `meta`, another schema version, or other than one finished run.
- **A copy is one level deep and carries no facts**: the target, its landing
  target, each one's `pages` row and `anchors`, all `baseline` (D51, D52); so
  the held set and the inverse check, read off the run's `page_facts`, never
  count a copied page. A target the run already holds keeps its row.
- **Every file of the build is a seed**, `x/index.html` also `x/`; a non-HTML
  seed is a target with no page (`fetch::local` reads none of its bytes). A
  prefix below a root's URL never asks the root's own URL, and a seeded run
  reads no sitemap or robots.txt.
- **The gate's `baseline` column** reads a key with several baseline links as
  its least certain outcome (`error`, then `unchecked`, then the rest);
  `baseline_date` is the run's day in the session's timezone; `check-dir`'s
  `run N: …` line goes to stderr under `--format json`. `report inverse --top`
  truncates the count line too.
- **Resolve's "latest"** is Antora's order by version name only, a `-` suffix
  the one prerelease it knows (the baseline holds no prerelease flag); a
  `family$` other than `page$` is not resolved; a placeholder version
  (`master`, `""`) matches only that literal.
- **The inverse check groups a linking page** by its baseline `component`
  fact, else its repo, else nothing; the runs' ids are written into its
  statement as numbers, leaving the positional placeholders to the user's
  values.

## Not done

KD5, KD6 and KD10 are the register's. Beside them, met and not removed:

- **One limit of the held set, in `Held::holds`**: a bare version holds every
  version-less URL of its component and every version a refspec names, so a
  link to `component/1.0/…` with `component` built at `master` is `not_built`,
  and `1.0@component::x` stays unresolved. A held version served under another
  name (`latest_version_segment`, I5) is not recognized, so a page deleted
  from it is the baseline's `ok`.
- **A component-version with no prefix to name**: the latest `fedora`, served
  at `latest` (I5), and one whose baseline pages carry no `version` fact, so
  an xref there no page's facts find stays unresolved.
- **I16's layout is read in three places** (the concept map's "Not in one
  place").
- **A hop onto a URL the build lacks is asked live**, never copied
  (`Frontier::hop_plan`); Antora aliases stay in their component, so none is
  known. Under `--externals=skip` a hop onto an allowlisted host ends the chain
  whether or not the baseline holds the target, and `is_skipped_external`
  repeats `plan`'s order, `plan` returning a class.
- **The inverse check cuts a URL at the first segment equal to the
  component's name**, so a prefix whose own path holds one cuts early. No
  deployment has one.
- **Xref rows are written late**, in `settle_xrefs`; a run stopped before
  then leaves them out. `Summary::targets` counts a copy once though
  `copy_target` may write its landing too.

## Negative results

- **A plain directory is not an Antora content source** ("must be a git
  repository"), so `e2e.py` commits each component under `runs/e2e/repos/`;
  `branches: ~` finds no pages there, `HEAD` builds the working tree, so a
  planted change needs no commit. A block image macro inside a paragraph
  renders as text.
- **The fixture's whole build is given a github.com `origin`**, so its pages
  carry the edit link production's do (I15) and Resolve finds pages by their
  facts; built without, every xref into another component stayed unresolved
  before D57.
- **gitlab's job API answers 401**, so `ui-bundle.json` takes the newest
  successful `main` pipeline's sha.
- **`main.<table>` is no catalog qualifier**: the opened DB's catalog is
  named for its file, so the local side is the schema `main`, an attached one
  `<alias>.main`. **`started_at::DATE` does not cast** a `TIMESTAMPTZ`;
  `::TIMESTAMP::DATE` does.
- **Placement forced by the module graph** (`module_graph.py`): the mapping
  cannot be a fetcher's, `fetch` sitting below `ci`; the run's methods, the
  frontier's `Baseline` and Resolve cannot sit in `crawl`'s `baseline` child
  without a cycle; a `gate` call from `query` with `links` called from `gate`
  is a cycle, hence `report::links`.
- **Routing by the prefix is wrong**: a URL under it the build lacks and the
  baseline does not know is live's (D22), not a local 404; `Routed` routes by
  what the build holds.
- **Copying at claim time doubles a landing page**, `targets` having no key;
  the copies go last, and `Msg::Copy` flushes the appenders first.
- **A column marking a resolved link** would be schema 5, which D43 makes
  every baseline refuse; the refspec is the evidence.
- **"A page not in the build", read literally**, counts a deleted page's own
  links as inverse breakage; the linking pages are those outside every held
  prefix (D56). Seeding only the baseline's targets under held prefixes was
  refused: the held set is known only after the crawl.
- **"Not `ok`" as the rule for an unresolved xref** fails the gate on an
  `error` or `unchecked` URL; rule 1 takes `broken_url` alone (D57). Asking
  the chain inside Resolve is out: Resolve sees only the baseline (D38).
- **Trimming the baseline** (the chrome nav links on every page) is not needed
  at 5.9 MiB gzipped.
