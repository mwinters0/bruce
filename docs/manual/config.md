# Configuration reference

`bruce` needs no configuration file. Given only a URL, it crawls that site
with the defaults below. A file, in TOML, changes any of them, and can name
more than one site to crawl.

Every key is optional except a root's `url`. A key `bruce` does not know is an
error, so a misspelt key never passes silently as a default.

## Example

```toml
allowlist = ["github.com", "gitlab.com"]

[rate]
per_second = 2
host."github.com".per_second = 1

[[root]]
url = "https://docs.example.org/en-US/"
exclude = ["https://docs.example.org/en-US/archive/**"]
profile = "auto"
```

## Roots

A **root** is a site crawled fully: every page in its scope is fetched, and
every link on those pages is recorded. Each root is a `[[root]]` table.

With no file, the URL given is the only root, with every key below at its
default.

| Key | Default | Meaning |
|---|---|---|
| `url` | required | Where the crawl starts. An `http` or `https` URL with no `#fragment`. |
| `include` | `[]` | URL globs a page must match to be crawled. Empty means everything under the root's path prefix (below). |
| `exclude` | `[]` | URL globs that keep a page out, even when `include` matches it. |
| `sitemap` | discovered | The sitemap to start from. By default, the `Sitemap:` lines of the host's `robots.txt`, then `sitemap.xml` under the root, then `sitemap.xml` at the host. |
| `profile` | `"auto"` | How the root's pages are read: `"auto"`, `"generic"` or `"antora"` (below). |
| `robots` | `true` | Whether the root's `robots.txt` is obeyed, its `Crawl-delay` included. External hosts' `robots.txt` is always obeyed. |
| `crawl_delay` | none | Seconds between requests to the root's host. Overrides both `robots.txt`'s `Crawl-delay` and `[rate]`. Set it only for a site you have permission to crawl faster. |

**The path prefix** is the root's URL without its query, up to the last `/`.
For `https://docs.example.org/en-US/` it is the URL itself. For
`https://docs.example.org/en-US/index.html` it is
`https://docs.example.org/en-US/`.

**Globs** match the whole URL, scheme included. `*` and `?` match within one
path segment, `**` across segments, and `[ab]` one of a set of characters.
For example:

```toml
[[root]]
url = "https://docs.example.org/en-US/"
include = ["https://docs.example.org/*/fedora/**"]
exclude = ["https://docs.example.org/*/fedora/f2?/**"]
```

crawls every locale's `fedora/` pages, except versions `f20` to `f29`.

**Profiles** know how a site generator lays out its pages.
- `"generic"` reads any HTML page.
- `"antora"` also reads what an Antora site adds, such as each page's
  component and version.
- `"auto"` picks one for each page from its `<meta name="generator">`, and
  uses `"generic"` when there is none.

## Allowlist

`allowlist` lists external hosts, as bare host names, whose links are checked.
Each such link's target is fetched once, so its `#fragment` can be checked,
but the host is never crawled. A link to any other host is recorded and not
checked.

| Key | Default | Meaning |
|---|---|---|
| `allowlist` | `[]` | Host names such as `"github.com"`. No scheme, port or path. |

## Rate

Requests are limited per host. A root's `crawl_delay`, or the `Crawl-delay` in
its `robots.txt`, takes precedence. `[rate]` governs every request neither
covers.

| Key | Default | Meaning |
|---|---|---|
| `rate.per_second` | `2` | Requests a second to any one host. |
| `rate.host."<host>".per_second` | `rate.per_second` | Requests a second to that host. |
