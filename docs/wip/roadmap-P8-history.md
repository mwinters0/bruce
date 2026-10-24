# P8 — Run-history reports

What P8 does and why, settled in the grilling that began 2026-10-23. The
decisions are in [`../design/decisions.md`](../design/decisions.md) and are
cited here, never restated. The checklist is `../status/STATUS.md`'s.

## Intent

Reports over two runs beyond `diff`: `changed`, the pages whose content hash
moved (D15), and `anchors-removed`, the anchors a page lost while links still
name them (the roadmap's sketch). **And an HTML report** (maintainer,
2026-10-23), below.

## The HTML report, as the maintainer asked for it

Stated 2026-10-23 (maintainer), before the grilling's questions:

- **Optional**: the reports are still text and JSON; HTML is asked for.
- **A static HTML file carrying the day's date.**
- **Lightweight, with no external JavaScript or CSS**; it should look like a
  professional data report.
- **Actionable**: for instance, the pages ranked by how many broken outbound
  links they hold, each with a link to the live page and to its source on the
  forge.
- **CI keeps only the most recent reports, but they may be copied out into an
  archive**, so a report must make sense read in either place.
- What the report holds, and how, is the agent's to propose.
- **The day's report is also written as a dated JSON file**, which the HTML
  might read, so it can be loaded into another database and graphed over
  indefinite retention, where the CI database is truncated from time to time
  (maintainer, 2026-10-23).

## The HTML report is P8's

Settled 2026-10-23 (maintainer). **P8 holds the HTML report and the two-run
reports together**, the HTML being the phase's face: `changed` and
`anchors-removed` are what let it say, of a link broken since the last run,
whether the linking page changed or its target lost the page or anchor, and
so whose repo broke it. The slices put the HTML over one run's data first,
so it is reviewed on a real en-US crawl before the two-run sections land.
Rejected: a phase of its own, which would ship a report unable to say who
broke a link.

## One document per run, not an HTML format per report

Settled 2026-10-23 (maintainer). **The HTML is one self-contained document
over a run**, composing what the reports hold into headline figures, ranked
tables and drill-downs; every report's text and JSON stay as they are (D6).
Rejected: `--format html` on each of the eleven reports, eleven thin pages
with no ranking across them, where JSON already serves a script and text the
terminal.

## Crawl runs only

Settled 2026-10-23 (maintainer). **A crawl run gets the document; a
`check-dir` run does not.** The gate's sections already reach its CI log, and
a CI job has no `upload-artifact` to keep a file (below, "Facts the grilling
found"). Reopens: a consumer for the gate's page, which reuses the renderer.

## The HTML is rendered from the JSON's data, with no script

Settled 2026-10-23 (maintainer). **Both files are written from one document
in memory: the JSON is the data, and the HTML is that data rendered by
`bruce`**, plain HTML and inline CSS with no JavaScript, drill-downs as
`<details>`, navigation as in-page anchors, rows written already ranked.
Each file stands alone wherever it is copied. Rejected: the HTML fetching
the JSON beside it, which a file opened from disk cannot do (below, "Facts
the grilling found"); the JSON embedded as a data island and rendered by an
inline script, a blank page without script and the data twice. Reopens: a
reader missing a sortable or filterable table, the one thing a script would
add.

## The JSON is every row, under a format version of its own

Settled 2026-10-23 (maintainer). **The JSON holds the aggregates (totals,
counts by page, repo and component) and every broken and error row**, so a
database loading the files can say how long a link has been broken and
which breakage is oldest, after the CI database has pruned the runs that
knew. **It carries a `format_version`**, apart from the database's
`schema_version` (D43), changed only by a change a loader would misread,
an added field being none. **It carries its own context**: the roots, the
run, its start and finish, the tool version and the run it was compared
with, so a file out of an archive needs nothing beside it. Rejected:
aggregates alone, which cannot date a breakage once the runs are pruned.
Sized from the en-US crawl, about 300 KB a run; every locale's weekly run
perhaps 17 MB, should the breakage grow with the locales.

## `bruce report digest`, named by the run's day

Settled 2026-10-23 (maintainer). **`bruce report digest --out DIR`** writes
`DIR/<stem>-YYYY-MM-DD.json` and `.html` (`--format` choosing one or
both), over the run `--run` names or the latest finished one. **The date is
the run's start, in UTC**, so a digest made again later is named for the
same day. **The stem is `bruce` unless `--name` says otherwise**, so the
nightly en-US and the weekly every-locale digests can share an archive
directory. **A second run on the same day overwrites the first's files**:
the file is the day's report, and the run inside says which. A stable
`latest` copy for a published URL is the deployment script's (P9).
Rejected: a flag on `bruce crawl`, which could not regenerate a digest of an
earlier run and puts a report into the crawl, where D6 keeps none.

## What the document holds

Settled 2026-10-23 (maintainer), numbered as below. **The HTML shows them
in the order 1, 2, 3, 5, 6, 4, 7, 8, 9**, the pages first, then the repos,
then the targets, and **every section after the figures is a `<details>`
that starts collapsed** (maintainer, 2026-10-24, reading the en-US digest:
[`../status/history/2026-10-24.md`](../status/history/2026-10-24.md)).
The JSON keeps its own order.


1. **The header**: the roots, the run's start, finish and duration, its id,
   the tool version, and the run compared with, or that there was none.
2. **The headline figures**: pages and links checked, `broken_url`,
   `broken_anchor` and `error`, each with its change since the compared run.
3. **Since the last run**: the breakage new and fixed, each new row saying
   why: the linking page changed, the target lost the anchor, or the target
   page is gone or moved, with the source link of the page responsible.
4. **Fix once, fix many**: the broken targets ranked by the broken links
   naming them, then by their pages, each with the target page's own source
   where the site holds it, the repo that would restore an anchor. On en-US it
   leads with the 266. The HTML lists the targets named by more than one
   link and says how many the rest are, each in section 5's drill-downs; the
   JSON holds every target.
5. **The pages ranked by broken outbound links**: title, live page, source,
   counts, and a `<details>` of each broken link (raw href, the target as a
   link, fragment, reason, status).
6. **By repo**: the repos ranked by the broken links on their pages, each a
   link to the repo and in-page links to its pages' rows in 5 (D17).
7. **Anchors removed**: the pages that lost anchors links still name.
8. **Errors**, apart from breakage (D16).
9. **Housekeeping**, collapsed: redirected links to update, the sitemap's
   entries no page was fetched for, orphans,
   unlisted pages, and the unchecked counts by reason, **each reason
   explained, with the hosts or path prefixes it covers most, and the external
   hosts whose links were checked**, all read off the run's rows (2026-10-24,
   maintainer: a reader cannot know what `out_of_scope` means, or which hosts
   were allowlisted).

**The digest counts every kind**, `kind` a column, where `report` counts
`anchor` alone unless asked (D13): a broken image is on a page someone is
fixing anyway. **No section by component**; the JSON carries that aggregate
for graphing, the repo being the grouping someone acts on.

## The comparison is the run before, or a baseline file

Settled 2026-10-23 (maintainer). **The two-run sections compare with the
finished crawl run before the digest's**, `--from N` naming another, **or
with a baseline file, `digest --since FILE|URL`**, imported as a run of its
own (D21, D52): a deployment keeping no history compares with the baseline
it published the day before. With no run to compare, the sections say so
and the JSON's `compared_with` is `null`. Whether P9's nightly passes
`--since` is P9's script's, for its re-read. Rejected: comparing with the
previous digest's JSON, which holds no hash or anchor; leaving a deployment
without history with no comparison.

## A hash records its scope

Settled 2026-10-23 (maintainer), draining three entries of the inbox. **`changed`
reads only the pages a root covers** (`pages.root` set), so the forge's
pages, `generic` and hashed by its `[role=main]` region, never appear: between two en-US crawls 3
hours apart 137 of the 138 pages whose hash moved were the forge's
(`runs/p4.10-en-us-20261019-2125/combined.duckdb`). **A page records which
scope its hash covered**, `root`, `region` or `whole`, and **a run its
roots' `hash_root` and `hash_exclude`**: a schema change, to version 5
(D43). A page whose scope differs between the two runs is listed as its
scope changed, not as changed, and a page hashed `whole` under a root that
sets `hash_root` is flagged as missed by it, the misconfiguration D15 keeps
visible. P9's guide keeps the profile's region (`roadmap-P9-deployment.md`,
"The guide's hash scope is the profile's region"). Rejected: a note where
`runs.config_hash` differs, which cannot tell a scope edit from a rate edit.

## A new break is laid at the door of what the comparison shows

Settled 2026-10-23 (maintainer). **Section 3 attributes a link broken in
the run and not in its comparison by what the comparison held**, never by
a hash:

| In the comparison | Cause | Source shown |
|---|---|---|
| no linking page | new page | the linking page's |
| the page, not the link | link added | the linking page's |
| the link `ok`, its target now 4xx or gone | target removed | the target's, as the comparison recorded it |
| the link `ok`, its target now redirected elsewhere | target moved | the target's |
| the link `ok`, its target without the anchor | anchor removed (an `anchors-removed` row) | the target's |
| the link `error` or `unchecked` | first judged now, not a regression | the linking page's |

`changed` stays a report of its own, its count of pages in the header.
Rejected: "the linking page changed", read off the hash, which blames a
page for a break its edit did not make.

## Every row, until every locale is measured

Settled 2026-10-23 (maintainer). **The HTML holds every row**: en-US's 324
pages and 828 rows are small, and every locale's size is unmeasured. P9's
every-locale crawl measures its digest's size and how it renders, and
reopens this where either is bad, with whether translations fold into their
en-US source. Rejected: a cap chosen before any every-locale data exists.

## How it looks, and what proves it

Settled 2026-10-23 (maintainer).

- **Dark, in colours near Fedora's** (maintainer): dark on every screen,
  whatever the viewer's `prefers-color-scheme`, and light only in print, one
  palette to get right. Fedora blue
  `#51a2da` and dark blue `#294172` as accents, approximately. A system
  font stack and no web font; a print stylesheet opening every `<details>`;
  readable at a phone's width; numbers right-aligned in tabular figures;
  every link absolute; times in UTC, the date in the `<title>`.
- **Checks**: a test that the document has no `<script>`, no stylesheet
  `<link>` and no external `src`; a golden file of the digest over
  `mise run e2e`'s fixture site; a test holding the JSON's fields to its
  `format_version`'s documented shape.
- **A slice with no code: the maintainer reviews a digest of a real en-US
  crawl**, after the one-run digest and before the two-run sections, the
  verdict and what it spawns in that slice's notes.

## An unresolved xref carries a hint

Settled 2026-10-23 (maintainer), draining the inbox's D20 entry. **The
`antora` profile's views (D38) pair each `unresolved_xref` link with the
nearest resource ID among the run's pages**, by DuckDB's own string
similarity above a floor, and `broken` and the digest show it as a `hint`
column, a field added to `broken`'s JSON. A site with no Antora pages gets
none. The measure and the floor are chosen on en-US's 92 rows by the slice
landing it, whose notes give the hit rate. Rejected: leaving it out, for a
tenth of the breakage whose fix is often one word.

## A release names its schema change itself

Settled 2026-10-23 (maintainer), draining the inbox's release-notes entry.
**`release.yml` compares `SCHEMA_VERSION` at the tag with the latest
published release's**, read from that tag's `src/store/schema.rs` by
`git show`, and where they differ the body it publishes says so, both
numbers and that the old database is refused (D43). `scripts/workflows.py`'s
tests rehearse it. Rejected: a notes file in the tag's tree with a check
that it mentions the change, a hand step a machine could do; a hand edit
after Publish, which the unattended Publish forgets.

## A link that cannot work is broken

Settled 2026-10-24 (maintainer), reviewing the en-US digest (8.6's notes,
"Finding 4"). **An unchecked link that cannot work is broken, and every
report counts it so**, the gate included:

- **A URL on a root's host in no root's scope is asked once, its links not
  followed, as an allowlisted URL is (D22), unless it is another locale's
  page**, which stays `unchecked`, `out_of_scope` (D10). Another locale is
  learned from the `antora` profile's language menu (`a.language`, 68 a page
  on docs.fp.o): the path prefix where a menu link's URL departs from its
  page's. A site without the menu has every such URL asked. On en-US that is
  11 URLs, among them `https:/kubernetes.io`, which resolves to a 404 on
  docs.fp.o, and the slashless home link, a 301 the housekeeping's redirects
  then list. Rejected: locales named by globs in the config, one more list to
  keep as locales are added; judging an href malformed without asking, which
  misses a well-formed link to a missing page.
- **A scheme that works on a public web page stays `unchecked`, `scheme`**:
  `mailto`, `tel`, `irc`, `ircs`, `xmpp`, `appstream`, `ftp`, `news`,
  `magnet`, `sms`, and `data` and `javascript`, which work in the page
  itself, an embedded image or a script's link (added 2026-10-24, maintainer;
  8.12 lands it). **Any other is broken**, `file:` and a mistyped
  `hhttps:` among them, no reader of the site being able to follow it.

## The allowlist names the Fedora and Red Hat sites

Settled 2026-10-24 (maintainer), reviewing the en-US digest. **An
`allowlist` entry `*.<domain>` names every host under the domain**, the
domain itself being an entry of its own; **a root's own host is never matched
by one**, keeping D10's other locales unasked under
`"*.fedoraproject.org"`. **The deployment guide's allowlist names Fedora's
and Red Hat's sites**: `fedoraproject.org`, `getfedora.org`,
`fedorapeople.org`, `fedoramagazine.org`, `fedorainfracloud.org`,
`fedorahosted.org`, `pagure.io`, `centos.org` and `redhat.com`, each with
its subdomains. En-US links to about 150 hosts under `fedoraproject.org`
alone; a link to a staging or internal one then shows as broken or an error,
which is what it is to a reader. Rejected: the twenty hosts most linked,
named one by one, a list gone stale as hosts come and go.

## Where it lives

Settled 2026-10-23 (maintainer), on the concept map's layering (D39):

- **`report/digest/`**: the document built from SQL over the run and its
  comparison in the parent, the JSON and its `format_version` in `json`,
  the rendering in `html`, section 3's attribution in `cause`.
- **`changed` and `anchors-removed`** are `report` children as `diff` is,
  and `bruce report` kinds over `--from` and `--to`, in text and JSON.
- **`--since`** fetches through `cli::check_dir::file`, which moves up to
  `cli::file`, two subcommands sharing it; the import is `store::baseline`'s
  (D52).
- **The hash scope**: `extract::hash` says which scope it hashed, the
  `store` writer records it, `store::schema` is version 5.
- **The hint**: `profile/antora`'s views.
- **The release's schema line**: `release.yml`, rehearsed by
  `scripts/workflows.py`.
- **A link that cannot work**: `crawl::frontier`'s plan, the locales
  learned in `profile/antora`.
- **A domain in the allowlist**: `config`, where the allowlist is parsed and
  matched; the guide's list in `docs/manual/deploy.md`.

## The slices

Settled 2026-10-23 (maintainer), ordered so each shows the next one's
mistakes. **8.1 lands the schema first** because the en-US crawl it
launches, under schema 5, is the evidence every later slice reads, `bruce`
refusing the schema-4 databases `runs/` holds; **8.6 is the maintainer's
review**, after everything over one run and before the two-run sections; and
**8.8's evidence is a second en-US crawl** compared with 8.1's. No closing
review: P9's 9.6 reviews the en-US reports, P8's among them. **8.9 closes the
phase by comparing Servo's renders of the digest with Chrome's headless
shell's** (maintainer, 2026-10-23): Servo checks the rendering because it
keeps the dev dependencies light, and 8.9 says whether it is enough, a
Chrome spot check being the maintainer's to request meanwhile.
**8.6's review found 8.4.1 to 8.4.4 and 8.10 to 8.13** (2026-10-24), and
**resumes on 8.14's second en-US crawl**, made after all of them, which 8.8
then compares with 8.1's, its notes telling the rows they reclassify or
newly check from new breakage. The checklist lists 8.6 after them.

## Facts the grilling found

Found 2026-10-23.

- **The latest en-US crawl has 827 broken links on 324 pages**
  (`runs/p5.2-en-us-20261020/en-us.duckdb`, `report broken`): 610
  `broken_anchor` and 217 `broken_url`, every one on a page with a
  `source_repo`, across more than 40 repos, led by fedora-legal-docs (270, on
  GitLab), commops (109), infra (72) and quick-docs (69). No `error`. The
  three legal license-list pages hold 266 of them (`report stats`).
- **One anchor is a third of the breakage**: 266 of the 827 links name
  `legal/license-review-process/#_what_is_licenseref`, from the license
  lists; the 827 name 494 distinct targets, and the next most named has 7.
  One broken image, no other kind broken (`report broken --kind all`).
  The same run has 61 redirect landings over 90 links, 74 orphans and 13
  unlisted pages.
- **92 of the 827 broken links are `unresolved_xref`** (D20), each an
  Antora `xref:` rendered as `#<refspec>`, such as `#projects::index.adoc`
  on `commops/events/sop-ambassadors/`.
- **The release body is one fixed sentence** (`.github/workflows/release.yml`,
  Publish, `-f body=…`, `generate_release_notes=true`), whatever the tag.
- **2,152 of the 2,320 pages a root covers record their source**, and
  `pages.source_url` is the forge's view of the file at its branch
  (`…/src/branch/f37/modules/…/Installation.adoc`), a link a report can
  print as it is (D36).
- **P9's dev deployment keeps no history**: its nightly job crawls into a
  fresh DB (`roadmap-P9-deployment.md`, "The scheduled job is one script,
  and history lives on a disk"), so a report there has no earlier run to
  compare with; the weekly every-locale crawl on the maintainer's machine
  keeps 30 runs.
- **A page opened from disk cannot read a file beside it**: Chromium refuses
  `fetch()` of a `file://` URL, and Firefox since 68 gives each local file
  an origin of its own (`privacy.file_unique_origin`), so a page that loads
  its data at runtime works only when served over HTTP.
- **No workflow may use a third-party action** (`../design/roadmap.md`,
  "Standing rules"), so a CI job has no `upload-artifact` step to keep a file
  it writes.
</content>
</invoke>
