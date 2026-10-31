# Fetching and politeness

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. How a request is made and paced: TLS and DNS, per-host
rates and concurrency, robots.txt and `Crawl-delay`, and the asking settings (timeouts, retries, the body cap) a site
sets and a run records. `fetch::http` holds the client and `fetch::http::host` a host's pace and robots file.

### D45 TLS is rustls on ring and DNS is hickory's, so a failure arrives typed
reqwest's default `rustls` feature builds aws-lc-rs, a C build, against D23's one self-contained binary for the Fedora
CI image; `rustls-no-provider` plus ring avoids it. hickory makes NXDOMAIN (`broken_url`, D16) a `NetError` down the
error chain, where the system resolver gives a message string. Cost: about 30 crates, a resolver reading `resolv.conf`
itself, and a client that needs the system's CA bundle when built, even for an `http://` crawl (the README says so).
Rejected: aws-lc-rs; matching the system resolver's message. Reopens: ring failing to build on the CI image; a use case
crawling with no bundle.
Code: `fetch::http::build`, `fetch::http::classify`. Evidence: `fetch::http::tests::the_cause_is_found_down_the_chain`.

### D8 Rate limits, concurrency and robots.txt are per host; only a root's host may skip robots.txt
Defaults in `[default]`, a host's own in `[host]`; retries and `Retry-After` are D16's and D85's. robots.txt is always
obeyed on an external host, its switch refused there. Every run refetches every page (I3). The file follows RFC 9309: a
4xx allows all; a 5xx, a 429 outlasting the retries or `Retry-After` ceiling, or redirects leading nowhere close the
origin for the run; no response on its own name fails the origin's targets (D16); the file is an origin's (D69).
Rejected: one global rate, pacing the root at the slowest host's; RFC 9309 closing a silent host, hiding a dead one's
links as `unchecked`; asking a failed host again per link, a timeout a link where `error` fails no CI; `robots` on a
root, which two roots on a host can disagree on and an `include`'s other host lacks. Code: `fetch::http::host`.
Evidence: `fetch::http::host::tests`.

### D69 A robots.txt is an origin's; pace, concurrency and D8's switch stay the name's
RFC 9309, 2.3, scopes the file to scheme, host and port, so a name silent on one origin fails only that origin's links.
A redirect ending at another origin's `/robots.txt` takes that origin's read, so `http` and `https` cannot disagree by a
flicker; a silence fails the asking origin only on its own name, leaving it `robots_unavailable` on another's. The
slowest `Crawl-delay` among a name's origins paces it (D18). Rejected: a file per name, read off whichever origin was
asked first; a pace per origin, asking one server at twice the rate it set; a visited set per chain for two origins'
files redirecting to each other, missing two reads begun at once. Reopens: a site whose origins disagree at a cost.
Code: `fetch::http::host::Hosts::read`. Evidence:
`fetch::http::host::tests::a_name_silent_on_one_origin_fails_only_that_origin`.

### D18 A host's `Crawl-delay` is obeyed unless a root's host overrides it with `per_second`
Obeyed by default (D40). A `per_second` written for a host a root starts on beats `Crawl-delay`: a faster crawl is an
explicit override by whoever has permission. On any other host the slower wins, and the default rate never beats it. The
guide sets `robots = false` and `per_second = 1` on docs.fp.o (I2), D8's switch: its `Disallow`s, written for search
engines, hid nine live 404s. A deployer controlling the site writes a `robots.txt` group for its agent instead.
Rejected: a `crawl_delay` key; `robots = false` alone keeping `Crawl-delay`, reading the file it switches off.
Code: `fetch::http::host::interval`. Evidence: `config::tests::the_guide_turns_robots_off_for_the_root_host_alone`.

### D85 How a crawl asks is recorded with the policy and judged by nothing: a host's own settings over `[default]`'s, a body cap the site's
`connect_timeout`, `timeout`, `retries`, `max_retry_after` (D16) and `slash_probe` (D72) are keys of `[default]` and
`[host]`, a host's own winning; `max_body_mib` is top-level. Each is a `policy` row, defaults included (D78). Why: a run
cannot be asked again, so a trial policy (D84) holds the judging rows alone, and the digest reads how the run asked from
the stored `policy`. Rejected: one total timeout on the client, which cannot be a host's; a list of waits, where a count
of doubling waits is what a site tunes; a body cap per host, a page's size not being its host's. Reopens: a second
reader of the asking rows needing the trial's; a wait that is not a doubling.
Code: `config::policy::asking`, `store::Store::try_policy`. Evidence:
`report::digest::tests::a_trial_judges_the_digest_and_leaves_how_the_run_asked_as_it_recorded`.
