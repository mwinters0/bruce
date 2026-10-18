# P3.1 Canonical URLs: notes

What P3.2 onward inherits, and what was refused.

- **`url::canonical(&Url, strip_query) -> Canonical`**, with `Canonical::as_str`
  and `Canonical::id` (`xxh3_64`, the `url_id`). The fragment is not in it:
  `links.fragment` is read from the `Url`. The form is built from the `url`
  crate's own: only `%XX` is added, in the path and the query alike.
- **Callers canonicalise per root.** `Root::canonical(&Url)` applies that
  root's `strip_query`, and `Root::in_scope` takes the result, so a crawler
  holding a URL asks each root for its canonical form and tests scope on it.
  A URL no root claims has no `strip_query` to apply and has the plain form,
  `canonical(url, false)` (`crawl::frontier`), which is the `url_id` of every
  `unchecked` and allowlisted target.
- **`config` now reads `url`**, so D39's layering reads `config` › `url`.
- **An empty query stays**: `/p?` and `/p` are two URLs, as `/p` and `/p/` are.
  Under `strip_query` both are `/p`.
- **`url` already did the rest**: default ports, case, punycode, dot-segments
  (`%2e%2e` included), an empty path. Tests pin each, so an upgrade of the
  crate that changes one fails here rather than in a stored ID.
- **Negative result**: the `url` crate leaves `%7e` and `%2f` as written, and
  does not uppercase hex, so the escapes cannot be left to it.
- **Not done, left as it was**: the duplicate-root check in `Config::parse`
  compares `Root.url` as written, so two roots spelt `/~x/` and `/%7Ex/` are
  not seen as one. Nothing yet depends on it; the crawler's seeding (P3.5)
  deduplicates by `url_id` and would settle it.
