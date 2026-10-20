# P5.11 notes: quick-docs against the en-US baseline

Outcome: quick-docs, built as its `site.yml` builds it and with seven changes
planted, was checked against P5.2's en-US crawl (`runs/p5.3-baseline/baseline.duckdb`,
the export of `runs/p5.2-en-us-20261020/en-us.duckdb`). Every planted rule
fired on real content. The unchanged build is not silent: it exits 1 on five
rows, none a change of quick-docs's, which are two defects of the gate, KD8
and KD9. Everything is under `runs/p5.11-quick-docs/`.

## The evidence, and how to reproduce it

- **Sources** (`playbook.yml`, `site.yml`'s with the vendored UI bundle and
  `indexify`): quick-docs `70cd18e` (a clone of
  `forge.fedoraproject.org/docs/quick-docs`; `pagure.io`'s now holds only a
  README), team-docs `e896044`, user-documentation `ca833e4`, operations/docs
  `98ef1c0`, all `main` on 2026-10-20. Antora 3.1.15 from `runs/e2e/antora`,
  `CI=true`; 25 s with the fetches, 299 `.html` files (`public/`).
- **Check**: `target/release/bruce check-dir public --prefix
  https://docs.fedoraproject.org/en-US/ --baseline ../p5.3-baseline/baseline.duckdb
  --config ../../sim/bruce.toml --db ci.duckdb --format json`, the deployment
  guide's config. 9 s and 236 pages; no external host is allowlisted but the
  forge, and no request left the machine bar the one 404 below. `published.json`
  is run 2, `planted.json` run 3, both in `ci.duckdb`.
- **Planted** (`planted.patch`, applied to a copy, `public-planted/`): the
  page `bugzilla-file-a-bug` deleted (16 baseline pages link to it), the
  anchor `the-updates-testing-repository` removed (3 link to it), and a
  new `link:` to the deleted page, a link to a page there is not, an `xref:`
  to a page of a held component that is not there, one out of the build to a
  page that is (`package-maintainers::index.adoc`), and a missing image.

## As published: exit 1, four introduced and one inverse

Nothing else: no `error`, 91 breakage already in the baseline, and 236
introduced redirected links, the home link of every built page (P5.10's
prediction), which never fail.

- **`#project::objectives.adoc`, `#releases::spins.adoc`,
  `#package-maintainers::Packaging_Tutorial_GNU_Hello.adoc` (2 pages)** are
  not breakage: production resolves all four. Antora resolves a page ID
  through a page-alias when no page has it
  (`content-catalog.js`, `resolvePage`); `project/objectives/` is an alias
  to `initiatives/` that the baseline crawled, the live site serves
  `package-maintainers/Packaging_Tutorial_GNU_Hello/` as a meta refresh to
  `Packaging_Tutorial_2_GNU_Hello/`, which the baseline never asked about,
  and `releases` holds both `spins.adoc` and `spins/index.adoc`, one URL
  that the baseline's single page carries the second's facts for. The
  baseline can find none of them by `module` and `rel_path`: **KD8**.
- **The inverse row** is `neurofedora/contributing` linking
  `quick-docs/creating-rpm-packages/index.html`, a baseline target that is
  a meta refresh to `package-maintainers/Packaging_Tutorial/` (quick-docs
  deleted the page in 2022; the alias is the package-maintainers page's).
  The check judges the alias's URL, in the held `quick-docs`, as a page the
  build lacks: **KD9**.

Neither is "real breakage elsewhere", which the spec expected the noise
floor to be made of: both are the gate wrong. The other `ERROR`s in Antora's
log (`releases::lifecycle.adoc` and 60 more) are not introduced rows: they
resolve through the baseline or were broken in it.

## Planted: exit 1 on 12 introduced and 22 inverse pages

`planted.json`: the four rows above, and eight the changes cause:

| Rule | Rows |
|---|---|
| `not_built` | the `link:` to the deleted page, and `fedora-server/server-communicating`'s link to its alias `howto-file-a-bug/` (the deleted page's `page-aliases`), baseline `ok` |
| `broken_anchor` | `quick-docs/` linking the removed anchor |
| `unresolved_xref` | the held-component xref to a missing page, and the two quick-docs pages that `xref:` the deleted one |
| a live 404 | `../nowhere/` (outside any held component, so asked, status 404, no reason) |
| image | `missing.png`, `not_built` |

The valid xref out of the build resolved and is absent. Inverse breakage is
23 links on 22 pages in 7 components: 16 to the deleted page, 3 to its alias,
3 to the removed anchor, and KD9's. The deleted-image case is not planted:
no other component embeds an image of quick-docs's (the baseline has no
cross-component image link into it). `report introduced` and `report inverse`
on run 3 were not re-run; P5.10 asserts their agreement.

## What the next work inherits

- **KD9 is a small fix** (`report::inverse`'s `typed` CASE: a target that
  redirected to a landing in no held prefix is the landing page's owner's);
  it earns **5.11.1**, a tested path reworked on D56's definition of
  removed, so it is not made unattended here.
- **KD8 is 5.12**: the URL a refspec names, by I16, judged by D22's
  chain (history `2026-10-20`, "The review of KD8"). An alias is a
  redirect target the baseline holds only if the crawl reached its URL (15
  meta-refresh targets in en-US out of 6435 pages), and a page's HTML does
  not name its aliases, so the one 5.11 found uncrawled is live's to answer.
- **P6** ([`../design/roadmap-P6-release-inbox.md`](../design/roadmap-P6-release-inbox.md)): a repo's CI job is Antora from npm
  with `CI=true` and the playbook's UI, then `check-dir` with the deployment's
  config; the second takes seconds.

## Negative results

- **The quick-docs on pagure.io is a stub**: its HEAD is "Migrate to
  Forgejo"; the spec's `70cd18e` is the forge's.
- **The errors section is still unanswered**: `errors` is empty in both runs,
  the config allowlisting only the forge and the baseline holding no `error`
  for a link the build makes, so whether it is noise (P5.8's question) waits
  for a repo that links to a host that refuses.
