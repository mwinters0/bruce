# P8.10 — A scheme that cannot work is broken: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"A link that cannot work is broken", its scheme bullet) and 8.6's Finding 4
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 4"): `crawl::frontier`'s `WORKING_SCHEMES`, `SCHEME`, `BAD_SCHEME`
and `Plan::Broken`, the frontier and crawl tests, `crawl`'s dispatch and
sitemap-index rows writing it, D66 with D13 and D16 amended, the digest's
unchecked fold reworded, the manual (`digest.md`, "Why a link is unchecked";
`report.md`'s intro and `reason` column; `check-dir.md`, "How a link is
answered"), and a `file:` link, a `mailto:` link and a planted `hhttps:` link
in the e2e site, with its golden digest and saved pages written anew.

## The reason is `bad_scheme`

A link with an unlisted scheme is `broken_url`, reason **`bad_scheme`**, not
`scheme`. The reason is shown raw in the digest's Problem column ("URL broken
· bad_scheme") and in `report broken`, so it has to say why the URL is broken
on its own, as its sibling `bad_redirect` does for a 3xx that no browser can
follow. Using `scheme` under both outcomes would make one name mean two
verdicts, and `reason = 'scheme'` would match `unchecked` rows in a run before
this slice and `broken_url` rows after it. With its own name, a diff of two
runs shows the link moving from (`unchecked`, `scheme`) to (`broken_url`,
`bad_scheme`), which is what 8.8 has to tell apart from new breakage.

## What the next slices inherit

- **On en-US's run 1 this reclassifies two links**
  (`duckdb -readonly runs/p8.1-en-us-20261023-2004/en-us.duckdb`, targets
  with reason `scheme` joined to their links): the `hhttps://forge…` link
  8.6 named, on `infra/release_guide/`, and `file://path/to/config.ign` on
  `fedora-coreos/provisioning-aws/`, a placeholder path that is an `href`
  all the same. The other 278 `scheme` links
  are `mailto:` (275), `irc:` (2) and `appstream:` (1), all listed schemes.
  No `javascript:` or `data:` link is on en-US. 8.8's second crawl shows the
  two as `broken_url`, and its notes count them as reclassified, not new.
- **`check-dir` plans a link as the crawl does**: the scheme test comes
  before the build and the baseline in `Frontier::plan`, so a build's
  `file:` link is broken whatever the baseline says. The e2e gate proves it
  both ways: usage's `file:` link is in the unchanged build's
  `already_broken` (now 10) and the planted build's (now 7), and the planted
  `hhttps:` link is introduced, `new`.
- **A baseline crawled before this rule** holds such a link as `unchecked`,
  so a `check-dir` built after it would call the link introduced. P9 ships
  the binary beside the baseline it crawled (P9's spec, "CI fetches the
  binary from beside the baseline"), so a content repo's CI never meets the
  mismatch; a deployer pinning a release against a newer baseline, or the
  reverse, can.
- **A redirect to an unlisted scheme is unchanged**: `hops::location` takes
  only an `http(s)` `Location`, so such a 3xx is `broken_url`,
  `bad_redirect`, as before. `Plan::Broken` never reaches `hop_plan` in
  practice, and is asked for its status there as `Plan::Unchecked` is.
- **A sitemap index listing an unlisted scheme** writes the entry
  `broken_url`, `bad_scheme`; a page sitemap's such entry is dropped, as any
  entry outside every root's scope is (D10).

## Negative results

- **`data:` and `javascript:` are working schemes, added by 8.12** (D66).
  The maintainer's first list left them out; both do something in a
  browser: a `data:` image renders (Asciidoctor writes them under its
  `data-uri` attribute), and `javascript:void(0)` is a common no-op link.
  Neither is on en-US, so nothing reclassified depends on it.
- **Rejected: a configurable list.** Not asked for, and like the locale
  globs P8's spec rejected, it would be one more list to keep. A scheme found
  missing is added to `WORKING_SCHEMES`.
