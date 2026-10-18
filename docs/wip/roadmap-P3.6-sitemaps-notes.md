# P3.6 Sitemaps: notes

What P3.7 onward inherits, and what was refused.

- **`Summary` gained `sitemaps` and `dropped`**: the files read as sitemaps,
  and the entries outside every root's scope. P3.7 prints both; a run with
  `sitemaps == 0` had none, and says so. In the DB the same fact is a run with
  no `links` row of kind `sitemap` (STATUS, "Decisions worth another look").
- **A run has two passes** (D47): `Shared::discover` claims each root's
  sitemaps, `drive` reads them and their children with every page they list
  held back, and then the held pages and the roots' seeds are crawled
  together. P3.7's Ctrl-C stops new work in `drive`'s spawn loop, which both
  passes share.
- **Discovery** is D40's order per root: `sitemap` set, else robots.txt's
  `Sitemap:` lines (all of them, resolved against the root's URL, `http` and
  `https` only), else `sitemap.xml` under the root and then at the host, the
  first that answers with a sitemap. A probe already claimed by another root
  counts as found. `Fetcher::sitemaps` is on the trait, empty by default;
  `HttpFetcher`'s loads the host's robots.txt if no one has yet.
- **`Mode::Sitemap` and `Body::Raw`**: a GET whose 2xx body is read as
  bytes whatever its `Content-Type`, up to `Settings::max_sitemap`
  (`fetch::MAX_SITEMAP`, 50 MiB, the protocol's limit). `crawl::sitemap::read`
  undoes a gzip of its own, told by the magic bytes, capped at the same 50 MiB.
  Over either cap the target is `ok`, `too_large`, and no entry is read.
- **A sitemap keeps its plan through a redirect** (`Shared::hop`), so a moved
  sitemap is read where it lands, D46's landing row included. A redirect from a
  sitemap entry is an ordinary page's.
- **Calls beyond the spec**, none touching a contract:
  - **An index's sitemaps are followed wherever they are**, scope applying to
    pages only; a host-level index of locale sitemaps costs their requests
    and drops their entries.
  - **Depth counts files**: the discovered one is 1, and an index at 3 does not
    follow its children. Each is a target `unchecked`, reason `too_deep`, with
    its `sitemap` link.
  - **A `<loc>` that is not an absolute URL** is a `broken_url` target, reason
    `invalid_url`, like an href (D26), not dropped: it is not known to be out
    of scope.
  - A 2xx file that is not XML, gzip, a `<urlset>` or a `<sitemapindex>` is
    `error`, `unreadable`, as a page the extractor refuses is.
  - An entry's fragment, which no sitemap should carry, goes to
    `links.fragment` as a page link's does.
- **`in_sitemap` and `via_links`** are the claimant's (P3.5): a listed page is
  `in_sitemap` true, `via_links` false even when also linked, a root's seed
  listed in a sitemap included. Both stay conveniences; the joins are the
  authority (P4 inbox).
- **Negative results.** `quick-xml` 0.42 reports `&amp;` as a `GeneralRef`
  event of its own, not inside the text, so a `<loc>` is assembled from text,
  CDATA and references. Wiremock answers an unmatched path 404, so every
  existing crawl test now probes two `sitemap.xml`s and writes nothing for
  them; only a test counting requests had to learn to ignore them.
