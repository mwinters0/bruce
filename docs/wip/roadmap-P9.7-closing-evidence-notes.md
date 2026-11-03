# P9.7 — The closing evidence: notes

What 9.8 to 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The slice order", item 7, and
"The dev deployment is a GitHub repo of its own". The slice is **not delivered**: its
box stays empty.

## What landed

- **The mirror**, `mwinters0/quick-docs-bruce-mirror` (public, created 2026-11-03), is
  `docs/quick-docs`'s `main` at `b86cb96` ("Prepare for F45 release") plus one commit
  adding the guide's `links.yml`, taken from `docs/manual/deploy.md` with `PUB_URL` set
  to `https://mwinters0.github.io/bruce-fedora-docs-ci`. Its two pull requests are
  `clean-change` (#1, one sentence of `accessibility-installation-with-orca.adoc`
  reworded) and `planted-break` (#2, the page's xref to
  `creating-and-using-a-live-installation-image.adoc` given an anchor that page lacks).
  The repository is deleted at P9's wrap.
- **The workflow ran on GitHub Actions end to end** in 58 to 70 seconds a job: the
  tools, the fetch of the change by SHA, Antora 3.1.15, the build, C3's fetch (the
  set of the 2026-11-03 first night: the binary, config and baseline named by content,
  `sha256sum -c` passing) and `bruce check-dir`, run 2 of 1488 targets and 239 pages
  against the baseline of run 1. No step failed but the check.
- **The planted break fails the check, exit 1**: the row `accessibility-installation-with-orca/`,
  `../creating-and-using-a-live-installation-image/#proc_planted-missing-anchor`, in
  "introduced breakage" (run `41717735992`).

## The clean change does not pass

Both runs of #1 (`41717454068`, `41717482807`) fail, on one introduced row that no
line of the change touches, and #2's introduced breakage is that row plus the planted
one. It is `quick-docs/virtualization-getting-started/`'s
`xref:f{MAJOROSVER}@fedora:release-notes:welcome/Hardware_Overview.adoc[Release Notes]`,
`MAJOROSVER` being 45 on `main` and the release not yet built by production.

- **Production left the xref unresolved**: its page carries `href="#f45@fedora:release-notes:welcome/Hardware_Overview.adoc"
  class="xref unresolved"` (`curl` on docs.fp.o), and the baseline's row is a link to
  its own page with that fragment, `broken_url`, reason `unresolved_xref`
  (`link_verdicts` of the published baseline, `raw_href` like `%Hardware_Overview%`).
- **The CI run points it at the URL its refspec names** (`crawl::xref::resolve`, D57),
  `.../fedora/f45/release-notes/welcome/Hardware_Overview/`, which answers 404; the link is
  `broken_url`, reason `unresolved_xref` there too (`store::views`, rule 1). Both rows are
  broken for the same reason, and `report::gate`'s key, the page, the target and the
  fragment (D101), differs, so the CI row is introduced. The baseline's other
  already broken links (156 in all) are not introduced.
- **The link is broken in production, not pre-release.** The baseline holds `fedora` f45
  (5 ROOT and 7 `release-notes` pages in `runs/en-us-20261103-0104/en-us.duckdb`, run 2),
  which is what let D57 name a URL at all; f45's and f44's release notes are flat
  (`hardware_overview.adoc`), and that crawl has the xref's f44 form `broken_url`,
  `unresolved_xref` too. It stays broken, and every quick-docs PR red, until quick-docs
  fixes the path, f45 shipping or not.
- **It is KD40**: any cross-component xref production leaves unresolved, into a
  component-version the baseline holds, whose named URL is broken. Quick-docs carries 7
  unresolved xrefs naming another component, the site 95 unresolved xrefs in all. 9.13
  fixes it.

## What 9.8 to 9.10 inherit

- A clean pass on the mirror is the evidence still owed, 9.13's to take: a push to the
  mirror's `clean-change` at `b86cb96`, the broken xref kept, it being the case the pass
  has to show.
- The three "introduced redirected links" (`package-maintainers::Packaging_Tutorial_GNU_Hello.adoc`
  from two pages, `project::objectives.adoc` from one) are listed and do not fail.
- The Forgejo runner stays unexercised (P17).

## Negative results

- `gh repo create --push` failed on a shallow clone ("did not receive expected object"):
  the mirror was pushed after `git fetch --unshallow`.
- The runs began some three minutes after the pull requests were opened, on a reopen
  and a push. Whether GitHub's delay for a new repository or the reopen started them is
  unmeasured.
