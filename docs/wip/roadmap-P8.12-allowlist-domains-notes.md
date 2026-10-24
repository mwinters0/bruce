# P8.12 — An allowlist entry `*.<domain>` names its hosts: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"The allowlist names the Fedora and Red Hat sites", and the scheme bullet of
"A link that cannot work is broken") and 8.6's Finding 5
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 5"): `config::Allowlist`, `Config::is_allowlisted` and
`Config::check_allowlisted_roots`, called by `cli::crawl::configure` beside
the other checks a root from the command line needs; `crawl::frontier`'s plan
and hop plan matching through it; `data` and `javascript` in
`WORKING_SCHEMES`; D68, D5 and D66 amended; KD21; the manual (`config.md`,
"Allowlist" and "Hosts"; `deploy.md`'s allowlist and rate prose;
`digest.md`'s `scheme` row). The e2e site is unchanged: its one host is
`127.0.0.1`, which no `*.` entry can name, and `crawl::tests` serves the
case over `*.app.localhost` instead, every name under `localhost` being the
loopback to hickory.

## Precedence

A root's own host named outright in `allowlist` is **refused at load**, as
D5's "a root's own host belonging there and in no allowlist" already said
and nothing enforced. Since 8.11 such an entry's only effect was to ask
another locale's pages one hop deep, the check D10 rejects; testing the
locale rule first instead would leave the entry loaded and doing nothing.
The refusal runs where a root is known: in `Config::parse` when the file
names one, and in `configure` after the command line's URL is added, as the
`robots = false` and `skip` checks do. With that, `Frontier::plan`'s order
(allowlist before the locale rule) is moot for a root's host: no entry
matches one. A file that loaded before and named its root's host now fails
with the message; none in the repo did.

`[host."*.x"]` is refused too (`a host name holds no *`): `Host::parse`
takes a `*`, so before this slice both `[host]` and `allowlist` loaded such a
name as a literal host that no link could ever name.

## Cost

`is_allowlisted` is one hash lookup for the host outright, then one per `.`
in it against the set of `*.` domains, slicing the host with no allocation:
under the guide's list, four lookups for `a.b.fedoraproject.org` whatever the
list's length. The root-host exclusion runs only when a domain matched, and
`is_root_host` now compares `Url::host_str` rather than allocating each
root's host. No measurement: the plan already allocates the host string per
link, which costs more than the lookups.

## The crawl P8.8 runs

From 8.1's database (`runs/p8.6-review/en-us.duckdb`, run 1, targets with
reason `host_not_allowlisted` whose host is one of the nine domains or below
one): **3,521 targets on 161 hosts**, named by 14,685 links on 2,320 pages;
3,499 first named by an anchor link (a GET), 22 by an image (a HEAD). 8.6's
"about 4,000" was an estimate.

| Domain | Targets | Hosts |
|---|---:|---:|
| fedoraproject.org | 2,541 | 108 |
| redhat.com | 416 | 16 |
| pagure.io | 357 | 1 |
| fedorapeople.org | 66 | 12 |
| fedorahosted.org | 46 | 3 |
| fedoramagazine.org | 42 | 1 |
| fedorainfracloud.org | 20 | 7 |
| centos.org | 18 | 12 |
| getfedora.org | 15 | 1 |

Each host's robots.txt read 2026-10-24 under the guide's `user_agent`, its
rules applied to each target (the group naming `bruce-fedora-docs`, else
`*`; longest match; a `Crawl-delay` above every group applying, as
`texting_robots` reads it), at the host's `Crawl-delay`, else the default
2 a second. The hosts run in parallel, so the longest is the bound:

| Host | Targets | robots.txt | Asked | Minutes |
|---|---:|---|---:|---:|
| lists.fedoraproject.org | 175 | `Crawl-delay: 10` | 175 | 29.2 |
| src.fedoraproject.org | 245 | `Crawl-delay: 5` | 245 | 20.4 |
| bugzilla.redhat.com | 336 | `Crawl-delay: 5`, only bug pages allowed | 224 | 18.7 |
| fedoraproject.org (the wiki) | 986 | none | 982 | 8.2 |
| communityblog.fedoraproject.org | 47 | `Crawl-delay: 10` | 47 | 7.8 |
| fedoramagazine.org | 42 | `Crawl-delay: 10` | 42 | 7.0 |
| meetbot.fedoraproject.org | 391 | `Crawl-delay: 1` | 391 | 6.5 |
| access.redhat.com | 24 | `Crawl-delay: 10` | 24 | 4.0 |
| www.redhat.com | 21 | `Crawl-delay: 10` | 21 | 3.5 |
| pagure.io | 357 | 403, so none | 357 | 3.0 |
| the other 151 | 897 | | 710 | under 1 each |

**3,218 of the 3,521 are asked.** The rest: 274 forbidden by robots.txt
(Bugzilla's 112, all but its bug pages; fedorapeople.org's and its people's
hosts', dl.fedoraproject.org's and koji's, which disallow everything), and 29 on 16 hosts whose robots.txt had no
answer, internal names such as `app1.fedora.rdu3.redhat.com` and retired ones
such as `teams.fedoraproject.org` (KD21). **Added wall time: none expected.**
8.1's crawl ran 70 minutes (`runs.started_at` to `finished_at`), and the
longest host, 29 minutes, runs beside it under D61's schedule; only a host
whose links are all met in the crawl's last half hour would extend it, by up
to its own time. Redirect hops are not counted, each another request at its
landing host's pace.

## What the next slices inherit

- **8.8's second crawl shows about 3,200 targets moving** from `unchecked`,
  `host_not_allowlisted` to an asked outcome, and 274 to `unchecked`,
  `robots`, 29 to `robots_unavailable`. Its notes count them as
  reclassified, not new.
- **pagure.io is a static read-only archive** (its pages say so; `/login`
  is a 404), and every `new_issue` link, 18 targets, answers 404 whether or
  not the project exists. They are reported broken, which is what they are
  to a reader, so the guide skips none of them.
- **src.fedoraproject.org's `new_issue`** redirects to its login page, which
  answers 200: one target, reported ok while telling nothing. Not skipped,
  being one link; a `skip` glob like the forge's is the remedy if more
  appear.
- **KD21, found here**: a host whose name does not resolve, or that never
  answers, closes at its robots.txt, so its links are `robots_unavailable`
  and never D16's `broken_url` for NXDOMAIN. P8's spec says a staging or
  internal host "then shows as broken or an error"; that holds for a host
  that answers, and the guide's prose says which is which.
- **`config.rs` is at 37 functions** of the soft limit's 40.

## Negative results

- **No `[host]` or `skip` entry is added to the guide.** Every new host's
  `Crawl-delay` is obeyed without one (D18), and the defaults are polite
  where there is none; robots.txt already keeps Bugzilla's `enter_bug.cgi`,
  23 targets, unasked.
- **A host with a trailing dot** (`fedoraproject.org.`) is not below
  `*.fedoraproject.org`, as it is not equal to a bare entry either; no link
  on en-US has one.
- **Rejected: glob matching over hosts** (globset over the host string),
  whose cost per link grows with the list; the suffix lookup does not.
