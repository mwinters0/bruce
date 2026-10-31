# Decisions

The decisions the code cannot explain: shapes kept for a deliverable not yet built, defaults chosen over an alternative,
and obvious changes argued and refused. Nothing here says how the code works (the named module does); external facts are
cited by `I<n>` from `site-invariants.md`. Each entry lives in its topic's file under `decisions/`, which opens with
what the topic is and is read on its own; this file is the index and the citation target: cite as
`docs/design/decisions.md`, "D12". The rules, the caps and the meter included, are `docs/process.md`, "The decision
register".

D25 and D29 are spent: D25 was superseded by D30, and D29 became `roadmap.md`'s phase index. The en-US crawls the
entries quote are named by their day (2026-10-23, 2026-10-24, 2026-10-25, run 2 of a copy of 2026-10-24's database, and
2026-10-28). Their databases predate schema 6 and are no longer kept: a figure stands as recorded, and a new crawl
remeasures it.

<!-- decision-watermark: D94 -->

## Modes, commands and deployment
[`decisions/modes.md`](decisions/modes.md)
- **D1** Two fetchers behind one trait: live HTTP crawl first, local build directory second
- **D27** One binary, `bruce`, with subcommands `crawl`, `check-dir`, `report`, `export-baseline`, `prune`
- **D30** The name is `bruce`, after the Bruce claw anchor
- **D40** A run needs no config: the URL given is the root, and every other setting has a default
- **D41** This repo builds the tool and never deploys it
- **D42** The simulated deployment is the manual's deployment guide, executed

## Configuration and scope
[`decisions/config.md`](decisions/config.md)
- **D5** Configuration is optional TOML, and no flag overrides a key; roots are separate from the allowlist
- **D3** Allowlisted external hosts are checked one hop deep and never crawled
- **D10** Default scope: en-US, every fedora version; locales are a root that is off by default
- **D44** A root's scope is its path prefix until `include` names globs, which match the whole canonical URL
- **D50** A config `skip` list makes a matching page link or seed `unchecked`, reason `skipped`, before scope and the allowlist
- **D67** Another locale is a prefix the roots' pages' language menus name, and a URL no locale covers yet waits for the last page
- **D68** An `allowlist` entry `*.<domain>` names every host below the domain, and none names a root's own host

## The crawl
[`decisions/crawl.md`](decisions/crawl.md)
- **D7** Canonical URLs keep the fragment apart; redirects are followed and stated
- **D14** Discovery is sitemap and link-following both, and each page records which
- **D47** A run reads its sitemaps before any page, and each root links to its sitemaps
- **D46** Where a redirect lands is a target of its own, written by whoever claims it first
- **D61** Live slots are shared among the hosts with work, and a build's reads have a budget of their own
- **D48** A stop drops each target's hops 30 s after it; "cut" is every target found and not written
- **D49** An abandoned run waits for the writes it has begun, and only for them
- **D58** SIGINT and SIGTERM are each a press, and a stopped `check-dir` judges nothing

## Fetching and politeness
[`decisions/fetch.md`](decisions/fetch.md)
- **D45** TLS is rustls on ring and DNS is hickory's, so a failure arrives typed
- **D8** Rate limits, concurrency and robots.txt are per host; only a root's host may skip robots.txt
- **D69** A robots.txt is an origin's; pace, concurrency and D8's switch stay the name's
- **D18** A host's `Crawl-delay` is obeyed unless a root's host overrides it with `per_second`
- **D85** How a crawl asks is recorded with the policy and judged by nothing: a host's own settings over `[default]`'s, a body cap the site's

## Extraction
[`decisions/extract.md`](decisions/extract.md)
- **D13** Every reference kind is extracted, each target fetched once per run
- **D12** A fragment is valid where a browser would scroll to it
- **D34** An anchor's placement is `content`, `chrome` or `unknown`; the profile draws the line
- **D15** Pages store metadata, a content hash and their profile's facts; bodies only on request

## Profiles
[`decisions/profiles.md`](decisions/profiles.md)
- **D32** Two extension layers: generator profiles in code, the deployment in config
- **D33** Profiles are compiled in, or declared in config as a `[[profile]]` array
- **D35** A root's profile defaults to `auto`, which picks one per page by its generator meta
- **D36** The page → source tie-back is core: an edit-link heuristic and forge URL shapes
- **D93** A profile's source rules are tried in file order before an edit link is parsed, each a regex with templates over its captures
- **D94** A declarative profile may extend one compiled profile, adding its rules to all the base does
- **D17** Each page stores its source file and repo, read from its edit link
- **D37** Profile facts live in one core table, `page_facts`, under namespaced keys
- **D38** A profile has four hooks: detect, extract, resolve, views
- **D19** Pages carry Antora resource IDs, and unresolved refspecs resolve through them
- **D65** An unresolved xref's hint is the nearest page ID by edit distance, within three tenths of the longer, read in the page's own context
- **D87** The reports read a profile's generator knowledge through the profile: `link_hints` for a hint, `sitemap_note` for the sitemap's sentence

## Storage and history
[`decisions/store.md`](decisions/store.md)
- **D2** Runs are append-only: every row but `meta`'s carries a `run_id`, and only a run's `finished_at` is updated
- **D26** A URL's identity is a u64 hash of its canonical form, stable across runs
- **D43** A DB records its schema version, and `bruce` refuses a DB of another version
- **D28** Fragment verdicts are computed in SQL after the crawl, not during it
- **D51** A target records where its answer came from, in a column the writer fills and no view reads
- **D52** A baseline file is imported as a run of its own, and a target is copied between runs row by row

## Outcomes and the recorded policy
[`decisions/judgement.md`](decisions/judgement.md)
- **D16** Five outcomes; `error` is separate from broken and fails CI only where a site's `fail` list names it
- **D20** An unresolved xref is a broken page reference, never a same-page fragment
- **D66** A scheme a public web page uses is `unchecked`; any other is `broken_url`, reason `bad_scheme`
- **D70** A redirect that only adds or drops a final `/` is `ok`, reason `pedantic_trailing_slash`
- **D71** A doubled slash in a URL's path is the linking page's fault: the link is `broken_url`, reason `doubled_slash`
- **D88** A page at a doubled-slash URL counts only what it owns; what the slash breaks is the linking page's `through`
- **D72** A broken 4xx whose URL with its final `/` toggled answers is `broken_url`, reason `wrong_trailing_slash`
- **D77** Links are judged by a table macro reading the recorded policy, and `link_verdicts` is its call with each link's own run's
- **D78** A run records the policy it was judged by, whole, as rows of a `policy` table
- **D79** A status is judged by the host that answered, a URL's shape by the host linked, and a scheme by the URL
- **D81** A comparison judges both runs by the policy of the run it reports on, through a session variable the macro reads
- **D84** A trial policy is a temporary table of the policy rows every reader asks for, one set per run
- **D90** A constant becomes a site's setting where a site would rightly choose otherwise and the choice is meaning

## CI mode
[`decisions/ci.md`](decisions/ci.md)
- **D4** CI resolves targets outside its build against a published baseline, then live
- **D9** CI fails only on breakage the change introduced
- **D21** CI consumes a single-run baseline file, not the history DB
- **D22** A local build's links resolve locally, then against the baseline, then by allowlist
- **D53** In a local build, an href beginning `/` resolves against the prefix
- **D54** A URL the build lacks waits for the held set, and a copy from the baseline waits for the end
- **D55** An xref waits for the held set, resolves through the baseline's pages, and keeps its refspec
- **D56** The inverse check reads the held component-versions back from the CI run, and counts the links of pages outside them
- **D57** An xref no page's facts find is the URL its refspec names, answered as any URL the build lacks
- **D83** The gate fails on the outcomes the CI run's `gate.fail` names, recorded as one row

## Reports and the digest
[`decisions/reports.md`](decisions/reports.md)
- **D6** Reports are SQL views stored in the DB and run by `bruce report <kind>`
- **D59** A bare `report` keeps the latest finished run, says of a later unfinished one, and refuses a baseline it imported
- **D64** The digest is one document over a crawl run's crawled pages in every kind, its JSON holds every broken and error row, and its HTML is that document rendered with no script
- **D73** A digest compares with the crawl run before it, a named run, or a baseline file imported for the digest and removed after
- **D74** A new break is attributed by the first thing the compared run held that explains it, and a link is keyed by its page, target and fragment
- **D75** A row is new where the compared run's same section lacked it, a broken link where section 3 lists it, and a bar marks it
- **D76** A link's severity is one of three levels read off its outcome and reason, and only the HTML shows it
- **D80** The doubled slash's reach is the digest's, and it moves what is blamed on the linking page, not what is counted
- **D82** A site maps each kind to one of D76's levels in `[severity]`, and the map ranks as well as dots
- **D86** The digest groups its pages by the fact `[digest]` `group_by` names, and by nothing unless a site says so
- **D91** The digest states the policy it was judged by: Housekeeping what differs from the defaults, the JSON all of it, the guide what is judged
- **D92** A kind at `none` is no finding in the digest, and its JSON and figures keep it

## Build, layering and release
[`decisions/build.md`](decisions/build.md)
- **D31** One crate, `bruce`; its concepts are separated by top-level module, not by crate
- **D39** The layering: `cli` › `crawl`, `ci`, `report` › `fetch`, `extract`, `store` › `profile` › `forge`, `config` › `url`
- **D11** Our own stack, not lychee-lib; lol_html is the tokenizer
- **D24** Tests run on saved real pages, a mock HTTP server, and an optional Antora build
- **D23** DuckDB is bundled; distribution is a release binary on GitHub
- **D60** `bruce` is `MIT OR Apache-2.0`, and a release carries every notice its binary owes, made offline
- **D62** A release is tried as `-rc.N` prereleases, since a published release is immutable
- **D63** The release workflow publishes the draft its create call returned, by id
- **D89** A release's notes are its annotated tag's message, drafted by `/release` from a template
