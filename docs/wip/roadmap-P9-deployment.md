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

## P9 follows P8

Settled 2026-10-21 (maintainer). **P9 is specified and waits for P8**, the
run-history reports. P8 is likely to change the schema (its inbox asks
whether a run records its hash scope), and a 0.x release migrates nothing
(D43), so a
deployment before P8 would discard its history at the next release. P8 needs no
deployment, its evidence being local en-US crawls, and nothing P9 builds
serves anyone before a deployment exists. When P9 is sliced, the grilling
re-reads this spec against what P8 changed.

## The guide's hash scope is the profile's region

Settled 2026-10-21. **The guide's roots set neither `hash_root` nor
`hash_exclude`**, so an Antora page hashes `article.doc` (D15), and the guide
says why in a sentence: a whole-page scope takes in the nav, which lists a
component's pages, so adding one page would change every page of its
component, and a UI-bundle update every page of the site. Dropped: the
extractor's grilling's intent that the guide show a whole-page scope less
`hash_exclude = ["footer"]`.

## A partial build's site-root refresh is not the production page

Settled 2026-10-21. **Where a build's `index.html` at the prefix is a
meta refresh, the gate treats the prefix as not built**, so a built page's
home link is answered from the baseline (D22's "under the prefix and not
built") and is not an introduced `redirected` row. Today 236 of quick-docs's
239 redirected rows are that link (`runs/p5.12-quick-docs/published.json`),
burying a real one. This amends D22 for that one file; the slice that lands
it writes the amendment. Rejected: the guide telling a repo to expect the
rows; a report filter dropping redirected rows to the prefix, which hides the
symptom and leaves the run's data wrong.

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

**Runs stay en-US only until P8's run-history reports exist** and the
maintainer has judged the reports acceptable on en-US runs (maintainer,
2026-10-21). P8 lands first (above, "P9 follows P8"), so **the every-locale
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

Settled 2026-10-21. **A deployment repo on the maintainer's GitHub (say
`mwinters0/bruce-fedora-docs`) holds the guide's `bruce.toml` and a scheduled
workflow** that crawls en-US nightly, exports and gzips the baseline, and
publishes binary, config and baseline on that repo's GitHub Pages, never on
releases, an immutable release refusing a nightly overwrite. The crawl's
1:48 fits GitHub's six-hour job; **the weekly every-locale crawl runs on the
maintainer's machine**, its 20 hours fitting no runner. Mirrors of content repos
on the same GitHub run the guide's CI workflow against the Pages URL: **that
is P9's closing evidence**, as quick-docs' build was CI mode's. The slice standing
it up measures whether Anubis challenges a GitHub runner's address, which no
crawl from here has met. D41 holds: the deployment lives outside this repo.
Rejected: crawling on the maintainer's machine alone.

## The guide's new parts are executed, as its config is

Settled 2026-10-21 (D42 extended). **`mise run sim` extracts the guide's
workflow and scheduled-job script too, and checks them**: the workflow
parses as YAML and has no `uses:` step, the no-third-party-actions rule's
check, and every `bruce` command line in either parses against the CLI.
**`mise run e2e` runs the CI job's command steps, Antora's install excepted,
against its fixture site and baseline.** The dev deployment's real runs are
the closing evidence. Rejected: the static checks alone, which leave the CI
section free to drift from the tool.

## The scheduled job is one script, and history lives on a disk

Settled 2026-10-21. **The guide gives one script that both schedules run**:
`crawl`, then `export-baseline --force` and gzip, then the binary,
`bruce.toml` and `baseline.duckdb.gz` staged in a publish directory, then
`prune --keep 30` (about 1 GB of en-US runs at 33 MB each,
`runs/p5.2-en-us-20261020/en-us.duckdb`). A crawl exiting non-zero stops it
before anything is published. One sequence is what keeps `prune` off a DB a
crawl holds (`../manual/prune.md`, "Not beside a crawl"). Moving the
directory to a web server is the deployer's, left out. **The guide schedules
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
checked, real breakage `out_of_scope` today. CI's config stays the nightly
en-US one. Rejected: 69 roots, a list kept by hand.

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

## The slice order the grilling proposed

Proposed 2026-10-21, to be re-checked against what P8 changed when P9 is
sliced; not yet a checklist.

1. **9.1 The site-root refresh**: `ci`, D22 amended, the concept map's
   clause, `scripts/e2e.py`'s `UNCHANGED_REDIRECTED` reversed.
2. **9.2 The guide's scheduling and publishing**: the script, the systemd
   units, `bruce-locales.toml`, `per_second = 3`, the hash-scope and
   faster-rate sentences, `sim` checking the script's `bruce` lines.
3. **9.3 The guide's CI section**: the workflow, `sim`'s YAML and `uses:`
   checks, `e2e` running its steps, the Antora pin checked against
   production's.
4. **9.4 The dev deployment**: the deployment repo, its nightly workflow and
   Pages; whether Anubis challenges a runner and the crawl's time on Actions,
   the guide corrected where they show it wrong.
5. **9.5 The closing evidence**: a quick-docs mirror running the guide's
   workflow against the Pages URL, a planted break failing and a clean
   change passing.
6. **9.6 The maintainer's review of the en-US reports**, P8's included: no
   code; the verdict and what it spawns in the notes.
7. **9.7 The every-locale crawl**, on the maintainer's machine after 9.6's
   verdict, and the guide's locale figures.
8. **9.8 The release P9 closes on**, and the README's CI section corrected.

Every tag push and every new repository (9.4, 9.5, 9.8) is the
maintainer's: a round stops there, as at 9.6's verdict.

## Facts the grilling found

Found 2026-10-21, for the decisions above.

- **docs.fp.o has 69 locales, and each carries en-US's pages.** A
  quick-docs page's language menu links 69 (en-US among them); 12 of 12
  sampled quick-docs paths answer 200 under both `fr/` and `tzm/`. No
  locale's pages are in a sitemap: every locale's `sitemap.xml` repeats
  en-US's index (I1), so a locale root is found by links alone.
- **An en-US crawl is about 3,200 docs.fp.o requests and 4,100 to the
  forge.** `runs/p5.2-en-us-20261020/en-us.duckdb`: 3,186 docs.fp.o and
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
- **docs.fp.o's `/` is a 200 meta refresh to `./en-US/`.** The en-US run's
  docs.fp.o targets begin with 72 distinct first segments: the locales,
  2,307 targets apiece, and a few others (`infra`, `f28`, `fedora-project`),
  `out_of_scope` today.
