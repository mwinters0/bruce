# The crawl

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. One run over the live site: discovery from sitemaps and
links, the canonical form a URL is keyed by and the redirects a target follows, the slots the hosts share, and what a
stop or a second press leaves behind. `crawl` holds the frontier, the hops and the schedule; `url` the canonical form;
`cli::interrupt` the signals.

### D7 Canonical URLs keep the fragment apart; redirects are followed and stated
The canonical form decodes only unreserved escapes, a `%2F` never moving the path; keeps the query unless the root
strips it; and keeps `/foo` and `/foo/` distinct. Redirects are followed to 10 hops, a 2xx page's delay-0 refresh one of
them (I14), a loop or more being `broken_url`. The fragment is checked against the final page, and `redirected` never
fails (I4, I5), a final `/` alone pedantic (D70). `crawl` follows hops, not `fetch`: a refresh is found only by reading
the page. Rejected: normalising the trailing slash, which servers do not; a delayed refresh as a hop, its page having
content of its own.
Code: `url::canonical`, `crawl::hops`. Evidence: `url::tests`, `crawl::hops::tests`.

### D14 Discovery is sitemap and link-following both, and how a page was reached is read from the links
Each root's sitemap is discovered (D40) or set (docs.fp.o's is under its en-US root, I1). Both sources yield orphans
(listed, no inbound link) and unlisted pages (reachable, not listed). `report_pages` derives `listed` and `linked`,
`'direct'`, `'redirect'` or NULL, from `sitemap` links and `report_inbound`'s through `final_url`: a page a sitemap
entry reaches only by a redirect is listed, one linked only through a redirect no orphan. Each page records its root,
each run its roots, both canonical, so a root is its own target's `url`. Rejected: either source alone; flags written
by whichever target claimed the page, which a redirect's race decided; a redirecting entry not listing its landing,
one defect twice; matching a page to its root by scope in SQL, which `include` globs (D44) defeat.
Code: `report::views`. Evidence: `report::reach::tests`.

### D47 A run reads its sitemaps before any page, and each root links to its sitemaps
Every listed URL is then claimed before a link can name it, so a stopped run's cut (D48) counts every listed page it
lacks, and a listed page's plan is the sitemap's, never a link's, at the cost of the files' time (69 on docs.fp.o,
about a minute at its pace). A root links, `sitemap_file`, to
each `http(s)` top-level sitemap it names or found, an index to the files it lists, so a broken one shows as any broken
link and "no sitemap" is per root; a probe that misses writes nothing, nothing having named it. Rejected: reading
sitemaps beside the pages, which undercounts that cut; a target per failed probe; a `runs` column, per run and naming
no broken file. Code: `crawl::Shared::discover`. Evidence: `crawl::tests::each_root_links_once_to_each_sitemap_it_read`.

### D46 Where a redirect lands is a target of its own, written by whoever claims it first
A `final_url` then names a `targets` row, bar a stopped run's, so D28's fragment join finds the landing page's anchors.
The landing URL is claimed when the hops end; the first claim writes its target and page, and a loser keeps its own
answer from the chain. Rejected: holding the hops open on the landing URL's own task, which makes one target's outcome
wait on another's; a memo of settled targets, to save a repeat request to a landing URL already claimed. Reopens: that
repeat mattering at docs.fp.o's pace.
Code: `crawl::Shared::write`. Evidence: `crawl::tests::a_redirect_is_a_chain_and_its_landing_page_is_recorded_once`.

### D61 Live slots are shared among the hosts with work, and a build's reads have a budget of their own
A host starts a target, filed by its first URL's host, only while it runs fewer than the live slots divided by the hosts
with a target waiting or running, taken in turn, what it runs already finishing, so the forge's waiters for its
one-a-second pace leave docs.fp.o its share; a host alone has every slot. A build's reads run in a budget of their own,
so no waiter for production's pace (the prefix's host) holds a slot a read wants. Rejected: a fixed cap per host;
turn-taking alone; asking the fetcher whether a pace is free, a trait method; the build as a third lane of the live
slots; `available_parallelism`, differing by machine. Reopens: a host whose concurrency (D8) exceeds its share. Code:
`crawl::schedule`. Evidence: `crawl::schedule::tests::the_builds_reads_run_whatever_waits_on_the_prefix_host`.

### D48 A stop drops each target's hops 30 s after it; "cut" is every target found and not written
Every started target's hops end at one instant, the stop plus `STOP_GRACE`, and one whose hops ended still writes whole,
so a stopped run holds whole targets and its counts are exact. "Cut" is claimed less written, the queued included: a
lower bound on what the run lacks, unfound URLs not in it. Rejected: aborting the tasks in `drive`, which cannot reach a
write already on the blocking pool, its rows landing after `drive` returns; counting only targets started and dropped,
which says how the stop went, not what the run lacks.
Code: `crawl::Shared::in_grace`, `crawl::Summary::cut`. Evidence:
`crawl::tests::a_target_unfinished_at_the_grace_is_cut`.

### D49 An abandoned run waits for the writes it has begun, and only for them
The second press closes the run's `Writes`: a write not yet begun sends nothing, and the close waits, unbounded, for
each begun to send its last row. Rejected: dropping the runtime, whose wait covers all blocking work (an extraction
nobody reads, whatever a dependency adds, as a resolver off hickory running `getaddrinfo`) and whose skipping of queued
work tokio does not guarantee; a bound on the wait, which can only end it mid-target; a third press, a wedged DuckDB
being a bug to fix.
Code: `crawl::Writes`, `cli::interrupt::drive`. Evidence:
`crawl::tests::an_abandoned_run_holds_the_page_it_was_writing_whole`.

### D58 SIGINT and SIGTERM are each a press, and a stopped `check-dir` judges nothing
The first signal of either kind stops `crawl` or `check-dir` (D48), the second abandons it (D49), and the exit is 128
plus the first's number (130, 143). A stopped `check-dir` runs no gate, says so on stderr and leaves its run unfinished
(D2): a gate over a cut build reads what was never written as absent. GitHub's SIGTERM comes 7.5 s after its SIGINT and
its kill 2.5 s later (I24), inside the 30 s grace, so the second press must abandon. Rejected: one flat 130, hiding from
a runner's log which signal ended the run; judging what was written.
Code: `cli::interrupt`, `cli::check_dir::Checked`. Evidence: `tests/signals.rs`.
