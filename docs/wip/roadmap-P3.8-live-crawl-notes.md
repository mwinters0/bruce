# P3.8 The live crawl: notes

What P4 inherits from the first contact with docs.fp.o, and what was refused.

**Two runs of `/en-US/fedora-docs/`**, 2026-10-18, at commit `010f25a`, the
default UA, allowlisting `forge.fedoraproject.org` and `gitlab.com`. Configs,
logs and DBs are `runs/p3.8-live-20261018/`: `bruce.toml` (`crawl.log`,
`live.duckdb`) discovers its sitemap; `bruce-sitemap.toml`
(`crawl-sitemap.log`, `live-sitemap.duckdb`) sets it to I1's
`/en-US/sitemap.xml`. `sm24.txt` is the 24 component pages of the grilling's
`all-locs.txt`.

| | discovered | set |
|---|---|---|
| Summary | 1827 targets, 84 pages, no sitemap | 1967 targets, 87 pages, 69 sitemap files, 2283 entries dropped |
| Wall clock | 1:44 | 2:54 |
| Peak RSS | 54 MiB | 55 MiB |
| `ok` / `broken_url` / `error` | 134 / 3 / 0 | 207 / 3 / 0 |
| `unchecked` | 1650 `out_of_scope`, 39 `host_not_allowlisted`, 1 `robots` | 1717, 39, 1 |

**It passes the row**:

- **Every sitemap page is fetched** with the sitemap set: 24 of 24, each
  `in_sitemap`. 2307 `<loc>`s less 24 is the 2283 dropped.
- **The alias resolves as a hop**: `/en-US/fedora-docs/` is a 200 whose
  chain is one refresh hop, `final_url` `team-info/` (I14).
- **No `error`.** The three `broken_url`s are each a 404 to `curl`:
  `fedora-docs/content/` and `fedora-docs/content/style-guide/`, hand-written
  links missing the `contributing/` module, and
  `forge.fedoraproject.org/docs/tickets/projects/407`.
- **Outcomes are plausible.** gitlab.com's `/groups/*/-/saml/` is its own
  robots.txt Disallow. Each forge "report an issue" link
  (`issues/new?title=…`) is a 303 to `/user/login`, `ok`: the link works for
  a reader, who is asked to sign in. Of the 169 fragment links, 8 name an id
  absent from a fetched target, each absent to `curl` too: 7 `prerequisites`
  or `_prerequisites`, and one I6 unresolved xref
  (`#./index.adoc#Prerequisites`, its `xref_refspec` set). P4's join has real
  breakage to find in the first component tried.
- **Politeness held**: completions are a median 1.0 s apart on docs.fp.o
  and on the forge, whose robots.txt also asks `Crawl-delay: 1`. gitlab.com's
  robots.txt has its `Crawl-delay` commented out, so it gets the default 2/s.

**What the next phase inherits:**

- **A component root does not discover docs.fp.o's sitemap**: neither
  `/en-US/fedora-docs/sitemap.xml` nor `/sitemap.xml` exists (I1), so the
  discovered run is link-only and misses the one page nothing links to,
  `contributing/content/contrib-existing-documentation/` — an orphan, the
  page D14's report exists for. A root at `/en-US/` finds the index by its
  probe; any narrower root must set `sitemap`.
- **`pages.via_links` is false for every listed page**, 23 of the 24 being
  linked too (run one reached them by links): the claimant's flag, as P3.6
  left it (P4 inbox, "What a crawl leaves for the join to place"). An orphan
  report must join `links`.
- **The forge costs about four requests a docs.fp.o page** when allowlisted
  (P4 inbox, "Allowlisting the forge sets a full crawl's length").

**Negative results.** None of KD3, KD4 or the stop's paths showed: no fragment
link here names an allowlisted URL an image named first, and no run was
stopped.
