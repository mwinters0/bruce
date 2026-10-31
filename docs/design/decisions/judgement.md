# Outcomes and the recorded policy

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. How a link is judged: the five outcomes and their
reasons, the URL shapes that are broken or pedantic whatever they answered, and the policy a run records and every read
judges it by, a comparison and a trial included. `crawl::outcome` sets what the crawl stores, `config::policy` the rows,
and `store::views` the verdict SQL.

### D16 Five outcomes; `error` is separate from broken and fails CI only where a site's `fail` list names it
`ok`, `broken_url` (a 4xx but 401, 403 and 429, NXDOMAIN, a redirect loop, a 3xx leading nowhere, D66, D71, D72),
`broken_anchor`, `error` (timeouts, TLS, a 5xx after the retries, 401/403/429 with the status kept, a `Retry-After` past
the ceiling at once), `unchecked`. A 2xx body over the cap is `ok`, reason `too_large`, and not extracted: half a page's
anchors would give false `broken_anchor`s. `gate.fail` leaves `error` out by default (D83); the limits are a site's
(D85), a code's outcome its `status` map's (D79). Rejected: `error` folded into `broken_url`, which fails CI on a flaky
proxy; `too_large` as `error`, for a URL that works, or as a sixth outcome, which every report and the gate would have
to place. Code: `crawl::outcome`, `fetch::http`. Evidence: `crawl::outcome::tests`.

### D20 An unresolved xref is a broken page reference, never a same-page fragment
Live crawl: `broken_url`, reason `unresolved_xref`, raw refspec kept, with a "did you mean" hint from a fuzzy match over
resource IDs, run only on failures. CI: resolved via D19 first (I6). Rejected: checking `#comp::page.adoc` against the
page's own ids, which is what a naive checker does. The hint's measure and floor are D65.

### D66 A scheme a public web page uses is `unchecked`; any other is `broken_url`, reason `bad_scheme`
The listed schemes take a reader somewhere no request follows (`mailto`, `irc`, `ftp`, …) or work in the page (`data`,
`javascript`): `unchecked`, reason `scheme`. Any other but `http(s)`, a `file:` or a mistyped `hhttps:`, no reader can
follow: broken in every report and the gate (both maintainer, 2026-10-24). `judge.schemes` extends the list (D79). Rejected:
every other scheme `unchecked`, which hid the 2026-10-23 crawl's 2 such links among its 280; `scheme` under both
outcomes, one name for two verdicts. Reopens: a listed scheme found broken, or an unlisted one working. Code:
`config::WORKING_SCHEMES`, `crawl::frontier`. Evidence:
`crawl::frontier::tests::a_scheme_a_public_web_page_uses_is_unchecked_and_any_other_broken`,
`crawl::tests::a_scheme_no_reader_can_follow_is_reported_broken_and_a_working_one_unchecked`.

### D70 A redirect that only adds or drops a final `/` is `ok`, reason `pedantic_trailing_slash`
Whatever the hops between; a missing fragment still makes the link `broken_anchor`. Reports list the pedantic rows last,
and the gate fails on none, as on no redirect (D7). Maintainer, 2026-10-24: a slash is to be corrected, below a 404 or a
missing fragment; the class is named for one form so that others (a scheme upgrade, a moved page) can stand beside it.
Rejected: a sixth outcome (D16) for a link a reader follows; `redirected` alone, which hid the 2026-10-24 crawl's 496
such links among its redirects. Reopens: a second pedantic class, making the reason a list or the view a table.
`judge.pedantic_trailing_slash = false` turns it off (D77). Code: `store::views`. Evidence:
`store::views::tests::a_redirect_that_only_adds_or_drops_a_final_slash_is_ok_and_pedantic`.

### D71 A doubled slash in a URL's path is the linking page's fault: the link is `broken_url`, reason `doubled_slash`
Whatever the target answered, asked or not (`skip`, robots, off the allowlist), and for a link redirected to one. Why:
a linking page writes Fedora's doubled slashes, and the target cannot fix the 404s beyond (2026-10-24 crawl: 562 behind
one link of `en-US/project/`). Rejected (maintainer, 2026-10-24): pedantic or amber, the target's relative links
breaking for the reader; an unasked URL left `unchecked`, the URL being the fault; `doubled_slash` on a page's link to
itself, which works for its reader; the redirecting server at fault, a broken experience being reported whatever breaks
it; every scheme's path read (`ftp://host//etc` is absolute). Reopens: a site doubling one on purpose (`doubled_slash =
false` per host, D79). Code: `store::views`. Evidence:
`store::views::tests::a_link_to_a_url_with_a_doubled_slash_is_broken_whatever_it_answered`.

### D88 A page at a doubled-slash URL counts only what it owns; what the slash breaks is the linking page's `through`
With a twin (its URL, slashes collapsed) among the run's pages it counts no link; without, none of its broken or errored
relative links, which the doubled path breaks. Those are the linking page's `through`: each doubled page its
doubled-slash links reach, once, bar a link broken on the twin too (same `raw_href` and kind), ranked with its red.
Rejected (maintainer, 2026-10-24): `through` per link reaching a page, multiplying breakage one fix removes; a
`broken_anchor` there amber, the slash being the fault; the twin's copy matched by resolved target, the fix leaving the
reader on the twin's HTML (7 of 536 on the 2026-10-24 crawl), or by outcome. Reach: D80. Code: `link_verdicts.counted`,
`report::digest::sections::PAGE_COUNTS`. Evidence:
`report::digest::tests::a_link_broken_on_the_twin_too_is_not_broken_through_it_and_a_query_link_is`.

### D72 A broken 4xx whose URL with its final `/` toggled answers is `broken_url`, reason `wrong_trailing_slash`
The toggle is asked once, for its status alone, where `crawl::slash::toggled` allows; it is no target, never claimed or
followed, and the digest names it as the fix. Maintainer, 2026-10-24: a slash the server does not supply is still
broken; a redirect onto the 404 has its landing toggled. Why ask: no `broken_url` target of the 2026-10-24 crawl had its
toggle among the targets, and on 2026-10-25's the probes cost about 11 s of a 75-minute long pole. Rejected: a target
row or a column for the fix; a 3xx as an answer (maintainer: a toggle that moves is no fix); a redirected target's own
toggle, the landing's reason being every URL's that lands there. Reopens: the probes on a long pole; `slash_probe`
(D85). Code: `crawl::slash`. Evidence: `crawl::tests::a_404_whose_url_with_its_slash_toggled_answers_says_so`,
`runs/d72-toggle-probe-20261024/`.

### D77 Links are judged by a table macro reading the recorded policy, and `link_verdicts` is its call with each link's own run's
`link_verdicts_with(policy_run := NULL)` judges by the policy (D78) of `policy_run`, else D81's `judged_by`, else the
link's own run; a setting with no row is at its default. `link_verdicts` calls it bare, a view taking no parameter (D6).
Why: one copy of the judgement serves a run under its own policy and under another's, and the macro is inlined: on the
2026-10-28 crawl a run's verdicts cost 3 to 8 percent more, the digest 5 to 10. Rejected: a second copy of the SQL for
comparisons; a view per setting; a scalar parameter per setting, which a host's `doubled_slash` or a status map cannot
be. Reopens: a setting a stored fact cannot judge, which the crawl must keep (P7's rule), or the digest's cost mattering
on a larger site. Code: `store::views::link_verdicts_with`. Evidence: `runs/p7.1-verdicts/`, `runs/p7.3-verdicts/`
(reports byte-identical).

### D78 A run records the policy it was judged by, whole, as rows of a `policy` table
`policy(run_id, host, key, value)`, `host` NULL for the run-wide value; every setting is written, the defaults included,
and the rows travel with the run (D21, D2). Why: a report reads a run, never the file (P7), the macro joins a host's
value to the default, and a run's judging must outlive a later default. Rejected: a `runs` column or a struct fixing the
keys, each new setting bumping the schema; JSON, a feature `Cargo.toml` does not enable, giving the macro no join; only
values differing from the default, which a later default would silently change. Reopens: a setting that is not text in a
row. Code: `config::Policy::entries`, `store::Store::begin_run`. Evidence:
`store::tests::a_run_records_its_whole_policy_beside_itself`,
`store::baseline::tests::a_run_policy_travels_in_the_baseline_file_and_into_the_run_that_imports_it`.

### D79 A status is judged by the host that answered, a URL's shape by the host linked, and a scheme by the URL
Read over what the crawl stored (D77): a code by the answering host's `status` map over the default; `doubled_slash` by
the linked URL's host; a scheme by the policy's list. Why: a redirect onto a forge's 403 is the forge's answer; a
doubled slash is the linked URL's or its host's redirect's, as on every landing doubling one on the 2026-10-28 crawl.
Rejected: the linked host for a status; the landing's host for a doubled slash, switching off its own doubled URLs; a
host's map replacing the default, not laid over it per code (maintainer, 2026-10-30); the crawl storing the policy's
answer, a trial then needing a crawl; a class such as `4xx`, ambiguous beside single codes. Reopens: a host's code and a
landing's disagreeing. Code: `store::views`. Evidence:
`store::views::tests::a_status_the_policy_judges_is_read_over_the_outcome_the_crawl_stored`.

### D81 A comparison judges both runs by the newer run's policy, through a session variable the macro reads
`store::JudgedBy` sets the connection's `judged_by` for a whole read: the gate and `report inverse` at the CI run,
`diff` at `--to`, a digest with a comparison at its own run (KD24). A policy change then shows as no breakage. Why: the
views take no parameter (D6), and one variable reaches every section a comparison re-queries, with no copy of any query.
Rejected: `policy_run` named in each comparison's SQL, a parameterised copy of every section; a table of overrides, a
write in a read; each run by its own policy, the change named a cause (D74), failing a gate on a setting nobody broke.
Reopens: readers sharing a connection, or a comparison wanting the older run's policy. Code: `store::JudgedBy`.
Evidence: `report::gate::tests::the_baseline_is_judged_by_the_ci_runs_policy`,
`report::diff::tests::both_runs_are_judged_by_the_newer_runs_policy`.

### D84 A trial policy is a temporary table of the policy rows every reader asks for, one set per run
`report --config FILE` puts the file's judging rows, once per run, in a temporary `policy_in_force`, shadowing the view
of that name that every reader of the policy asks for; the asking rows stay the run's (D85). Why: the reports hold one
connection, so one shadowed name reaches every reader, copying no query and writing nothing. Rejected: rows under a
scratch run, a write in a read that a crash leaves and a later import collides with; a parameter on every report (D81);
shadowing `policy` itself, which the import's `main.policy` also finds. Reopens: a judging reader bypassing the view, or
a report holding a second connection. Code: `store::Store::try_policy`. Evidence:
`store::views::tests::a_trial_policy_judges_every_run_and_writes_nothing_to_the_database`,
`cli::report::tests::a_report_judged_by_a_trial_config_leaves_the_database_as_it_was`.
