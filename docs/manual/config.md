# Configuration reference

`bruce` needs no configuration file. Given only a URL, it crawls that site
with the defaults below. A file, in TOML, changes any of them, and can name
more than one site to crawl.

Every key is optional except a root's `url`. A key `bruce` does not know is an
error, so a misspelt key never passes silently as a default.

## Example

```toml
allowlist = ["github.com", "gitlab.com"]
user_agent = "docs-example-bot/1.0 (+https://docs.example.org/bot)"

[rate]
per_second = 2
concurrency = 4

[host."github.com"]
per_second = 1

[[root]]
url = "https://docs.example.org/en-US/"
exclude = ["https://docs.example.org/en-US/archive/**"]
profile = "auto"
```

## Roots

A **root** is a site crawled fully: every page in its scope is fetched, and
every link on those pages is recorded. Each root is a `[[root]]` table.

With no file, the URL given to `bruce crawl` is the only root, with every key
below at its default. A file naming no root takes the URL as its only root,
and a file naming roots refuses one ([`crawl.md`](crawl.md)).

| Key | Default | Meaning |
|---|---|---|
| `url` | required | Where the crawl starts. An `http` or `https` URL with no `#fragment`. |
| `include` | `[]` | URL globs a page must match to be crawled. Empty means everything under the root's path prefix (below). |
| `exclude` | `[]` | URL globs that keep a page out, even when `include` matches it. |
| `strip_query` | `false` | Whether a URL's `?query` is dropped, in this root's scope, when `bruce` decides which URLs are the same page (below). |
| `sitemap` | discovered | The sitemap to start from. By default, the `Sitemap:` lines of the host's `robots.txt`, then `sitemap.xml` under the root, then `sitemap.xml` at the host. |
| `profile` | `"auto"` | How the root's pages are read: `"auto"`, `"generic"` or `"antora"` (below). |
| `hash_root` | `[]` | CSS selectors for the part of each page whose change counts as the page changing (below). |
| `hash_exclude` | `[]` | CSS selectors for parts of each page whose change never counts (below). |

**The sitemap** may be an index of further sitemaps, followed three files
deep, and any of them may be gzip-compressed (`.xml.gz`). Its entries outside
every root's scope are dropped. When none is found, the crawl follows links
alone.

The `url` must be in the root's own scope: a root whose `include` or `exclude`
rules out its starting page is an error.

**The path prefix** is the root's URL without its query, up to the last `/`.
For `https://docs.example.org/en-US/` it is the URL itself. For
`https://docs.example.org/en-US/index.html` it is
`https://docs.example.org/en-US/`.

**Globs** match the whole URL, scheme included. `*` and `?` match within one
path segment, `**` across segments, and `[ab]` one of a set of characters.
For example:

```toml
[[root]]
url = "https://docs.example.org/en-US/fedora/"
include = ["https://docs.example.org/*/fedora/**"]
exclude = ["https://docs.example.org/*/fedora/f2?/**"]
```

crawls every locale's `fedora/` pages, except versions `f20` to `f29`.

**Writing globs.** A glob is matched against a URL in the form `bruce`
treats as its one spelling: the scheme and host in lower case, no default port,
`.` and `..` segments resolved, and a `%XX` escape decoded where it stands for
a letter, digit, `-`, `.`, `_` or `~`, so `%7Euser` is `~user`. Every other
escape stays, with its hex digits in capitals (`%2F`, `%C3%A9`). Write a glob as
the URL reads in that form: one written with `%7E` never matches. The `#fragment`
is never part of it.

**`strip_query`** makes `https://docs.example.org/en-US/a/?v=1` and
`https://docs.example.org/en-US/a/?v=2` one page, for a site that serves the
same page whatever the query says. The query is then dropped from the URLs in
this root's scope before they are compared or matched against globs; it
is kept by default, since a query can pick a different page.

**Profiles** know how a site generator lays out its pages.
- `"generic"` reads any HTML page.
- `"antora"` also reads what an Antora site adds, such as each page's
  component and version.
- `"auto"` picks one for each page from its `<meta name="generator">`, and
  uses `"generic"` when there is none.

**The content hash** tells a changed page from an unchanged one. It covers
each page's tags, attributes and text within a scope that `hash_root` and
`hash_exclude` set:

| `hash_root` | `hash_exclude` | What is hashed |
|---|---|---|
| unset | unset | The page's content region, as its profile finds it, or the whole page where it has none. |
| set | unset | The elements `hash_root` matches. |
| unset | set | The whole page, minus the elements `hash_exclude` matches. |
| set | set | The elements `hash_root` matches, minus those `hash_exclude` matches. |

A page where nothing matches the root, set or default, is hashed whole, less
the exclusions. Exclude a part of the page that changes at every build, such
as a footer stamped with the build time, and the rest of the page, its
stylesheet and script links included, is still watched:

```toml
[[root]]
url = "https://docs.example.org/en-US/"
hash_exclude = ["footer"]
```

Each entry is one selector, or several separated by commas. Tag, class, id
and attribute selectors, descendant (` `) and child (`>`) combinators,
`:not()`, `:first-child` and `:nth-child()` are accepted; sibling
combinators (`+`, `~`) and `:has()` are errors, since `bruce` reads a page
in one pass. Changing either key can change every page's hash, once.

## Allowlist

`allowlist` lists external hosts, as bare host names, whose links are checked.
Each such link's target is fetched once, so its `#fragment` can be checked,
but the host is never crawled. A link to any other host is recorded and not
checked.

| Key | Default | Meaning |
|---|---|---|
| `allowlist` | `[]` | Host names such as `"github.com"`. No scheme, port or path. |

## Rate

Requests are limited per host. `rate.per_second` is the rate for a host that
nothing else governs; a `Crawl-delay` in a host's `robots.txt` takes
precedence over it, and `[host]` (below) says when a host's own rate does.

| Key | Default | Meaning |
|---|---|---|
| `rate.per_second` | `2` | Requests a second to any one host. |
| `rate.concurrency` | `4` | Requests in flight to any one host at once. At the default rate it binds only on a host slow to answer. At least 1. |

## User agent

Every request names `bruce` and its version, `bruce/0.1.0`. `user_agent` names
your deployment instead, so a site operator can tell whose crawler it is.

| Key | Default | Meaning |
|---|---|---|
| `user_agent` | `bruce/<version>` | Sent as `User-Agent`. Visible ASCII, starting with a product name of letters, digits, `-`, `_` or `.`. |

A site's `robots.txt` knows the crawler by that first product name, lowercased:
`docs-example-bot` for the example above, `bruce` by default. A `robots.txt`
group written for `User-agent: docs-example-bot` is the one that applies.

## Hosts

`[host."<host>"]` changes how one host is fetched, whether or not a root starts
on it. The name is a bare host name, as in `allowlist`. A host that a root
starts on, and one that only a root's `include` or an `allowlist` reaches, are
both fine. `allowlist` says whether a host's links are checked; `[host]` says
how the host is fetched.

```toml
[[root]]
url = "https://docs.example.org/en-US/"

[host."docs.example.org"]
per_second = 10
```

| Key | Default | Meaning |
|---|---|---|
| `host."<host>".per_second` | none | Requests a second to that host. |
| `host."<host>".concurrency` | `rate.concurrency` | Requests in flight to that host at once. |
| `host."<host>".robots` | `true` | Whether the host's `robots.txt` is obeyed, its `Crawl-delay` included. |

**Which rate applies** to a host:
- With `robots = false`: its `per_second` if written, else `rate.per_second`.
- On a host a root starts on, a `per_second` beats the host's `Crawl-delay`.
  Write one only for a site you have permission to crawl faster.
- On any other host, the slower of its `per_second` and its `Crawl-delay`.
- With no `per_second` written, the `Crawl-delay` if there is one, else
  `rate.per_second`.

`robots = false` is an error on a host that no root starts on: an external
host's `robots.txt` is always obeyed.

**What `robots.txt` does** is as RFC 9309 says. Each host's is read once a
run, before any other request to it. A URL a rule forbids is not requested and
is reported as not checked. If the file is absent (a 4xx answer), everything
is allowed. If it cannot be read (a 5xx answer, no answer, more than five
redirects, or a 429 that outlasts the retries), the whole host is not
requested for the run, and its links are reported as not checked. A
`Crawl-delay` of more than an hour is read as an hour.
