# P8.4.4 — The digest's unchecked links explained: notes

What landed, for 8.6's Finding 3
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 3"): two queries in `report/digest/sections.rs`
(`unchecked_groups`, `checked_hosts`, over one head, `link_urls`), two
`Digest` fields, the JSON's `housekeeping.unchecked_groups` and
`housekeeping.checked_hosts` under the same `format_version`, the
housekeeping's "Unchecked links" fold rewritten in `report/digest/html.rs`
(`unchecked`, `most`, `unchecked_meaning`), two rules of `html/style.css`,
the tests, the golden file (`cd scripts && uv run e2e.py --save-digest`), the
manual's "The HTML", JSON table and "Why a link is unchecked", and D64's
`Rejected` line, which now records the run not keeping its configuration.

## What the fold says

- **Each reason is a heading** with its links and targets, a paragraph
  saying in a reader's words what its links are (`out_of_scope`: "Links to
  pages on a site this run checks, but outside the part of it the run was
  set up to check, such as the other languages a language menu links to when
  only one language is crawled."), with at most one more sentence naming the
  setting that would check them (`[[root]]`, `allowlist`, a host's
  `robots`), and a table of the five groups its links point to most, then
  "And N more, holding M links, none more than K." On en-US the last clause
  shows that none of the 72 unlisted groups holds more than 2,320 links,
  without listing them.
- **The groups**: a non-`http(s)` URL's scheme (`mailto:`); for `skipped`
  the host and the path's last two segments, elided between
  (`forge.fedoraproject.org/…/issues/new`), the part a forge's `skip`
  glob matches; on a root's host the URL up to the path's first segment,
  with the `/` when the path goes on; elsewhere the host. **On a root's
  host the scheme and the slash are kept** because a root's scope is a URL
  prefix: `http://docs.fedoraproject.org/en-US/` (16 links) and
  `https://docs.fedoraproject.org/en-US` (2,320, the home link on every
  page) are both out of scope, and as `docs.fedoraproject.org/en-US` they
  would read as the root itself. Ranked by links, then targets.
- **"External hosts checked"**: the hosts no root starts on whose targets
  were asked about (target outcome not `unchecked`), with links and targets,
  ten listed. In a crawl that is the allowlist, plus any host a root's
  `include` reaches, which the wording ("the hosts no root starts on")
  covers.
- **Every reason the crawler writes has a meaning**, listed in
  `every_reason_the_crawler_writes_for_an_unchecked_link_is_explained`: the
  frontier's `scheme`, `skipped`, `out_of_scope`, `host_not_allowlisted` and
  `externals_skipped`; the walk's `robots`, `robots_unavailable` and
  `too_deep`; `link_verdicts`' `cut`, `text_fragment` and `no_anchors`. The
  list is kept by hand. A reason added later and not described renders as
  "This version of bruce has no description for this reason." No test fails
  for it.
- **The wording is general**, compiled in and true of any site and run: an
  example names a kind (a language menu, a forge's "new issue" form), never
  an en-US figure. Precision (globs, the redirect into a `skip`, robots.txt's
  failure modes, the sitemap depth of three) is the manual's "Why a link is
  unchecked", which the HTML must not contradict.

## On en-US (run 1, `runs/p8.6-review/`)

The fold reads `out_of_scope` 157,827 links in 77 groups: 67 locales and
the no-slash home link at 2,320 each, then `https://docs.fedoraproject.org/`
(26), `/fedora-project/` (18), `http://…/en-US/` (16). `host_not_allowlisted`
has 27,405 links in 1,315 hosts, led by fedoraproject.org (5,786),
getfedora.org (2,374), creativecommons.org (2,372), www.redhat.com (2,343)
and use.fontawesome.com (2,320, one stylesheet on every page). `skipped`
has 1,958 links, 1,951 of them `…/issues/new`, and `scheme` has 280, 275 of
them `mailto:`. The one checked host is forge.fedoraproject.org (4,469
links, 4,151 targets). **Finding 3's per-host figures were targets, not
links**. It is corrected in place. The digest takes 2.0 s, and the JSON
grows from 682 KB to 805 KB, 1,403 of its rows being groups.

**Not all unchecked links are harmless**: `hhttps:` (1), and
`https://docs.fedoraproject.org/kubernetes.io` and `/docker.io`, which are
scheme-less hrefs that a browser resolves against the page. The fold showed
them and no section counted them as broken; 8.10 makes the first broken
(D66), and 8.11 is to ask for the other two.

## What 8.8 and P9 inherit

- **8.8's comparison** can diff `unchecked_groups` by (`reason`, `group`)
  without new SQL.
- **An every-locale run** (P9) covers the locales its roots do, so the
  language menus' links stop being `out_of_scope`. The JSON's groups are then
  mostly the external hosts, about 1,300 rows on en-US, listed whole like
  every other table.

## Negative results

- **Rejected: the first draft's definitions**, which said how bruce decides
  ("outside every root's scope", "a `skip` glob", "answers the same whether
  or not…"); the maintainer read them on en-US as confusing and technical.

- **Rejected: the reasons' meanings in the JSON.** They are prose for a
  reader. A loader keys on `reason`, and the manual and the HTML say what
  each means.
- **Rejected: grouping `skipped` by host and first segment**, which on en-US
  gives forge's organisations (`docs` 754, `infra` 268) and does not say
  what the glob matched. The last two segments do.
- **Rejected: capping the JSON's groups** at the HTML's five. It would need a
  remainder row of another shape, and it drops the per-host series a loader
  might graph.
- **Servo 0.7.0 drops the underscores** of `host_not_allowlisted` in the
  `<h3>`'s `<code>` (`mise run shot --height 6000`, print copy). Chrome's
  headless shell 156 shows them at 794 and 390 px, closed and opened, and
  the page is unchanged for it.
