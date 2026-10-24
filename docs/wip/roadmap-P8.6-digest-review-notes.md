# P8.6 — The maintainer's review of a real en-US digest: notes

The review is in progress and is expected to take several rounds, so this doc
holds the findings so far, not a verdict. The digest under review is run 1 of
8.1's crawl, written at `c02085a` from a copy of its database:
`runs/p8.6-review/digest/en-us-2026-10-23.{html,json}`, served by
`python3 -m http.server 8086` from that directory (its PID in
`runs/p8.6-review/server.pid`). It is rewritten in place after each round of
fixes, so the URL stays the same.

## Finding 1: a count does not say which way it points

Maintainer, 2026-10-24: the digest is confusing about whether a number counts
links *to* the URL shown or links *on* it, that is, whether the URL is the
source of a problem or its target. The digest has both kinds of section and
labels neither:

- **Inbound, per target**: "Fix once, fix many" (a row is a broken target,
  its Links and Pages the links naming it and the pages holding them, "Fix in"
  the target's source) and the housekeeping's redirects (Landing a target,
  On pages its sources).
- **Outbound, per source**: "Pages by broken links" (a page's broken links,
  its "644 links" being every link on it, which nothing says) and "By repo"
  (the broken links held on the repo's pages).
- **The two blame different repos for one link.** The 7 links to
  `fesco/Updates_Policy/#updates-testing-activation` are fixed in fesco's
  file by "Fix once", while "By repo" counts them against maintainer-docs (3),
  operations/docs (2), quick-docs (1) and fesco (1). Of the run's 828 broken
  links, 674 name a target in their own page's repo, 21 one in another, and
  133 one with no source (`broken_targets` against `pages` in the JSON).
- **The figures count links under target nouns.** "218 Broken URLs" and
  "610 Broken anchors" sum to the 828 broken *links* "Fix once" cites.
- **The drill-down's columns read as two destinations.** "Link" is the href as
  written and "Target" where it resolves, and the page holding them is only in
  the `<summary>` above. **"anchor" still means two things there**: Kind
  `anchor` is the `<a>` element, Problem `anchor missing` a fragment absent
  from the target, so `firstdonoharm.dev/… | anchor | broken | 404` reads as a
  fragment failure. 8.4.1 renamed the summaries' count to `fragments` for the
  same clash and left the Problem column.

**It spawns 8.4.2**: the HTML's and the manual's wording, every count and
heading saying which way it points. The JSON's field names stay, a rename
being a change a loader would misread (D64's `format_version`).

**Open, for the maintainer**: whether "By repo" should also count broken
links by the repo of their target, the repo "Fix in" names, beside the repo
of their page. That changes what the section delivers, so 8.4.2 does not
decide it.

## Finding 2: the sections' order and folding

Maintainer, 2026-10-24, on reading 8.4.2's wording: better. Every section
should be collapsible as the housekeeping is, and start collapsed, and the
order should be the pages, the repos, the targets, then the housekeeping.
**It spawns 8.4.3**, and amends P8's spec, "What the document holds", to that
order. The spec's numbers stay the sections' names, being cited by number.
Placed by this session, the maintainer not having named them: the header and
the figures stay open and first; "Since the last run" (3, 8.8) stays after the
figures; "Anchors removed" (7, 8.7) follows the targets; the errors precede the
housekeeping.
