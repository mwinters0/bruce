# P9 — The deployment guide

What P9 does and why, settled in the grilling of 2026-10-21 that split the
release from it. The decisions are in
[`../design/decisions.md`](../design/decisions.md) and are cited here, never
restated.

## Intent

A deployer can run `bruce` for a site from the manual alone: the release
binary (D23), and the deployment guide's remaining parts, scheduling a crawl,
publishing a baseline for CI (D21), and a content repo's CI job, verified
against the simulated deployment (D41, D42). **The deployment will soon check
every locale, not only en-US** (maintainer, 2026-10-21), so each part is
designed for a site of many locale roots, not for one.

**P9 builds whatever a CI installation needs, then stands one up for
docs.fp.o as the exemplar and its validation, and only then is the next
full version cut** (maintainer, 2026-11-02). Code changes being anticipated
(the site-root refresh above all), no release is cut before P9's rc (below, "Prereleases while
the deployment is stood up").

**An installation must survive a schema change and must not grow without
end** (maintainer, 2026-11-02): below, "Schema changes and retention".

## P9 follows the run-history reports

Settled 2026-10-21 (maintainer). **P9 follows the run-history reports**,
which changed the schema, a run recording its hash scope (D15), and a 0.x
release migrates nothing (D43), so a deployment before them would have
discarded its history at the next release. They needed no deployment, their
evidence being local en-US crawls, and nothing P9 builds serves anyone before
a deployment exists. When P9 is sliced, the grilling re-reads this spec
against what they changed (below, "What the run-history reports left").

## P9 follows the site policy

Settled 2026-10-25 (maintainer). **P9 follows the site policy** (D90), which
extends the deployment's config (the gate's `fail` list, D83; the severity
map, D82; the digest's grouping, D86) and changes the schema, a run recording
its policy (D78): the guide is then written once, and a deployment is not
stood up on a schema the next release discards (D43). **The guide's config
sets `[digest]` `group_by = "antora.component"`**, the core grouping by
nothing by default (D86). When P9 is sliced, the grilling re-reads this spec
against what the policy added (below, "What the site policy left").

## P9 follows the declarative extraction rules and the digest's view

Settled 2026-10-31 (maintainer). **P9 follows the declarative extraction
rules and the digest's view**: the maintainer does not
consider `bruce` ready for a content repo's CI until both have landed. The
guide then documents their config once, as it does the site policy's. When P9
is sliced, the grilling re-reads this spec against what they added.

## P9 follows the redirect facts

Settled 2026-11-01 (maintainer). **P9 follows redirects read as facts**
and a chain past a site's configured length broken (D100): the guide sets
the limit for docs.fp.o, and the gate's `fail` list decides whether it fails
a content repo's CI. The limit is `max_redirects`, counting every hop, a
delay-0 refresh included (D7, D100), so production's home link is three
hops (below, "Facts the grilling found") and a guide limit under 3 breaks it
on every page, the baseline's included.

## The guide's hash scope is the profile's region

Settled 2026-10-21. **The guide's roots set neither `hash_root` nor
`hash_exclude`**, so an Antora page hashes `article.doc` (D15), and the guide
says why in a sentence: a whole-page scope takes in the nav, which lists a
component's pages, so adding one page would change every page of its
component, and a UI-bundle update every page of the site. Dropped: the
extractor's grilling's intent that the guide show a whole-page scope less
`hash_exclude = ["footer"]`.

## A partial build's site-root refresh is not the production page

Settled 2026-10-21, amended 2026-11-02 (maintainer). **Where a build's
`index.html` at the prefix is a meta refresh, the gate treats the prefix as
not built**, so a link to it is answered from the baseline (D22's "under the
prefix and not built"), landing where production's does, its hops and its
fragment checked as production's. This amends D22 for that one file; the
slice that lands it writes the amendment. It removes no gate row: the
build's refresh and production's `/en-US/` both redirect, so what keeps a
built page's home link out of `redirected` is the key (next section).
Rejected: a report filter dropping redirected rows to the prefix, which
hides the symptom and leaves the run's data wrong.

## The gate matches a link with its final `/` toggled

Settled 2026-11-02 (maintainer). **The gate's key matches a CI link to a
baseline link from the same page whose target differs from its own only by
a final `/`**, wherever it keys: the `baseline` state, `introduced` and the
breakage already in the baseline, and `redirected`. Production's `site.url`
has no final `/`, so its pages link home at `.../en-US`, where a partial
build, having none (I11), links `.../en-US/`; both are redirected, and 236 of
a quick-docs CI run's 239 redirected rows were that link, burying a real
one. A match hides a CI link only where the baseline's was in the same
state, so a `/foo` that 404s where the baseline's `/foo/` answered is still
introduced. Rejected: the toggle in `redirected` alone, leaving the home link
introduced under a `max_redirects` that breaks it; matching a redirected
link by its landing, which hides an edit from one outdated href to another
landing alike, the edit `redirected` exists to show; the guide telling a
repo to expect the rows, or to build with `site.url` (I15, D53).

## The site-root refresh rule is `ci`'s

Settled 2026-10-21. **`ci`, which says which file is which URL, serves a
top `index.html` that is a meta refresh at its own path,
`<prefix>index.html`, not at the prefix**, so the prefix is not built and
the baseline answers it (D22); the file stays a seed, D56's targets
unchanged. No module is added: the concept map's CI line gains the clause.
`Held` never sees the file, which carries no `antora` facts. Rejected: a
held-set exception in `crawl::baseline`.

## CI fetches the binary from beside the baseline

Settled 2026-10-21. `check-dir` refuses a baseline of another schema (D43),
so the scheduled crawl and every content repo's CI must run binaries of one
schema. **The deployment publishes the binary it crawled with beside the
baseline and the config (D27)**, and a content repo's CI fetches all three
from there, never naming a version. The guide also shows pinning a release,
for a deployer who prefers it. Rejected: one version pinned in every job and
bumped together by hand; a published version file naming the release to
fetch, a second fetch for the same lockstep.

## Every locale is a weekly crawl of its own

Settled 2026-10-21. **docs.fp.o is crawled at three requests a second**
(maintainer, for the site: the guide's `per_second = 3`), the forge staying
at its `Crawl-delay` of one; `bruce`'s own default is unchanged (D18, D40).
Every locale at three a second is about 220,000 / 3 ≈ 20 hours (below,
"Facts the grilling found"), so **the guide schedules two crawls into two
DBs**: en-US nightly, the run CI's baseline is exported from, and every
locale weekly in a DB of its own, D28's one writer a file keeping the long
crawl out of the nightly one's. CI's baseline stays en-US's. Rejected: a
rotation of a few locales a night, each run blind to the links into the
locales it skips.

**The every-locale crawl's DB, digest and baseline stay on the maintainer's
machine** until 9.9 measures its digest (maintainer, 2026-11-02); the guide
publishes them as a second directory, `locales/`, only if they fit Pages'
1 GB beside 30 nights of en-US's. Rejected: publishing from the start, on a
size nobody has measured.

**Runs stay en-US only until the maintainer has judged the reports
acceptable on en-US runs** (maintainer, 2026-10-21), so **the every-locale
crawl is a P9 slice**, run after that verdict, on the maintainer's machine,
measuring the figures the guide's locale section states.

## A crawl's hosts overlap

Settled 2026-10-21. Scheduling per host (D61), so a waiter for one host's pace
cannot fill the worker slots, should take the nightly en-US crawl from 1:48
towards the forge's 70 minutes. It has not been re-timed; the guide quotes
whatever crawl time is measured when it is written. It does nothing for the
locales, nearly all one host.

## One workflow for the forge and GitHub, with no third-party actions

Settled 2026-10-21 (maintainer). **The guide's CI example is a workflow
that runs on Forgejo Actions, the production target, and on GitHub Actions**,
where the maintainer runs Fedora's docs CI while iterating in development.
**It uses no third-party action** (`../design/roadmap.md`, "Standing
rules"), and P9 extends that rule's check to the workflow the guide tags. Rejected: a bare shell script each repo wraps
itself, which leaves every repo writing the workflow anyway.

## The CI job runs on Fedora 44 and installs Antora

Settled 2026-10-21. **The system the guide's CI job targets is Fedora's
latest container, 44**, not docs-fp-o's image, fedora-minimal 42, which is
past its end of life and rebuilt by nothing. **The job starts from
`registry.fedoraproject.org/fedora-minimal:44`, installs node, git and tar,
and `npm install`s a pinned Antora**, every run, about a minute inside the
forge's ten; the guide says the same steps can be baked into an image. The
pin is the one `scripts/e2e.py` holds, 3.1.15, and whether production builds
with it is checked when the guide is written. Rejected: an image the
deployment must build and push before the example works.

## The dev deployment is a GitHub repo of its own

Settled 2026-10-21. **A deployment repo on the maintainer's GitHub,
`mwinters0/bruce-fedora-docs-ci` (created empty, 2026-11-02), holds the guide's `bruce.toml` and a scheduled
workflow** that crawls en-US nightly and publishes the staged directory
(below, "The published directory is the contract") on that repo's GitHub
Pages, **from a branch holding one orphan commit, force-pushed each night**
(maintainer, 2026-11-02): the site changes as a set in one deploy, the digest's
HTML renders, and the repo does not gain the night's 26 MB, `main` alone
keeping history. The crawl's
1:48 fits GitHub's six-hour job; **the weekly every-locale crawl runs on the
maintainer's machine**, its 20 hours fitting no runner. Mirrors of content repos
on the same GitHub run the guide's CI workflow against the Pages URL: **that
is P9's closing evidence**, as quick-docs' build was CI mode's. The slice standing
it up measures whether Anubis challenges a GitHub runner's address, which no
crawl from here has met. D41 holds: the deployment lives outside this repo.
If CI's fetches near Pages' soft 100 GB a month, CI takes the binary from the
pinned `bruce` release, which has no bandwidth limit. Rejected: crawling on
the maintainer's machine alone; a rolling release (immutability being opt-in,
it could be overwritten), whose HTML downloads rather than renders and whose
assets are replaced one at a time, so a CI job between two fetches a binary
and baseline of different nights; Pages deployed from an Actions artifact,
which takes GitHub's `deploy-pages` actions (the no-`uses:` rule) or the
artifact service's undocumented API.

## The guide's new parts are executed, as its config is

Settled 2026-10-21 (D42 extended). **`mise run sim` extracts the guide's
workflow and scheduled-job script too, and checks them**: the workflow
has no `uses:` step, the no-third-party-actions rule's check, and every
`bruce` command line in either parses against the CLI. **`mise run e2e`
parses the workflow as YAML** (D42, `Rejected`) **and runs the CI job's
command steps, Antora's install excepted, against its fixture site and
baseline.** The dev deployment's real runs are
the closing evidence. Rejected: the static checks alone, which leave the CI
section free to drift from the tool.

## The scheduled job is one script, and history lives on a disk

Settled 2026-10-21. **The guide gives one script that both schedules run**:
`crawl`, then `export-baseline --force` and gzip, then the digest, then the
published directory staged (below), then `prune --keep 30` (about 1 GB of
en-US runs at 33 MB each, 2026-10-20's crawl). A crawl exiting non-zero stops
it before anything is published. One sequence is what keeps `prune` off a DB
a crawl holds (`../manual/prune.md`, "Not beside a crawl"). Delivering the
directory is a step after it (below, "Delivery is pushed or pulled"). **The guide schedules
it with a systemd timer and service**, Fedora's own, and names the one-line
crontab. **The dev deployment's nightly job keeps no history**, crawling into
a fresh DB each night on a runner with no disk to keep; history and `prune`
are exercised by the weekly every-locale crawl on the maintainer's machine.
Rejected: carrying the DB between nights as a mutable release's asset,
persistence a disk-backed host does not need.

## Every locale is one root at the site's top

Settled 2026-10-21. **The weekly crawl's config, `bruce-locales.toml`, a
second file the guide tags and `sim` loads, has one root at
`https://docs.fedoraproject.org/`** with `sitemap` set to en-US's index: the
locales are found by the language menu's links, a locale added later is
crawled unasked, and the stray first segments (`infra`, `f28`, …) are
crawled, where en-US's config asks each once (D10, D67). CI's config stays the nightly
en-US one. Rejected: 69 roots, a list kept by hand.

## Schema changes and retention

Raised 2026-11-02 (maintainer).

**An installation pins the exact `bruce` release its scheduled job runs**
(maintainer, 2026-11-02), `-rc.N` tags while the dev deployment is stood up,
so an upgrade, and with it any schema change, is a commit in the deployment
repo; content repos still fetch the binary from beside the baseline (above).
Rejected: following the latest release, which puts a schema change on an
unattended night; building from source.

**A schema change is met by the guide's script, not a person**
(maintainer, 2026-11-02): every command that opens a DB or a baseline
(`crawl`, `check-dir`, `report`, `prune`, `export-baseline`) **exits 3 for
"another schema version"**, `check-dir`'s 2 keeping its other could-not-run
causes, and a CI workflow treating 3 as it does 2. The refusal is a typed
error of `store::schema`'s, mapped to the status in `cli`. On it the script
moves the DB aside as an archive, crawls into a fresh one, and writes that
run's digest uncompared. **D43's refusal to migrate stands**, for a reason
beyond cost: runs are append-only (D2), and a migrated run lacks what its
crawl never collected (the hops before schema 7, the recorded policy before
6), so the new views would judge it on missing facts and a comparison across
the change would mislead. Rejected: a guide procedure keyed on the release
notes, which an unattended job cannot follow; migrations.

**What grows is bounded by count** (maintainer, 2026-11-02): a history DB by
`prune --keep 30` nightly and `--keep 8` for the weekly every-locale crawl
(two months, its size unmeasured); the dated digests in the published
directory, **30 nights of HTML and other views and 13 months of JSON**
(maintainer, 2026-11-02), the JSON being the report a loader keeps
(`../manual/digest.md`), some 450 MB on Pages with two nights of binary and
baseline; one archived DB, the last schema's, deleted at
the next change; the Pages branch by its single commit. **The scheduling slice
first measures whether a DB pruned nightly holds a steady size**, and if it
does not, the script copies the DB fresh after pruning.

Facts found that day:

- **What an installation accumulates**: the history DB (a disk-backed
  deployer's, and the weekly every-locale crawl's on the maintainer's
  machine), pruned by `bruce prune` (`--keep N`, `--older-than AGE`); the
  digests, one dated JSON and one file per view a day (`../manual/digest.md`,
  "The files"), which nothing prunes; and on the dev deployment, whatever its
  publishing leaves in the repo, a GitHub Pages branch committed to nightly
  growing its git history without end.
- **`prune` checkpoints so the file "can give the freed blocks back"**
  (`store::prune`). DuckDB truncates only trailing free blocks and reuses the
  rest, so whether a DB pruned nightly holds a steady size is unmeasured.
- **A schema change is refused, never migrated** (D43): every command exits 1
  on a DB of another schema, `check-dir` 2, and nothing tells that refusal
  from any other failure. The release body names the change. A digest
  comparing with the last night's baseline file (D73) is refused across one.
- **v0.2.0's archive is 20 MB**; the en-US baseline 5.9 MiB gzipped (above).
- **The dev deployment's nightly keeps no DB** (above, "The scheduled job is
  one script"), so GitHub holds no history: 30 en-US runs, about 1 GB, is a
  disk-backed deployer's total, and an every-locale run, perhaps 69 times
  en-US's 33 MB, the maintainer's machine's.
- **GitHub Pages** (docs.github.com, "GitHub Pages limits", 2026-11-02): a
  published site of at most 1 GB, a soft 100 GB a month of bandwidth (about
  3,800 CI fetches of binary and baseline), a source repo recommended under
  1 GB, a soft 10 builds an hour for a branch-built site. A Pages branch
  committed to nightly gains some 26 MB a night of binary and baseline.
- **GitHub Pages serves each file `Cache-Control: max-age=600`** with an
  ETag (`curl -sI https://pages.github.com/`, 2026-11-02), so for ten minutes
  after a deploy a fetch may be answered from the CDN's cache, file by file.
- **GitHub releases** (docs.github.com, "About releases"): no limit on a
  release's total size or bandwidth, each asset under 2 GiB. **Immutability
  is an opt-in repository setting** (GA 2025-10-28), so a release on a repo
  without it can have its assets replaced; replacing one is per asset, never
  the set at once.

## The published directory is the contract

Settled 2026-11-02 (maintainer). **The scheduled job ends by staging one
directory**: the latest files under fixed names (`bruce.tar.gz`, `bruce.toml`,
`baseline.duckdb.gz`, `digest.json`, `digest.html` and any other view's
file) for people, `digests/` holding the dated ones, and **`SHA256SUMS`**,
in `sha256sum`'s format. CI names no version (above, "CI fetches the binary
from beside the baseline"). **Whoever fetches from it, CI or a puller,
fetches `SHA256SUMS` first, then the files it lists, and runs `sha256sum
-c`**, failing as could-not-run, never as broken links. **The files it lists
are named by their content** (`baseline-<sha>.duckdb.gz`, `bruce-<sha>.tar.gz`)
**and last night's are kept one more night** (maintainer, 2026-11-02), so a
fetcher handed a stale `SHA256SUMS` by Pages' ten-minute cache (above, "Facts
found that day") finds last night's whole set, and one handed a fresh one
fetches names no cache holds. **That holds for CI's three files; the digests
are listed too, the latest under fixed names, so a puller may fail its check in
those ten minutes and retry on its next timer** (maintainer, 2026-11-03,
"Review of the `SHA256SUMS` call" that day). A puller knows there is something new when
`SHA256SUMS` has changed. CI downloads the baseline to check it and hands
`check-dir` the path. Rejected: waiting and retrying on a mismatch, a ten
minutes the forge's ten-minute job cannot spend; failing a PR checked in the
ten minutes after a deploy as could-not-run. The run, version and
schema are `digest.json`'s and the binary's, not the manifest's. It is not the
version file rejected above, which restated CI's lockstep: without it a
fetcher cannot tell a binary and baseline of two nights apart. Rejected: a
`manifest.json`, which every CI image would need `jq` or Python to read,
`sha256sum` being coreutils, in `fedora-minimal`.

**The digest compares with the baseline the publish replaces** (D73's
baseline file), so one script serves a deployment with a disk or none, and a
digest says what changed since the baseline CI last judged by, a night that
failed to publish folded into the next. With none, or one of another schema,
it is uncompared. **The script reads the last published set from a local
directory** (maintainer, 2026-11-02): a disk-backed host's published
directory, or on the dev deployment the `gh-pages` branch's one commit
fetched first, which also carries the dated digests into the next orphan
commit. **A failing digest does not stop the job**: the baseline CI needs is
published, and the job exits non-zero at its end. Rejected: the run before it
in the DB where there is one (D73's default), a second path; fetching the
baseline from the published URL, which loses the dated digests on a host
with no disk.

## Delivery is pushed or pulled

Settled 2026-11-02 (maintainer). **Some deployers push the results when the
job ends; others, docs.fp.o first, need a cron job on the web server to find
and pull them.** So the guide's script stops at the staged directory, and the
guide shows both: a push step the deployer writes (`rsync`, `scp`, a commit;
the dev deployment's publishes to Pages), and a pull script on the server's
timer that fetches the manifest from the published URL and, where its run is
new, the files it lists, checks their sha256s and moves them into place.
docs.fp.o's proxies are rsynced with `--delete` (below, "Facts the grilling
found"), so its pull goes in `rebuild-site.py` or on a path the rsync spares,
a choice the guide leaves to the site's operators. Both scripts are tagged
blocks `sim` checks, and `mise run e2e` runs the pull against a directory it
serves locally.

## The workflows, step by step

Settled 2026-11-02 (maintainer): what each workflow does, the guide's
flowchart drawn from it (9.4).

**The scheduled job**, one script (above, "The scheduled job is one
script"), given the config, the DB, the published directory as last
published (`PUB`) and the keep count; `bruce` on the `PATH` is the pinned
release, its archive beside it.

- **S1 Crawl** into the DB. On 3, the DB moves aside as the one archive
  (replacing the last) and the crawl runs again into a fresh DB. On any other
  failure the job stops non-zero and `PUB` is left as it was.
- **S2 Export** the run as the baseline, gzipped.
- **S3 Digest** the run, `--since PUB`'s baseline where there is one; on 3
  (another schema), again without it. A failing digest does not stop the
  job, so CI still gets its baseline, and the job exits non-zero at the end.
- **S4 Stage** a new directory beside `PUB`: the archive, `bruce.toml`, the
  baseline, the digest under fixed names, `digests/` carried over from `PUB`
  with tonight's added and the oldest past the counts dropped, the files
  `SHA256SUMS` will list under names by content, last night's carried one
  more night, and `SHA256SUMS` last. The new directory then replaces `PUB` by
  rename.
- **S5 Prune** the DB to the count (and copy it fresh, if 9.1 shows pruning
  alone does not hold its size). A DB-less run skips it.
- **S6 Deliver** `PUB`: the deployer's push step, or nothing where the server
  pulls.

**The dev deployment's nightly workflow** (GitHub Actions, on a schedule and
on demand, in `fedora-minimal:44`):

- **D1** Install git, tar, gzip and curl; clone `main` (the config, the
  script, the pinned tag).
- **D2** Fetch the pinned `bruce` release's archive and its `.sha256` from
  `mwinters0/bruce`'s releases, check, unpack. Fetch the `gh-pages`
  branch's one commit (`git fetch --depth 1`) as `PUB`, absent on the first
  night.
- **D3** Run the script with a fresh DB.
- **D4** Commit `PUB` as one orphan commit and force-push it as `gh-pages`;
  Pages serves it.

**The pull script** (a timer on the web server):

- **P1** Fetch `SHA256SUMS`; the same as the local copy, stop.
- **P2** Fetch each file it lists into a new directory, reusing a local file
  whose sum matches (the binary changes only at an upgrade).
- **P3** `sha256sum -c`; failing, stop non-zero, the served copy untouched,
  the next timer trying again.
- **P4** Rename the new directory into place.

**A content repo's CI workflow** (Forgejo and GitHub Actions, on a pull
request, in `fedora-minimal:44`, inside the forge's 10 minutes):

- **C1** Install node, npm, git, tar and gzip; fetch the PR's commit with git.
- **C2** `npm install` the pinned Antora; build the repo's playbook with
  `CI=true` (I15).
- **C3** Fetch `SHA256SUMS`, then the binary, config and baseline it names
  by content; `sha256sum -c --ignore-missing`. Failing, the job fails as could-not-run.
- **C4** Unpack `bruce`; `bruce check-dir` the build against the baseline and
  config.
- **C5** 0 passes; 1 fails the PR's check, the report in the log; 2 or 3
  fails as could-not-run, the message saying the job failed and not the PR.

**An upgrade** is a commit changing the pinned tag. Across a schema change
the next night's S1 meets 3 and starts fresh, S3 meets 3 and is uncompared,
and the new binary and new-schema baseline are published in one set, which
CI's next fetch takes whole.

**The weekly every-locale crawl** is the same script with
`bruce-locales.toml`, its own DB, `--keep 8`, and a `PUB` on the maintainer's
machine, delivered nowhere until 9.9.

## Prereleases while the deployment is stood up

Settled 2026-10-21. **The release P9 needs is tagged `-rc.N` while the dev
deployment is stood up** (D62), and in full when P9 closes.

## What the guide states, from the drained inbox

Settled 2026-10-21, each figure re-measured when its section is written.
**Scheduling**: an en-US crawl took 1:48:22 and 161 MiB under the guide's
config at one a second, the hosts nearly in turn (D61 overlaps them; 3 a
second on docs.fp.o shortens its share); `prune` fails at open on DuckDB's
lock beside a crawl, which the one script rules out. **Publishing**: the
en-US baseline is 27 MB and 5.9 MiB gzipped, so it is published gzipped,
`check-dir --baseline` decompressing a `.gz` path or URL. **CI**: quick-docs'
236 pages checked in 8.9 s, with no external request beyond the forge's
allowlist; the job needs the network, a link on the site the baseline lacks
being asked of production whatever `--externals` says (D22); the build sets
`CI=true`, dropping the `file://` edit link (I15); the config comes from
beside the baseline (D27).

## The slice order

Settled 2026-11-02 (maintainer); the checklist is `../status/STATUS.md`'s. **9.1 gathers
evidence** and lands no code, so each later slice meets what it found.

1. **9.1 Evidence**: a fresh en-US crawl under schema 7, detached (the kept DB
   being refused, and the review and a `max_redirects` trial needing one);
   whether a DB pruned nightly holds a steady size, one run of that crawl
   copied many times under `prune --keep 30`; whether Anubis challenges a
   GitHub runner, a throwaway workflow in the deployment repo fetching a
   sample of docs.fp.o and forge pages under the guide's User-Agent.
2. **9.2 The site-root refresh and the gate's key**: the final `/` toggled
   in the gate's key, `ci` serving the refresh at its own path, D22
   amended, the concept map's clause, `scripts/e2e.py`'s
   `UNCHANGED_REDIRECTED` empty and `PLANTED_REDIRECTED` the alias's row alone.
3. **9.3 Exit status 3**: the typed refusal in `store::schema`, the status in
   `cli`, the manuals' exit tables.
4. **9.4 The guide's scheduling, publishing and delivery**: the script, the
   systemd units, `bruce-locales.toml`, `per_second = 3`, the hash-scope and
   faster-rate sentences, the staged directory and `SHA256SUMS`, retention
   and the archive at a schema change, the digest's comparison, the pull
   script, the workflows' flowchart; `sim` checking the scripts' `bruce`
   lines, `e2e` running the pull against a directory it serves.
5. **9.5 The guide's CI section**: the workflow, its fetch and check
   included, `sim`'s `uses:` check, `e2e` parsing it as YAML and running its steps, the
   Antora pin checked against production's.
6. **9.6 The dev deployment**: `v0.3.0-rc.1`, its notes drafted by
   `/release` for the maintainer to edit (maintainer, 2026-11-02), the deployment repo's nightly
   workflow and its orphan Pages branch, the crawl's time on Actions, the
   guide corrected where they show it wrong.
7. **9.7 The closing evidence**: a quick-docs mirror running the guide's
   workflow against the Pages URL, a planted break failing and a clean change
   passing.
8. **9.8 The maintainer's review of the en-US reports**, the digest
   included: no code; the verdict and what it spawns in the notes.
9. **9.9 The every-locale crawl**, weekly on the maintainer's machine after
   9.8's verdict: history, `prune` and the schema archive exercised, the
   guide's locale figures, and its digest's size and rendering, reopening the
   digest's every-row HTML where either is bad (D64, Reopens) and deciding
   whether the guide publishes its results (above, "Every locale is a weekly
   crawl of its own").
10. **9.10 The release P9 closes on**, its notes drafted the same way, and the README's CI section corrected.

**A round commits and pushes to the deployment repo**, its settings
(Pages, the schedule) included (maintainer, 2026-11-02). Every tag push
(9.6, 9.10) and every new repository (9.7's mirror) is the maintainer's: a
round stops there, as at 9.8's verdict.

## Facts the grilling found

Found 2026-10-21, for the decisions above.

- **docs.fp.o has 69 locales, and each carries en-US's pages.** A
  quick-docs page's language menu links 69 (en-US among them); 12 of 12
  sampled quick-docs paths answer 200 under both `fr/` and `tzm/`. No
  locale's pages are in a sitemap: every locale's `sitemap.xml` repeats
  en-US's index (I1), so a locale root is found by links alone.
- **An en-US crawl is about 3,200 docs.fp.o requests and 4,100 to the
  forge.** 2026-10-20's crawl: 3,186 docs.fp.o and
  4,123 forge targets `ok`, and 154,581 docs.fp.o targets `unchecked`, the
  language menu's links into other locales. So every locale at one request a
  second is about 69 × 3,200 ≈ 220,000 docs.fp.o requests,
  some 61 hours, where en-US alone is under one; the forge's share does not
  grow, each target being fetched once a run (D13). The en-US baseline's 27 MB
  (the inbox) would grow towards 69 times that.
- **`mise run sim` extracts a tagged block of any language** (`scripts/sim.py`,
  `TAG_RE`) but executes only the config load; `scripts/e2e.py` runs only
  the commands it hard-codes.
- **Fedora's docs content repos are on the forge, and most run no PR CI.**
  forge.fedoraproject.org's `docs` org holds 15 repos (quick-docs,
  release-notes, docs-fp-o among them; `api/v1/orgs/docs/repos`), none but
  docs-fp-o with a workflow, and docs-fp-o's is an hourly footer-timestamp
  monitor. Seven content repos elsewhere (`forge/documentation`,
  `infra/docs`, `server/user-documentation`, …) build on `pull_request` with
  Forgejo Actions, `runs-on: podman`, in either
  `registry.gitlab.com/fedora/docs/docs-website/docs-fp-o:latest`
  (fedora-minimal 42, glibc 2.41, built 2025-09-08 and rebuilt by nothing
  since; F42 is EOL) or the upstream `antora/antora`, which is Alpine, musl.
  The release binary, built on Trixie, runs in the first, its glibc floor being 2.41.
- **The forge's runners are per-org, unprivileged Podman, and a shared
  runner's job is capped at 10 minutes** (the Forge Usage Policy, approved
  2026-10-06, `council/tickets` #558). So a content repo's `check-dir` fits
  (9 s, the inbox) and a crawl does not. The policy admits Fedora-specific
  software only, a general upstream belonging on GitHub, GitLab or Codeberg;
  the forge (Forgejo 15.0.10) serves release assets at
  `/<owner>/<repo>/releases/download/<tag>/<asset>`, GitHub at the same
  path, and GitHub's `/releases/latest/download/<asset>` 404s on Forgejo.
- **docs.fp.o is built hourly by an OpenShift CronJob and rsynced to the
  proxies with `--delete`** (`infra/ansible`,
  `roles/openshift-apps/docsbuilding/`, `roles/fedora-docs/proxy/files/docs-rsync`),
  so a file served from docs.fp.o itself, a baseline say, must come out of the
  build's `rebuild-site.py`. The proxies set `anubis: true`
  (`playbooks/include/proxies-websites.yml`); the en-US crawls and curl have
  not met its challenge, a browser-like User-Agent having.
- **docs.fp.o's build output persists; each language's tree in it does not.**
  The CronJob (`50 * * * *`) mounts the PVC `docs-storage` at `/antora/output`
  (`roles/openshift-apps/docsbuilding/templates/cron.yml.j2:30-38`), which the
  proxies' rsync (`55 * * * *`) mirrors. `rebuild-site.py` deletes and replaces
  `/antora/output/<lang>` each run, not atomically, and leaves every other
  top-level entry alone; a failed Antora build exits before copying, the last
  tree staying served (`docs/site-builder` `prod` `87767a7`, `:447-471`;
  `infra/ansible` `bc63ddd`; both moved from pagure to forge.fedoraproject.org,
  2026-11-03). So docs.fp.o's pull persists only in a top-level directory.
- **docs.fp.o's `/` is a 200 meta refresh to `./en-US/`, and `/en-US/` one
  to `/en-US/docs/`.** `/en-US`, the home link of all 2,341 pages of the
  2026-11-02 en-US run, 301s to `http://…/en-US/` and 302s back to https
  before that refresh (`runs/en-us-20261102-2011/en-us.duckdb`). The en-US run's
  docs.fp.o targets begin with 72 distinct first segments: the locales,
  2,307 targets apiece, and a few others (`infra`, `f28`, `fedora-project`),
  each asked once (D10, D67).

## What the run-history reports left

Found 2026-10-25, for the re-grilling.

- **A baseline crawled before a bad scheme was broken, or a stray URL on a
  root's host asked, holds such a link as `unchecked`** (D66, D10). A
  `check-dir` built after either calls the link introduced, or under
  `--externals=baseline` copies the row and stays silent. P9 ships the binary
  beside the baseline it crawled (above, "CI fetches the binary from beside
  the baseline"); a deployer pinning a release against a newer baseline, or
  the reverse, meets the mismatch.
- **An every-locale digest is unmeasured** (D64, Reopens), and three things
  in it may grow: the unchecked groups, about 1,300 rows a run of external
  hosts, listed whole; section 3, where `first_judged` may want folding to a
  count; and the reads of `link_verdicts`, which each of thirteen sections
  makes again, 0.2 to 0.45 s each on en-US. The remedy for the last is reading
  it once into a temporary table the views read by name, a change to how the
  store installs them.
- **The digest's cost on en-US**, a release build: 3.1 s over the 2026-10-24
  crawl, 5.6 to 7.2 s over the 2026-10-25 crawl with D75's marks. The JSON was
  671 KB a run against an estimate of 300 KB, its broken targets and redirect
  landings with their linking pages most of the difference; the HTML of the
  2026-10-24 crawl against 2026-10-23's is 4.0 MB. The crawls are named in
  `../design/decisions.md`'s preamble.
- **`robots_unavailable` means a host answered with no file** (D8), so a run's
  count should be near zero: on the 2026-10-24 crawl it is 1 link,
  `jenkins.fedorainfracloud.org`, whose `/robots.txt` is a 302 onto a name
  that does not answer. A large count means a host failing with 5xx or 429. A
  host down at its first request has every link `error` for the whole run.
- **`digest --since FILE` imports into the database it reads** (D73,
  Reopens).
- **pagure.io is a read-only archive**: its 18 `new_issue` targets answer 404
  and are reported broken. src.fedoraproject.org's `new_issue` redirects to a
  login page that answers 200. The guide adds no `skip` for either, and
  robots.txt already keeps Bugzilla's `enter_bug.cgi` unasked.
- **No real pair of runs has yet differed by content**: `changed` and
  `anchors-removed` were empty between the 2026-10-23 and 2026-10-24 crawls,
  and the 2026-10-25 crawl against 2026-10-24's has no new break, one fixed
  link, two new errors and 16 new redirect landings (D74, Reopens). The
  nightly is the first source of a real one.
- **The review tooling** is `mise run crawl | digest | serve`
  (`scripts/review.py`). The digest's golden file,
  `fixtures/antora/digest.html`, is rewritten by
  `cd scripts && uv run e2e.py --save-digest`; a change of layout shows only
  in a render (`scripts/shot.py`).

## What the site policy left

Found 2026-10-31, for the re-grilling.

- **`[rate]` is `[default]`**, and the loader refuses `[rate]` by name: the
  guide's config section is written under the new name.
- **Without `group_by` the digest's JSON has no groups**, where it listed
  components before; the guide sets it (above, "P9 follows the site policy").
- **The guide can set a host's own timeouts and `slash_probe`**:
  `../manual/config.md`, "Asking", is the reference.
- **A run recorded before the asking settings has no asking rows**, and its
  digest states none; a missing row reads as the default, which is what such
  a run asked with.
- **No crawl has run under schema 6**, so a digest compared across a policy
  change, a trial `--config` and a `none` kind's effect on the ranking are
  tested and never run on a real crawl, and `introduced` and `inverse` have
  run on `mise run e2e`'s fixture alone. The verdicts' cost was measured on
  the 2026-10-28 crawl (D77); time a digest by CPU, not wall clock, and
  alternate the binaries, a loaded machine doubling the first timing. The
  doubled slash's reach on the 2026-10-24 crawl, 554 pages `full` and 183
  `immediate`, was not measured again; hand-edited `policy` rows reproduce it
  on any schema-6 database.

## What the declarative extraction rules left

Found 2026-11-01, for the re-grilling.

- **A link to `download.fedoraproject.org` is judged by whichever mirror the
  request draws.** MirrorManager redirects each request; 7 of 40 for
  `/pub/fedora/linux/development/` landed on a 404, at five mirrors whose
  records are wrong, and the fcix mirrors' robots.txt leaves most of the rest
  `unchecked` (D69). So the link on `releases/branched/` was `unchecked` in
  run 1 of `runs/en-us-20261101-0005/en-us.duckdb` (crawled 2026-10-31) and
  `broken_url` in run 2 (2026-11-01), its page unchanged. A baseline that
  drew the 404 fails a PR adding a link to that URL (D9, D22's `--externals`
  default), and the nightly digest's `new_breakage` comes and goes with the
  draw. The guide sets no `skip` for the host (D50); whether it should is the
  grilling's. Contingent on the mirror records staying wrong; the robots draw
  stays either way.
- **The guide's config is an extension**, `fedora`, `extends = "antora"`
  with `locale_links = "a.language[href]"`, no `generator` and no root naming
  it (D94). `tests/extract.rs`, `scripts/e2e.py` and `examples/sim_config.rs`
  read it from the guide, the one `file=bruce.toml` block holding
  `[[profile]]`, so a re-grilled guide keeps that block whole.
- **A run before the declarative profiles has no `profile.*` rows**, so a
  digest comparing one with a run under the guide lists the extension's rows
  as added (D91).

## What the digest's view left

Found 2026-11-02, for the re-grilling.

- **A deployment's view is a file its job names with `--view`**: no key
  names one and no run records one (D99), so the guide documents the view
  beside the scheduled job's script, not in its config.
- **The palette docs.fp.o's deployment may want back** is `git show
  e30483c:src/report/digest/html/style.css`, its `:root` and `@media print`
  rules, which become the guide's view's `palette` block (D98).
- **A view extending the default sees its own top-level macros and `set`s in
  its blocks**, which minijinja 3.0.5 does and no test holds: the slice
  writing the guide's view adds one if the view relies on it.

## What the redirect facts left

Found 2026-11-02, for the re-grilling.

- **Schema 7 refuses every kept database** (D43), `runs/en-us-20261101-0005/`
  included, `pages.in_sitemap` and `via_links` having gone; `review.py crawl`
  starts a new one. A fresh en-US crawl so comes before any limit is tried
  against docs.fp.o with `report --config` (D84).
- **docs.fp.o's chains**, read off run 2 of that database: 798 redirected
  targets, by hops 646, 131, 14, 6 and 1 (one to five); slash-only, 241 of
  one hop and 31 of two (I4's, through `http://`). The limit breaks only an
  `ok` target, 716 of them: a limit of 1 breaks 139, 31 slash-only; 0, all
  716.
- **A host's limit is the linked URL's host's**, not the landing's (D100):
  the guide says so beside a forge's key.
- **`report_pages` costs a pass over `links`** for every reader (`report_reach`):
  0.3 s against the 2026-11-01 run's 1.18 million links, 1.2 s for both runs
  unfiltered, in the duckdb CLI. The `max_redirects` rule's cost in
  `link_verdicts` was not measured. A report that is slow at docs.fp.o's
  size looks there first.
