# Configuration and scope

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. Which URLs a run owns and which it only asks about: the
TOML file and its keys, a root's scope, the allowlist, the `skip` list and the locales a root's pages name. `config`
holds the file and the scope matching; `crawl::frontier` plans each URL against them.

### D5 Configuration is optional TOML, and no flag overrides a key; roots are separate from the allowlist
`[[root]]`s are crawled fully, each with its own globs, sitemap and profile (D35); the `allowlist` names hosts asked one
hop deep (D3, D68); `[default]` holds pace and concurrency, `[host."<h>"]` those and robots (D8). Roots are configurable
so the tool is not tied to Fedora, and every key but a root's `url` and a `[[profile]]`'s rule fields has a default
(D40). `bruce crawl`'s URL stands in for a file's absent roots and nothing else, refused beside a file naming roots;
per-key flags wait for someone to ask. Keys are snake_case and `[[root]]` singular, as Cargo's `[[bin]]`. Rejected:
kebab-case, which lychee and mise do not use; `allowlist` inside `[host]`, a root's own host belonging there and in no
allowlist (D68). Code: `config::Config::parse`, `cli::crawl`.

### D3 Allowlisted external hosts are checked one hop deep and never crawled
An allowlisted target is asked once and its links never followed: a GET storing its anchors when an anchor link names it
first, else a HEAD (KD4). A link to any other host no root starts on is `unchecked`, `host_not_allowlisted`, with no
request, so the link graph stays whole. Rejected: crawling allowlisted hosts, which would make them roots by another
name (D5). Code: `crawl::frontier::Frontier::plan`.

### D10 Default scope: en-US, every fedora version; locales are a root that is off by default
The guide's root is `/en-US/`, every version: frozen f26–f43 pages are cheap and current ones link into them, so their
anchors must be known. Translated heading ids can break fragments, but the translation team owns that (I9). A URL on a
root's host in no root's scope is asked as an allowlisted one is (D3, D22), unless it is another locale's page (D67):
`unchecked`, `out_of_scope` (maintainer, 2026-10-24). Why: such a link can be broken (`https:/kubernetes.io`, a 404 on
docs.fp.o); the 2026-10-23 crawl asks 12 and leaves 154,569. Rejected: latest-only, leaving links to frozen versions
unverifiable; other locales one hop deep, 67 requests a page (I25); every such URL unasked, which hid three broken ones
on en-US.
Code: `crawl::frontier::Frontier::plan`. Evidence:
`crawl::tests::an_out_of_scope_url_is_asked_unless_another_locales_page`.

### D44 A root's scope is its path prefix until `include` names globs, which match the whole canonical URL
`include` replaces the prefix rather than narrowing it, so one root can span locales; `exclude` always wins. `*` and `?`
stop at `/`, so a version segment matches as `f2?`. Globs match the canonical form (D7), so a glob's `~` matches a URL's
`%7E` too, and a glob's `%7E` never matches. A root's `url` outside its own scope is refused at load. Rejected: globs
over the path alone, which cannot scope by host; regexes; `include` narrowing the prefix. Reopens: a root needing both
its prefix and an include.
Code: `config::Root::in_scope`. Evidence: `config::tests::scope_is_matched_on_the_canonical_form`.

### D50 A config `skip` list makes a matching page link or seed `unchecked`, reason `skipped`, before scope and the allowlist
Globs on the plain canonical form (D44's, the query kept), on any host, a root's scope included: the deployer said not
to ask. A hop onto one ends the chain, a sitemap's too, as a robots.txt-forbidden hop does; a root's `url` matching one
is refused at load. Why: the forge answers `issues/new` for any repo with a 303 to `/user/login` (I17), a false `ok`,
and skipping `/user/login*` marks every link sent there. Rejected: detecting a login from a 401 or a login-like
redirect, which misfires elsewhere; learning it from the first logins; `*` crossing `/` in a query, or the query
unmatched, which forks D44's globs.
Code: `crawl::frontier::Frontier::hop_plan`. Evidence: `crawl::tests::a_redirect_onto_a_skipped_url_ends_the_chain`.

### D67 Another locale is a prefix a profile's language-menu links name, and a URL no locale covers yet waits for the last page
A locale is where a link a declarative profile's `locale_links` selects (docs.fp.o's `a.language`, I25) departs from the
page's URL; no compiled profile reads a menu, a UI bundle's and not Antora's (D32). Why: locales only grow, so judging
what none covers yet once every page is read makes no verdict depend on which page came first, which comparing two runs
needs, a hop under `--externals=skip` aside (KD29). Rejected: config globs for locales; a locale told by its shape
(`pt_BR`), which takes `f28` for one; judging every URL when met, which a page read before any menu turns into a
request; every such URL waiting, 154,569 held. Reopens: a site naming locales otherwise (`hreflang`). Code:
`profile::locale_prefix`, `crawl::frontier::Frontier::judge_outside`. Evidence:
`crawl::frontier::tests::a_waiting_url_is_judged_by_the_locales_learned_since`.

### D68 An `allowlist` entry `*.<domain>` names every host below the domain, and none names a root's own host
`*.<domain>` is every host below it at any depth, not the domain itself; any other `*`, or a domain without a dot, is
refused at load. No entry names a root's host, one naming it outright refused at load: its URLs outside every scope are
asked already (D10), so it would only ask other locales' pages (maintainer, 2026-10-24). Why: the 2026-10-23 crawl has
3,521 `host_not_allowlisted` targets on 161 hosts below the guide's nine domains, and hosts come and go. Rejected:
testing the locale rule first, leaving such an entry loaded and inert; host globs, whose cost per link grows with the
list. Reopens: a deployer wanting other locales asked one hop deep rather than crawled.
Code: `config::Config::is_allowlisted`, `config::Config::check_allowlisted_roots`. Evidence:
`config::tests::a_roots_host_is_never_matched_by_a_domain_nor_allowlisted_outright`.
