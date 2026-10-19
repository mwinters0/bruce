# Deploying `bruce`

A **deployment** is one site's configuration, the scheduled job that runs
`bruce crawl` with it, and wherever its results are kept. `bruce` ships none:
the configuration of a site belongs with whoever runs the crawl. This guide
shows how to write one, using docs.fedoraproject.org as the worked example.
The configuration keys are explained in [`config.md`](config.md), and
`bruce crawl` in [`crawl.md`](crawl.md); this guide says which to write for a
site like this one, and why.

This is the configuration part of the guide. Scheduling a crawl and publishing
its results come later.

The code blocks marked `file=<path>` are the files of the example: this
repository extracts them into a directory and checks that `bruce` accepts the
configuration (`mise run sim`), so the example cannot drift from the tool.
Blocks that share a path are one file, in the order they appear here.

## The configuration

Write the file as `bruce.toml`. Every key is optional but a root's `url`, so
the file below says only what docs.fedoraproject.org needs.

### Who is crawling

```toml file=bruce.toml
user_agent = "bruce-fedora-docs/1.0 (+https://docs.fedoraproject.org/)"
```

Every request names the crawler. Replace the address in the parentheses with
one where a site operator can reach whoever runs your crawl. The first word is
what a site's `robots.txt` knows the crawler by, here `bruce-fedora-docs`
([`config.md`](config.md#user-agent)).

### Which links are checked on other hosts

```toml file=bruce.toml
allowlist = ["forge.fedoraproject.org"]
```

Pages on docs.fedoraproject.org link to the Fedora forge for every page's edit
and history links, and a moved source file shows only if those links are
checked. `allowlist` makes `bruce` fetch each such link's target once, so its
`#fragment` can be checked, without ever crawling the host. A link to any other
host is recorded and not checked; add a host to the list to check its links
too.

### Which links are never asked about

```toml file=bruce.toml
skip = [
  "https://forge.fedoraproject.org/**/issues/new*",
  "https://forge.fedoraproject.org/user/login*",
]
```

Without a login, the forge answers `issues/new` with a redirect to
`/user/login` whether or not the repository exists. Such a link would be
reported as working, so it tells nothing, and every page carries one. A link
matching a `skip` glob is recorded as not checked, with the reason `skipped`,
and no request is sent.

The second glob is the login page itself. A link that redirects to a skipped
URL stops there, so any other link that the forge sends to the login page is
also marked not checked, rather than followed. `*` stops at a `/`, `**` does
not, and the query is part of what is matched ([`config.md`](config.md#skip)).

### How fast each host is asked

```toml file=bruce.toml
[host."forge.fedoraproject.org"]
per_second = 1
```

docs.fedoraproject.org asks crawlers for one second between requests
(`Crawl-delay: 1`), and `bruce` obeys it with no setting at all. The forge asks
the same. Writing the rate for the forge keeps the crawl that polite if its
`robots.txt` ever drops the request: on a host that no root starts on, the
slower of `per_second` and `Crawl-delay` applies.

A rate written for docs.fedoraproject.org itself would beat its `Crawl-delay`.
Write one only when you have the site's permission to crawl faster
([`config.md`](config.md#hosts)). The example leaves it out, and a crawl of the
English pages then takes one to two hours.

### What is crawled

```toml file=bruce.toml
[[root]]
url = "https://docs.fedoraproject.org/en-US/"
sitemap = "https://docs.fedoraproject.org/en-US/sitemap.xml"
```

The root is the English pages: every page whose address begins with the root's
URL is fetched, and every link on it recorded. The site's other locales have
other addresses, so they are not crawled.

The sitemap is an index of one sitemap per component. `bruce` would find it
unaided, at `sitemap.xml` under the root; writing it down saves the probe and
shows the next deployer where it is. It is also how `bruce` knows which pages
the site means to publish, as opposed to those it only found by following links.

## Trying it

With the file saved as `bruce.toml`, a crawl is one command:

```sh
bruce crawl --config bruce.toml --db bruce.duckdb
```

It runs for as long as the hosts' `Crawl-delay` makes it. Start a long crawl
detached from your terminal, with its output in a file:

```sh
setsid bruce crawl --config bruce.toml --db bruce.duckdb > crawl.log 2>&1 &
```

and read the log, and the run it wrote, when it has ended
([`crawl.md`](crawl.md#output)).
