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

## Finding 3: the unchecked reasons are unexplained

Maintainer, 2026-10-24, on reading 8.4.3: good. The housekeeping's unchecked
links need a legend: a reader may not know what `out_of_scope` means, or which
hosts were allowlisted. On run 1 (`frontier.rs`, `plan`, against the rows):

- **`out_of_scope`**, 157,827 links: on a root's own host and outside every
  root's scope. Here they are docs.fedoraproject.org's other locales, 2,320
  links to 2,307 targets in each, every page's language menu.
- **`host_not_allowlisted`**, 27,405 links to 1,315 hosts, fedoraproject.org
  (5,786 links, to 986 targets), getfedora.org (2,374) and creativecommons.org
  (2,372) the most linked; github.com has the most targets, 1,196.
- **`skipped`**, 1,958: a `skip` glob, here forge's "new issue" links.
- **`scheme`**, 280: not http or https, `mailto:` mostly.
- **The run records no allowlist, `skip` or scope**, only `config_hash`. The
  hosts checked are read off its rows instead: on run 1, forge.fedoraproject.org
  alone. An allowlisted host no page links to is not shown, and does not need to
  be. Recording the config in the run was rejected, being a schema change for
  nothing the reader lacks.

**It spawns 8.4.4**, and amends P8's spec, "What the document holds", item 9.

## Finding 4: some unchecked links are broken

Maintainer, 2026-10-24, on reading 8.4.4's first draft:

- **Its definitions were confusing and technical**; a plain summary that names
  what the links are ("the other languages", "forge's new-issue links") reads
  far more clearly. 8.4.4 is being reworded in that register.
- **Every unchecked link that cannot work is broken, and must be reported as
  broken.** On run 1 they are: `hhttps://forge…` (a mistyped scheme);
  `https:/kubernetes.io` and `https:/docker.io`, which resolve, as a browser
  resolves them, to `docs.fedoraproject.org/kubernetes.io` and `/docker.io`,
  both answering 404 live; and `…/infra/sysadmin_guide/…​..` (an
  ellipsis and a zero-width space pasted into an href). All four are
  `out_of_scope` or `scheme` today, so they were never fetched.
- Of the 11 URLs on docs.fedoraproject.org outside en-US and outside every
  other locale (2,371 links), the rest answer: the slashless home link
  `https://docs.fedoraproject.org/en-US` (2,320 links, every page's header)
  with a 301 to `http://docs.fedoraproject.org/en-US/`, and
  `fedora-project/project/fedora-overview.html` and
  `f28/release-notes/welcome/Welcome.html` with 200 (`curl -sI`, 2026-10-24).
- **bruce has no notion of a locale**: D10's `out_of_scope` is every URL on
  a root's host outside every root's scope, so the four broken links fall
  under the rule meant for the language menu's other locales.

**It spawns 8.10 and 8.11**, P8's spec having gained "A link that cannot
work is broken". The slashless home link is not broken; the housekeeping's
redirects list it once 8.11 asks for it. 8.10 rewrote 8.4.4's fold, which
said "None of them counts as broken" and called a mistyped `https:` "worth
fixing". The review reads it on a crawl made after both.

## Finding 5: the allowlist names the Fedora and Red Hat sites

Maintainer, 2026-10-24: add the Fedora and Red Hat sites to the allowlist,
every subdomain included, and pagure.io, centos.org and the retired
fedorahosted.org with them. The allowlist names hosts exactly, so **it spawns
8.12**, P8's spec gaining "The allowlist names the Fedora and Red Hat sites".
On run 1 those domains hold about 4,000 of the `host_not_allowlisted`
targets, fedoraproject.org (the wiki) 986 of them.

## Finding 6: a trailing slash is pedantic, not broken

Maintainer, 2026-10-24, on reading 8.14's notes: a missing or extra final
`/` is a pedantic error, to be corrected but below a 404 or a missing
fragment. On 8.14's crawl (`runs/p8.14-en-us-20261024-1322/en-us.duckdb`):

- **A slash the server supplies is `ok`, reason `pedantic_trailing_slash`**,
  where D7 now says only `redirected`: a redirected link whose final URL
  differs from the URL linked only by a final `/` added or removed, whatever
  the hops between (I4's slashless docs.fp.o URL included). 496 links to 259
  targets. **The class is named for one form** so that others can be added
  beside it (a scheme upgrade, I5's version redirect, a moved page, the last
  closer to breaking than any of these). It is not a sixth outcome: D16
  refused one for `too_large`, every report and the gate having to place it,
  and a reader reaches the page.
- **A slash the server does not supply stays `broken_url`**, a reader landing
  on a 404 (Finding 4's "a link that cannot work is broken"), but the digest
  names the fix where the URL with its slash toggled answers. No such target
  was found among 8.14's 826 `broken_url` targets, the toggled URL seldom
  being one bruce asked.
- **The 529 `broken_url` links on the three `code-of-conduct//` pages are one
  redirect**: fedoraproject.org's 302 with a doubled slash in its `Location`
  (8.14's notes, "The three double-slash pages"). It is reported once, as a
  redirect landing on a malformed URL, and a page reached only by a redirect
  from off a root is not that root's page, so its links are not counted.

**It spawns 8.16 to 8.18**, P8's spec gaining "A trailing slash is
pedantic". **Open, for the maintainer**: whether the gate (D9) warns, never
failing, on a `pedantic_trailing_slash` link a change introduces. Today no
redirect reaches the gate.

## Where the review stands

Findings 1 to 5 are filed: 8.4.2 to 8.4.4 and 8.10 to 8.12 have landed, and
**8.13 (KD21) lands before 8.14's crawl** (maintainer, 2026-10-24: a host
that does not resolve or never answers must not hide its links as
unchecked). The review **resumes on 8.14's crawl**, no digest of 8.1's
database showing what 8.10 to 8.13 reclassify:

```sh
target/release/bruce report digest --db <8.14's database> \
  --out runs/p8.6-review/digest --name en-us
cd runs/p8.6-review/digest && setsid python3 -m http.server 8086 --bind 0.0.0.0
```

**Still open for the maintainer**: Finding 1's question, whether "Repos"
should also count broken links by their target's repo; Finding 6's, whether
the gate warns on a `pedantic_trailing_slash` link. 8.16 lands before the
review resumes.
