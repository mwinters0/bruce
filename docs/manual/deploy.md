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
one where a site operator can reach whoever runs your crawl. The value up to
its first `/` or space is what a site's `robots.txt` knows the crawler by,
here `bruce-fedora-docs`
([`config.md`](config.md#user-agent)).

### Which links are checked on other hosts

```toml file=bruce.toml
allowlist = [
  "fedoraproject.org", "*.fedoraproject.org",
  "getfedora.org", "*.getfedora.org",
  "fedorapeople.org", "*.fedorapeople.org",
  "fedoramagazine.org", "*.fedoramagazine.org",
  "fedorainfracloud.org", "*.fedorainfracloud.org",
  "fedorahosted.org", "*.fedorahosted.org",
  "pagure.io", "*.pagure.io",
  "centos.org", "*.centos.org",
  "redhat.com", "*.redhat.com",
]
```

`allowlist` makes `bruce` fetch each link's target on these hosts once, so its
`#fragment` can be checked, without ever crawling the host. A link to any other
host is recorded and not checked; add a host to the list to check its links
too.

The list is the Fedora and Red Hat sites, since those are the links the docs
answer for: the Fedora forge, which every page links to for its edit and
history links, the wiki, the mailing lists, Bugzilla, the package sources, the
magazine, the retired fedorahosted.org and pagure.io, CentOS and Red Hat's
documentation. The English pages link to about 160 hosts under these domains.
Each domain is written twice, since `"*.fedoraproject.org"` names every host
below `fedoraproject.org`, the forge among them, but not `fedoraproject.org`
itself, the wiki. Naming domains rather than hosts keeps the list current as
hosts come and go.

A `*.` entry never names the host a root starts on, so the site's other
locales on `docs.fedoraproject.org` stay unchecked
([`config.md`](config.md#allowlist)).

A link to a staging or internal host under these domains is then checked like
any other, and reported as what it is to a reader: broken where the page is
missing, an error where the host asks for a login or fails. A host whose name
does not resolve outside Red Hat's network is `broken_url`, and one that
refuses or never answers is an `error`, the reason saying which (`nxdomain`,
`connect`, `timeout`): `bruce` reads a host's `robots.txt` before anything else
on it, and the host's silence there is the answer for every link to it. A
host's `robots.txt` can also forbid a link: Bugzilla's allows only its bug pages, and
fedorapeople.org's allows nothing, so those links are reported as not checked,
with the reason `robots`.

### Which links are never asked about

```toml file=bruce.toml
skip = [
  "https://forge.fedoraproject.org/**/issues/new*",
  "https://forge.fedoraproject.org/**/issues/new*/**",
  "https://forge.fedoraproject.org/user/login*",
]
```

Without a login, the forge answers `issues/new` with a redirect to
`/user/login` whether or not the repository exists. Such a link would be
reported as working, so it tells nothing, and every page carries one. A link
matching a `skip` glob is recorded as not checked, with the reason `skipped`,
and no request is sent.

The page's edit link carries its path in the query, as
`issues/new?title=Doc%20issue%20in%20file%20modules/ROOT/pages/index.adoc`,
and `*` stops at a `/`. The first glob matches a link whose query holds none;
the second matches one whose query does, and `issues/new/choose`.

The last glob is the login page itself. A link that redirects to a skipped
URL stops there, so any other link that the forge sends to the login page is
also marked not checked, rather than followed. `*` stops at a `/`, `**` does
not, and the query is part of what is matched ([`config.md`](config.md#skip)).

### How fast each host is asked

```toml file=bruce.toml
[host."docs.fedoraproject.org"]
robots = false
per_second = 1

[host."forge.fedoraproject.org"]
per_second = 1
```

docs.fedoraproject.org's `robots.txt` asks crawlers for one second between
requests (`Crawl-delay: 1`) and disallows some paths. Its disallows are
written for search engines, and a crawl of the site's own links is
better without them: they hid nine broken links from live pages, each a 404
for anyone who follows it. So the example turns `robots.txt` off for this host
and writes the same second as its own rate, which keeps the crawl as polite as
the site asked.

That is a choice open to whoever may speak for the site, since only a host that
a root starts on may skip its `robots.txt` ([`config.md`](config.md#hosts)). If
you control the site, you can instead add a group to its `robots.txt` for your
crawler's name (here `bruce-fedora-docs`), holding only `Crawl-delay: 1`. A
crawler obeys the most specific group that names it, so the group replaces the
disallows meant for others, and `robots` stays on. For a site you do not
control, leave `robots` on and write no rate: `bruce` obeys its `Crawl-delay`
with no setting at all.

The forge keeps its `robots.txt`: a host that no root starts on is always
obeyed. It asks for the same second between requests. Writing the rate keeps
the crawl that polite if its `robots.txt` ever drops the request, since there
the slower of `per_second` and `Crawl-delay` applies.

The other hosts on the allowlist need no section: each is asked at its own
`robots.txt`'s `Crawl-delay`, five seconds for Bugzilla and ten for the mailing
lists and www.redhat.com, and at the default two requests a second where there
is none. They are asked alongside the crawl, each at its own pace.

A rate faster than the site asked for needs its permission. The example asks
no faster, and a crawl of the English pages takes a little over an hour.

### What is crawled

```toml file=bruce.toml
[[root]]
url = "https://docs.fedoraproject.org/en-US/"
sitemap = "https://docs.fedoraproject.org/en-US/sitemap.xml"
```

The root is the English pages: every page whose address begins with the root's
URL is fetched, and every link on it recorded. The site's other locales have
other addresses, so they are not crawled, and the links to them are not
checked; a link to any other address on the site, such as the header's home
link `https://docs.fedoraproject.org/en-US` without its slash, is checked
once.

The sitemap is an index of one sitemap per component. `bruce` would find it
unaided, at `sitemap.xml` under the root; writing it down saves the probe and
shows the next deployer where it is. It is also how `bruce` knows which pages
the site means to publish, as opposed to those it only found by following links.

## Trying it

With the file saved as `bruce.toml`, a crawl is one command:

```sh
bruce crawl --config bruce.toml --db bruce.duckdb
```

It runs for as long as the hosts' request rates make it. Start a long crawl
detached from your terminal, with its output in a file:

```sh
setsid bruce crawl --config bruce.toml --db bruce.duckdb > crawl.log 2>&1 &
```

and read the log, and the run it wrote, when it has ended
([`crawl.md`](crawl.md#output)).
