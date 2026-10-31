# Configuration reference

`bruce` needs no configuration file. Given only a URL, it crawls that site
with the defaults below. A file, in TOML, changes any of them, and can name
more than one site to crawl.

Every key is optional except a root's `url`. A key `bruce` does not know is an
error, so a misspelt key never passes silently as a default.

## Example

```toml
allowlist = ["github.com", "gitlab.com"]
skip = [
  "https://github.com/*/*/issues/new*",
  "https://github.com/*/*/issues/new*/**",
]
user_agent = "docs-example-bot/1.0 (+https://docs.example.org/bot)"

[default]
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
| `profile` | `"auto"` | How the root's pages are read: `"auto"`, `"generic"`, `"antora"` or the name of a [profile](#profiles) the file defines. |
| `hash_root` | `[]` | CSS selectors for the part of each page whose change counts as the page changing (below). |
| `hash_exclude` | `[]` | CSS selectors for parts of each page whose change never counts (below). |

**The sitemap** may be an index of further sitemaps, followed three files
deep, and any of them may be gzip-compressed (`.xml.gz`). The pages it lists
outside every root's scope are dropped. When none is found, the crawl follows links
alone.

The `url` must be in the root's own scope: a root whose `include` or `exclude`
rules out its starting page is an error.

**A link outside every root's scope** to a host a root starts on is checked
as a link to an [allowlisted](#allowlist) host is: fetched once, and never
crawled. One to another locale's page is recorded and not checked (reason
`out_of_scope`): another locale is learned from the language menu of a root's
`antora` pages, so a site without one has every such link checked.

**The path prefix** is the root's URL in the form under "Writing globs",
without its query, up to the last `/`.
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
`.` and `..` segments resolved, and a `%XX` escape in the path or query decoded where it stands for
a letter, digit, `-`, `.`, `_` or `~`, so `%7Euser` is `~user`. Every other
escape stays, with its hex digits in capitals (`%2F`, `%C3%A9`). Write a glob as
the URL reads in that form: one written with `%7E` never matches. The `#fragment`
is never part of it.

**`strip_query`** makes `https://docs.example.org/en-US/a/?v=1` and
`https://docs.example.org/en-US/a/?v=2` one page, for a site that serves the
same page whatever the query says. The query is then dropped from the URLs in
this root's scope before they are compared or matched against its `include`
and `exclude` globs (a [`skip`](#skip) glob still sees it); it
is kept by default, since a query can pick a different page.

**Profiles** know how a site generator lays out its pages.
- `"generic"` reads any HTML page.
- `"antora"` also reads what an Antora site adds, such as each page's
  component and version.
- `"auto"` picks one for each page from its `<meta name="generator">`, and
  uses `"generic"` when there is none or no profile claims it.
- A name picks a [profile the file defines](#profiles).

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
The database records which scope covered each page (`pages.hash_scope`:
`root` for `hash_root`, `region` for the default, `whole` for the rest) and
each run's `hash_root` and `hash_exclude` per root (`runs.hash_scopes`), so a
report can tell an edit to the scope from an edit to the page.

## Profiles

A **profile** the file defines reads a site generator that `generic` and
`antora` do not know, with no change to `bruce`. A `[[profile]]` table names it
with `name`, and a root's `profile = "<name>"` (or a generator pattern, below)
picks it. The name is a letter, then letters, digits, `-` and `_`; `auto`,
`generic` and `antora` are taken, and a name given twice is refused. It starts
every fact the profile records: `<name>.<key>`. Under `"auto"`, the profiles are
tried in the order the file writes them, before `antora`.

| Key | Default | Meaning |
|---|---|---|
| `profile.name` | required | What a root's `profile` names, and the start of each fact's key. |
| `profile.generator` | none | A regular expression. Under `"auto"`, a page whose `<meta name="generator">` it matches is read by this profile. Without one, only a root naming the profile picks it. |
| `profile.region` | `generic`'s: `main`, `[role=main]` | Selectors for the elements holding a page's content. Anchors there count as content, and the [content hash](#roots) covers them by default. Each entry is one selector with no space or combinator, as `div.document`. |
| `profile.edit_link` | none | A selector for the link that is the page's edit link, for a site whose link `bruce` does not recognise. The first match with an `href` is taken. |

Each `[[profile.fact]]` table, written under the `[[profile]]` it belongs to, reads one fact from a page:

| Key | Default | Meaning |
|---|---|---|
| `profile.fact.key` | required | The fact's name: letters, digits, `-` and `_`. One fact to a key. |
| `profile.fact.selector` | required | The elements to read. |
| `profile.fact.attr` | none | The attribute to read, as written, an empty one included. Without it the value is the element's text, trimmed. |
| `profile.fact.pattern` | none | A regular expression with exactly one capture group, which is the value. An element it does not match gives nothing. |

The first element that gives a value gives the fact; a page with none records
no fact. Selectors are the ones `hash_root` takes, and a regular expression is
Rust's [`regex`](https://docs.rs/regex) syntax, unanchored, so write `^` and `$`
where you mean them.

```toml
[[profile]]
name = "sphinx"
generator = "^Sphinx"
region = ["div.document"]

[[profile.fact]]
key = "version"
selector = "meta[name=readthedocs-version-slug]"
attr = "content"

[[profile.fact]]
key = "heading"
selector = "h1"
pattern = '^(.*?)\s*¶?$'
```

`[digest]` `group_by = "sphinx.version"` then groups the digest's pages by the
fact. A run records each profile's definition with its other settings
(`profile.<name>.*`) and the digest's Housekeeping lists it; `--config` on a
report tries another file's judging settings and leaves the recorded profiles
as they are.

## Allowlist

`allowlist` lists the external hosts whose links are checked. Each such link's
target is fetched once, so its `#fragment` can be checked, but the host is
never crawled. A link to any other host is recorded and not checked.

An entry is a bare host name, such as `"github.com"`, naming that host alone.
An entry `"*.<domain>"`, such as `"*.fedoraproject.org"`, names every host
below the domain at any depth: `wiki.fedoraproject.org` and
`a.b.fedoraproject.org`, but not `fedoraproject.org` itself, which is an entry
of its own. To check the domain and its hosts, write both:

```toml
allowlist = ["fedoraproject.org", "*.fedoraproject.org"]
```

A `*.` entry never names a host a root starts on, so
`"*.fedoraproject.org"` beside a root on `docs.fedoraproject.org` leaves the
site's other locales unchecked ([Roots](#roots)). Naming such a host outright
is an error: a link to it outside every root's scope is checked already, and
another locale's pages are checked by a root of their own.

A `*` anywhere but at the start, as `*.`, is an error, and so is a `*.` entry
whose domain holds no dot, such as `"*.org"`.

| Key | Default | Meaning |
|---|---|---|
| `allowlist` | `[]` | Host names such as `"github.com"`, or `"*.<domain>"` for every host below a domain. No scheme, port or path. |

## Skip

`skip` lists URL globs for links that are never asked about, on any host,
inside a root's scope as well as outside it. A link whose URL matches is
recorded with the outcome `unchecked` and the reason `skipped`, and no request
is sent. A page a sitemap lists that matches is dropped, as one outside every
root is; a sitemap file is read whatever `skip` says, unless a redirect leads
to it. Use it for a URL that answers the same whether or not the page it
names exists, such as a forge's `issues/new` redirecting anonymous requests to
a login page.

A glob matches the whole URL as `include` does ([Roots](#roots)), with its
query: `*` and `?` stop at `/`, `**` crosses it, so `issues/new*` matches
`issues/new?title=x` and not `issues/new?title=a/b`, a query holding a `/`,
which `issues/new*/**` also matches. A glob that matches a root's own `url`
is an error. A redirect that leads to a skipped URL ends there, the link
`unchecked` with the reason `skipped`, so skipping a login page marks every
link that redirects to it.

| Key | Default | Meaning |
|---|---|---|
| `skip` | `[]` | URL globs, such as `"https://github.com/*/*/issues/new*"` and `"https://github.com/*/*/issues/new*/**"`. |

## Defaults

`[default]` holds the settings a host takes unless `[host]` (below) sets its own;
a file with a `[rate]` table is refused, the table having been renamed.

Requests are limited per host. `default.per_second` is the rate for a host that
nothing else governs; a `Crawl-delay` in a host's `robots.txt` takes
precedence over it, and `[host]` (below) says when a host's own rate does.

| Key | Default | Meaning |
|---|---|---|
| `default.per_second` | `2` | Requests a second to any one host. |
| `default.concurrency` | `4` | Requests in flight to any one host at once. At the default rate it binds only on a host slow to answer. At least 1. |
| `default.doubled_slash` | `true` | Whether a link to a URL that doubles a slash is broken, for a host that sets none ([Judging](#judging)). |
| `default.status` | `{ 401 = "error", 403 = "error", 429 = "error" }` | What a status code is judged to be, for a host that sets none; the file's codes are laid over these, a code at a time ([Judging](#judging)). |
| `default.connect_timeout` | `10` | Seconds to connect to a host, for a host that sets none ([Asking](#asking)). At least 1. |
| `default.timeout` | `30` | Seconds for a whole response, body included, for a host that sets none ([Asking](#asking)). At least 1. |
| `default.retries` | `3` | Times a request answered by a 5xx or a 429, or failed by a timeout or a lost connection, is sent again, for a host that sets none ([Asking](#asking)). At most 10. |
| `default.max_retry_after` | `60` | Seconds of the longest `Retry-After` waited out, for a host that sets none ([Asking](#asking)). |
| `default.slash_probe` | `true` | Whether a broken 4xx is asked again with its final `/` toggled, for a host that sets none ([Asking](#asking)). |

## Judging

A run **records the policy it was crawled with**, and a report reads the run,
never the file: a run is judged by its own policy, and changing a key changes
the runs that follow. A comparison (`check-dir`'s gate, `report diff`, a digest's
changes since) judges both runs by the policy of the run it reports on, the newer in an ordinary one, so changing a key breaks nothing by itself. Every key is at the value `bruce` has always judged by.
To see what a key does to a run already crawled, [`bruce report --config`](report.md#a-trial-policy)
judges it by a file's keys instead, writing nothing.

```toml
[judge]
pedantic_trailing_slash = false
schemes = ["matrix"]

[default]
status = { 403 = "ok" }

[host."pagure.io"]
doubled_slash = false
status = { 403 = "broken_url" }
```

| Key | Default | Meaning |
|---|---|---|
| `judge.pedantic_trailing_slash` | `true` | Whether a redirect that only adds or drops a final `/` is `ok`, reason `pedantic_trailing_slash`, rather than a redirect to correct. |
| `judge.doubled_slash_reach` | `"full"` | How far the breakage a doubled slash causes is counted on the page that links to it (below): `"full"`, `"immediate"` or `"off"`. |
| `judge.schemes` | `[]` | Schemes counted `unchecked`, reason `scheme`, besides the built-in `mailto`, `tel`, `irc`, `ircs`, `xmpp`, `appstream`, `ftp`, `news`, `magnet`, `sms`, `data` and `javascript`. Any other scheme but `http(s)` is `broken_url`, reason `bad_scheme`. Written in any case; `http` and `https` are refused. |

**`doubled_slash`** is the setting of the host in the link's URL. Off, a link to
`https://host/a//b` is judged by what the URL answered, and a page at such a URL
counts its own links as any page does.

**`doubled_slash_reach`** says what the digest counts on the page that writes a
link to a URL that doubles a slash, as "broken through it". A page at such a URL
with no twin in the run counts none of its broken relative links (`../`,
`committee/`, `?query`), which the doubled slash breaks, whatever the reach.
`full` counts them on the linking page for every such page its link reaches,
following their relative links on; `immediate` counts those of the page the link
lands on alone; `off` counts them nowhere, the link itself being the one red
finding. The digest's guide states the reach the run was judged by.

**`status`** maps single codes, `"<code>" = "<outcome>"`, the outcome `ok`,
`broken_url` or `error`. A class such as `4xx` is refused, and so is a 3xx,
which is a hop the crawl follows. The host judged is the one that **answered**,
the last of a redirect's hops. A code no table names is judged as it was
crawled: a 4xx other than 401, 403 and 429 is `broken_url`, a 5xx `error`. A map that gives
the outcome the crawl stored keeps the crawl's reason (`wrong_trailing_slash`, `too_large`);
any other drops it. A fragment on a URL judged `ok`
by this map is `unchecked`, reason `no_anchors`, as on any `ok` URL whose body
was not read.

## Asking

How a crawl **asks** is recorded with the policy and judged by nothing: a run
cannot be asked again, so [`bruce report --config`](report.md#a-trial-policy)
leaves it as the run recorded it, and the digest's Housekeeping lists what
differs from the defaults. The `default.*` keys above and the `host."<host>".*`
keys set it per host, the host's own over the default; the body cap is the
site's.

```toml
max_body_mib = 64

[default]
timeout = 60

[host."pagure.io"]
connect_timeout = 5
retries = 1
slash_probe = false
```

| Key | Default | Meaning |
|---|---|---|
| `max_body_mib` | `16` | MiB a page's body may come to. A larger one is `ok`, reason `too_large`, and is not read for its anchors; `check-dir` holds a built file to it too. At least 1. |

**Retries** are of a 5xx, a 429, a timeout or a lost connection. The waits
double from a second: `retries = 3` waits 1, 2 and 4 seconds, and `0` sends
nothing again. A 429 or 503 naming a `Retry-After` is waited out instead, when
it is no longer than `max_retry_after`; a longer one is `error` at once, and
`0` waits out only a `Retry-After` of `0` or a date already past. The retries and the timeout of a host are
the ones its robots.txt is read with too.

**`slash_probe`** is the host of the URL asked: off, a broken 4xx on that host is
`broken_url` with no `wrong_trailing_slash` reason and no request for it.

## The gate

`[gate]` `fail` names the outcomes that fail [`check-dir`](check-dir.md#what-fails)
and are `bruce report introduced`'s rows, `broken_url`, `broken_anchor` and `error`,
each once, in any order. A site that does not want a missing anchor to fail CI
leaves `broken_anchor` out; one that wants a 5xx to fail it adds `error`. The
list is the **CI run's**: it judges the baseline's links as well, as its other
judging keys do. An outcome left out stays in `bruce report broken`, the digest and
every other report, and `check-dir` lists the introduced links at it in a section
that never fails. The inverse check follows the list: a link to a page or a file
the change removed will read `broken_url`, and one to an anchor `broken_anchor`,
so a row fails where the list names that outcome and is listed apart where it does
not; `check-dir --inverse warn` only ever loosens it. An empty list, `fail = []`,
is report only: what the change introduced is listed and nothing fails, the
breakage already in the baseline in no section, and the exit status 2 still
meaning the check could not run. A redirect is no outcome.

| Key | Default | Meaning |
|---|---|---|
| `gate.fail` | `["broken_url", "broken_anchor"]` | The outcomes that fail the gate. Recorded as one row, and the digest's Housekeeping writes it as the file does where it differs. |

```toml
[gate]
fail = ["broken_url"]
```

## Severity

`[severity]` sets how loudly [the digest](digest.md#the-html) shows each kind of
finding, with no effect on any outcome, any report or the gate (the gate's own
list is [`[gate]`](#the-gate)). A level is
`"broken"` (a red dot), `"attention"` (amber), `"pedantic"` (a grey ring) or
`"none"` (no finding); the colours are fixed. A key is an outcome, `redirected` for
a redirected link, or a reason, and a reason's level beats its outcome's, so
`doubled_slash = "attention"` softens that kind of `broken_url` alone. `unchecked`
keeps no level, and a key `bruce` does not know is refused. The run records the map
with its other settings, and the digest's legend, its dots, its counts and
ranking and the guide's pedantic entry follow it: a page's and a repo's counts
are of their links by level, and they rank by their red links, then their amber
ones, so `doubled_slash = "attention"` moves the page counting those links down
the ranking. A part of a count that a reason's key moved off its outcome's level names that reason,
"● 2 broken links (`doubled_slash`)". A kind at `"none"` is no finding in the digest: it leaves the link
lists, the counts and the ranking, a page whose only findings are at `"none"` has
no row, and the legend names what it leaves out; a list the map empties says so, "By: Link Target" and the
sections since the compared run included. The run's outcomes, the figures' numbers,
the JSON's rows and order, every other report and the gate are untouched. A level on a link
that is neither broken nor errored (a redirect, a reason on an `ok` link) moves
its dot where it is listed and counts nothing.

```toml
[severity]
doubled_slash = "attention"
pedantic_trailing_slash = "none"
```

| Kind | Default level |
|---|---|
| `broken_url` | `"broken"` |
| `error` | `"broken"` |
| `broken_anchor` | `"attention"` |
| `redirected` | `"attention"` |
| `pedantic_trailing_slash` | `"pedantic"` |

A reason not listed has no level of its own and takes its outcome's: `nxdomain`,
`timeout`, `tls`, `connect`, `reset`, `other`, `unreadable` (a page or sitemap that
answered and could not be read as one), `redirect_loop`, `too_many_redirects`,
`bad_redirect`, `too_large`, `unresolved_xref`, `not_built` (a URL in a component
`check-dir` holds whole and the build lacks), `bad_scheme`, `invalid_url` (an href
that resolves to no URL), `doubled_slash` and `wrong_trailing_slash`.

## Digest

`[digest]` `group_by` names a fact the pages carry, by its full key
(`antora.component`: a profile's name, a dot and the fact), and the
[digest's JSON](digest.md#the-json) then holds `by_group`, the crawled pages
grouped by that fact's value and ranked as `by_repo` is. `bruce` groups by nothing
unless a site says so, and `by_group` is then empty; a key no page carries groups
nothing either. A run records the key with its other settings, and a report reads it
from the run, so `bruce report digest --config` tries another.

| Key | Default | Meaning |
|---|---|---|
| `digest.group_by` | none | The fact whose values group the digest's pages. A page without the fact is in no group. |

```toml
[digest]
group_by = "antora.component"
```

## User agent

Every request names `bruce` and its version, `bruce/0.2.0`. `user_agent` names
your deployment instead, so a site operator can tell whose crawler it is.
`check-dir`'s download of a `--baseline` URL always names `bruce`.

| Key | Default | Meaning |
|---|---|---|
| `user_agent` | `bruce/<version>` | Sent as `User-Agent`. Printable ASCII, a space allowed, starting with a product name of letters, digits, `-`, `_` or `.`. |

A site's `robots.txt` knows the crawler by that first product name, lowercased:
`docs-example-bot` for the example above, `bruce` by default. A `robots.txt`
group written for `User-agent: docs-example-bot` is the one that applies.

## Hosts

`[host."<host>"]` changes how one host is fetched, whether or not a root starts
on it. The name is a bare host name, as in `allowlist`; a `*.` domain is
`allowlist`'s alone, and each host below it takes its own rate. A host that a root
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
| `host."<host>".concurrency` | `default.concurrency` | Requests in flight to that host at once. |
| `host."<host>".robots` | `true` | Whether the host's `robots.txt` is obeyed, its `Crawl-delay` included. |
| `host."<host>".doubled_slash` | `default.doubled_slash` | Whether a link to a URL on that host that doubles a slash is broken ([Judging](#judging)). |
| `host."<host>".status` | `{}` | What a status code that host answered is judged to be, over `default.status` ([Judging](#judging)). |
| `host."<host>".connect_timeout` | `default.connect_timeout` | Seconds to connect to that host ([Asking](#asking)). |
| `host."<host>".timeout` | `default.timeout` | Seconds for a whole response from that host ([Asking](#asking)). |
| `host."<host>".retries` | `default.retries` | Times a request to that host is sent again after a 5xx, a 429, a timeout or a lost connection ([Asking](#asking)). |
| `host."<host>".max_retry_after` | `default.max_retry_after` | Seconds of the longest `Retry-After` that host's answer is waited out for ([Asking](#asking)). |
| `host."<host>".slash_probe` | `default.slash_probe` | Whether a broken 4xx on that host is asked again with its final `/` toggled ([Asking](#asking)). |

**Which rate applies** to a host:
- With `robots = false`: its `per_second` if written, else `default.per_second`.
- On a host a root starts on, a `per_second` beats the host's `Crawl-delay`.
  Write one only for a site you have permission to crawl faster.
- On any other host, the slower of its `per_second` and its `Crawl-delay`.
- With no `per_second` written, the `Crawl-delay` if there is one, else
  `default.per_second`.

`robots = false` is an error on a host that no root starts on: an external
host's `robots.txt` is always obeyed.

**What `robots.txt` does** is as RFC 9309 says. Each origin's (scheme, host
and port) is read once a run, before any other request to it; the rate, the
requests in flight and `robots` stay a host's, and the slowest `Crawl-delay`
among a host's origins sets its pace. A URL a rule forbids is not requested and
is reported as not checked. If the file is absent (a 4xx answer), everything
is allowed. If the host answers but the file cannot be read (a 5xx, a 429
that outlasts the retries or the `Retry-After` ceiling, or redirects that lead nowhere: more than five, one
with no web address to follow, or one to another host that does not answer),
nothing on that origin is requested for the run, and its links are reported as
not checked. A `robots.txt` that redirects to another origin's reads that
origin's file. If the host itself does not answer on that origin, every link to
it is reported as a request for it would have been, for the whole run: `broken_url` where its name
does not exist, an `error` where it refuses, times out or fails the TLS
handshake, the reason saying which. A `Crawl-delay` of more than an hour is
read as an hour.
