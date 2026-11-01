# P10.8 — The closing evidence: notes

Read on 2026-11-01. The "after" crawl is run 2 of `runs/en-us-20261101-0005/en-us.duckdb`, `b0fc6ac` under the
guide's config with the `fedora` extension, into a copy of 10.1's database, whose run 1 is the "before" (`d88ee2b`).
It exited 0 after 1 h 16 min. `compare.sql` beside the database is the comparison, run with `duckdb -readonly`; the
digest is `digest/en-us-2026-11-01.{html,json}` there.

## What the comparison found

**Nothing P10 changed differs.** Both runs hold 173,502 targets and 69 sitemap files; run 2 holds 9,001 pages to
run 1's 9,002 (2,329 `fedora` for 2,329 `antora`, 6,672 `generic` for 6,673).

- **Profile and policy**: `pages.profile` reads `fedora` on exactly the 2,329 pages that read `antora`, and the only
  `policy` rows run 2 adds are `profile.fedora.extends`, `.generator` and `.locale_links` (the digest's
  `policy_changes`, each `judges: false`); none is removed. The `antora.*` facts are the same 2,326, 2,158, 2,158
  and 2,320 rows in both runs, and **no fact row of any page, the 890 changed ones included, is in one run only**.
  No page differs in title, content type, hash scope, source, placement or sitemap membership.
- **Locales**: the targets under docs.fp.o outside `/en-US/` are the same 154,770 `out_of_scope`, 6 `ok` and 5
  `broken_url`, so `locale_links` learns what the compiled profile did.
- **Verdicts**: 3 of 590,047 link verdict rows differ, each with a cause outside bruce.
  - `releases/branched/` to `http://download.fedoraproject.org/pub/fedora/linux/development/`: `unchecked`/`robots`
    became `broken_url` 404 (the digest's one `new_breakage`). The host redirects to a mirror picked per request:
    `insect.mm.fcix.net` in run 1, whose robots.txt disallowed the path, `mirror.cs.odu.edu` in run 2 (`final_url`).
    The page's content hash is unchanged, so this is the one case the handoff's "a verdict moving on an unchanged
    page" rule meets; it moves with the target, not the page.
  - `infra/sysadmin_sops/copr/` to `http://copr-be-dev.cloud.fedoraproject.org/`: 200 became a `timeout` error, so
    that host's page, in run 1 only, is the 9,002nd. This also moves `report stats` by one broken URL and one error.
  - `quick-docs/rpmfusion-setup/` links `docs.fedoraproject.org/en-US/legal/` where it linked a wiki page: the page
    changed, and it is the only en-US page `report changed` lists (the other 890 changed hashes are `generic` pages
    outside the root). The 140 pages whose anchor sets differ all have a changed hash.

## Negative results

- **Counts alone mislead**: `report stats` moved 829 broken URLs to 830, 87 errors to 88 and 171,111 unchecked to
  171,110, each explained by the first two bullets above and by nothing else.
- **A SAML sign-in URL on `sso.redhat.com` never matches across runs**: its signature is fresh per fetch, so two
  such targets are in each run and none in both. `compare.sql` leaves them out.
- The digest was read as JSON; its HTML was not opened.
