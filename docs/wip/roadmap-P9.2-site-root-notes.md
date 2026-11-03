# P9.2 — The site-root refresh and the gate's key: notes

What 9.3 to 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The gate matches a link
with its final `/` toggled" and "The site-root refresh rule is `ci`'s". The
decisions are D101 and D22's amendment.

## What landed

- **The gate's key** (`report::gate`): a CI link matches a baseline link from the
  same page and fragment whose target, with its path's final `/` dropped, is the
  same. The base CTE carries that dropped form as `tkey`, computed in SQL by one
  `regexp_replace` (a root's own `/` and the query are left alone), so the
  correlated `EXISTS` stays a hash join on equal columns. Every section reads
  the one `same_key()`; the `baseline` state column too, which is why a `/foo`
  broken where `/foo/` answered still reads `ok` and is introduced.
- **The refresh** (`ci`): `Build::scan` reads the top `index.html` and asks
  `extract::leaves_at_once` (a `<meta http-equiv=refresh>` of delay 0 naming a
  URL, anywhere in the page); if it is one the file is a seed and a URL at its
  own path alone, and the prefix is none. A delayed refresh, a reload and a
  page with no meta stay the prefix.
- **`scripts/e2e.py`**: `UNCHANGED_REDIRECTED` is empty, `PLANTED_REDIRECTED`
  the alias's row alone; `mise run e2e` passes (`runs/9.2-e2e.log`).

## What the next slices inherit

- **Production's home link still passes through plain http** (`.../en-US` 301s
  to `http://.../en-US/`), on every page: a matter for 9.8's review, not the key.
- **The key's dedup is unchanged**: two CI links on one page whose targets differ
  only by the slash are two rows, as before. Only the match against the
  baseline toggles.
- **`report diff` and the digest's cause attribution key on the exact target**
  (`report::diff`, `report::digest::cause`): D101 is the gate's. A digest
  compared across a change of a site's slash convention lists the links as
  new and fixed.

## Negative results

- The earlier attempt to remove the three `UNCHANGED_REDIRECTED` rows by the
  refresh alone failed (the 2026-11-02 history entry): the rows are the key's.
  The refresh stays for what the baseline answers at the prefix: its landing,
  hops and fragment are production's, where the build's would be the PR's.
- Reading the whole page for the refresh, not to `<body>`, since browsers honour
  a `meta` in the body; the cost is one small file's stream.
