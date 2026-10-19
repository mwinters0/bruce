# P4.3.1 The guide's example: robots.txt and the skip glob: notes

`docs/manual/deploy.md` takes both fixes P4.4 earned. docs.fp.o gets
`[host."docs.fedoraproject.org"]` with `robots = false` and `per_second = 1`,
and the guide names the alternative for a deployer who controls the site, a
`robots.txt` group for its own agent holding only `Crawl-delay: 1`. The skip
list is `…/**/issues/new*`, `…/**/issues/new*/**` and `…/user/login*`, and
`config.md`'s example and Skip section take the same pair. D18 is rewritten.

## What the next slice inherits

- **Two config tests read the guide** (`config::tests`): the skip globs match
  seven forge shapes (a bare `issues/new`, a query with and without a `/`,
  `/choose`, `/user/login`) and not an issue, a file under `src/branch/`
  or a docs.fp.o page; and `robots` is off for docs.fp.o alone. Both read the
  guide's blocks as `mise run sim` joins them, so the guide cannot drift from
  them. The 1921 live URLs were checked in P4.4, not again here.
- **P4.10 compares like with like only if its config is this one.** The
  next full crawl should show no `robots` targets, about 1925 `skipped` still
  (the forge's 16 + 1905 + 4 login-related), and no forge redirect asked for
  `issues/new`. The estimate is about 47 minutes for docs.fp.o and 69 for the
  forge if the hosts still run in turn (P4.4's notes), about two hours; the
  guide says so.
- **The sitemap page `fesco/Fedora_Council_Engineering_Rep/` and the nine
  `Fedora*` 404s now surface**: the nine as `broken_url`, the page as `ok`.

## Negative results

- None measured; no crawl ran. The guide's two-hour figure is P4.4's
  arithmetic, not a new run.
