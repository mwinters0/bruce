# Reports and the digest

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. What a run's rows are read back as: the stored report
views and `bruce report`, the run a bare report reads, and the digest, its comparison with an earlier run, its causes
and novelty marks, the severity a reader skims by, and how it groups pages and charges a doubled slash. `report` holds
them; `report::digest` the digest, `config::policy` the settings it reads.

### D6 Reports are SQL views stored in the DB and run by `bruce report <kind>`
A one-run report reads a `report_*` view, so a `duckdb` user sees what the CLI does; `diff`, `changed`,
`anchors-removed`, `introduced` and `inverse` compare two runs, so `bruce report` composes them. A view takes no
parameter, so each holds every run and `bruce report` names the run. They are installed at every open and into
`export-baseline`'s file (D21), `store` being below `report` (D39).
Rejected: a view per run; installing them only when `bruce report` runs, hiding them from `duckdb` after a crawl; a view
for `inverse`, its held set (D56) being I16's layout parsed in SQL; the text's notes (`stats`' totals, `unlisted`'s bare
roots, `introduced`'s other sections) in `--format json` before a consumer needs them.
Code: `report::views`, `cli::db::open`.

### D59 A bare `report` keeps the latest finished run, says of a later unfinished one, and refuses a baseline it imported
Without `--run` a report reads the latest finished run (D2), says on stderr that a later run stopped before the end, and
**refuses** where that run is a later unfinished run's `baseline_run` (D58 leaves such a run): the rows would be the
imported baseline's, read as the build's. `diff`'s default `to` does likewise.
Rejected: the latest run, finished or not, reading a stopped crawl's partial rows over the last whole run; a warning
alone, which a CI log buries under a report that looks whole.
Code: `report::run::one`. Evidence: `report::tests::the_default_refuses_the_baseline_of_a_later_unfinished_run`.

### D64 The digest is one document over a crawl run's crawled pages in every kind, its JSON holds every broken and error row, and its HTML is that document rendered with no script
Its sections are SQL-built, ranked tables under a `format_version` apart from the schema's (D43), bumped only for a
change a loader would misread; it counts every kind (D13) on crawled pages. The HTML shows nothing the JSON lacks, lists
only targets more than one link names; `--format` defaults to `both`.
Why: the CI database is truncated; a loader dating a breakage needs rows its runs no longer hold.
Rejected: aggregates alone; a flag on `bruce crawl`, which cannot digest an earlier run (D6); `serde_json` for a few
`format!`s (D60); ranking targets by pages, hiding the links one restored anchor fixes; a page that runs script, blank
from disk; "By repo" by the target's repo (maintainer, 2026-10-24).
Reopens: an every-locale digest too large to open (P9).
Code: `report::digest`. Evidence: `report::digest::tests`, `scripts/e2e.py`'s golden file.

### D73 A digest compares with the crawl run before it, a named run, or a baseline file imported for the digest and removed after
`--from N` or `--since FILE|URL` (`check-dir --baseline`'s file) names it, else it is the finished crawl run before. A
file's run is imported (D52) and deleted once the document is built, unless the database already held it. Both runs are
judged as D81 says. Maintainer, 2026-10-23: a deployment with no history compares with the baseline it published
the day before.
Why: left in place, the import, a finished run later than the digest's, is what a bare `report` reads (D59).
Rejected: attaching the file read-only, the views being bound to `main` (D52); comparing with the previous digest's
JSON, which holds no hash or anchor.
Reopens: a database too large to import into.
Code: `report::digest::compare`, `Store::discard_import`. Evidence:
`report::digest::compare::tests::a_baseline_file_is_imported_for_the_digest_and_removed_after`.

### D74 A new break is attributed by the first thing the compared run held that explains it, and a link is keyed by its page, target and fragment
A link broken now, not in the compared run, takes the first cause in `new_breakage` that holds, a target cause showing
the target's source as the compared run recorded it. Maintainer, 2026-10-23: attribution by what the comparison held,
never a hash.
Why: removal is tested before the move, a redirect onto a 404 being a page gone, and the move before the anchor, a page
that moved having lost the fragment by moving; `unexplained` is a rule judging differently since (a doubled slash).
Rejected: "the linking page changed", read off the hash, blaming an edit for a break it did not make; a cause per
reason, the reason column's; a row per key, hiding the links a maintainer edits.
Reopens: runs that differ, where `unexplained` is large or `target_moved` mostly removals.
Code: `report::digest::cause`. Evidence: `report::digest::cause::tests`.

### D75 A row is new where the compared run's same section lacked it, a broken link where section 3 lists it, and a bar marks it
Each section a reader acts on gets `new`, true where the compared run's same section lacks the row's key, a broken link
being new where `new_breakage` lists it; `pages` and `by_repo` count `new_links`, and with no comparison both are
`null`. The HTML marks a new row or drill-down with a bar at its left edge. Maintainer, 2026-10-25: new entries marked
in every section in as little room as possible.
Why: one rule, so a mark means the same everywhere and agrees with section 3.
Rejected: a "NEW" pill; a glyph by the dot, read as a severity; a row new whenever a link under it is, making a
long-broken page look new; HTML-only marks (D64); `false` with no comparison, claiming what is not known.
Reopens: a section whose key a change moves wholesale (a site moving host).
Code: `report::digest::novelty`. Evidence: `report::digest::novelty::tests`.

### D76 A link's severity is one of three levels read off its outcome and reason, and only the HTML shows it
*Broken* (`broken_url`, `error`: the reader gets nothing) is red; *attention* (`broken_anchor`, a non-pedantic redirect:
it lands, but not where it says) amber; *pedantic* (`pedantic_trailing_slash`) a grey ring; `unchecked` none. These are
defaults (D82). A dot marks a finding in a row naming a link or a target; pages and repos rank by red then amber.
Maintainer, 2026-10-24: the digest is skimmed by severity.
Why: the colour repeats the row's text, and the ring reads apart from the dots without colour and in print.
Rejected: a `severity` field in the JSON, restating outcome and reason; a column of words, read where a dot is seen;
error-only pages kept out of the pages, leaving a repo's red in links no listed page shows.
Reopens: a level of a kind D82 cannot name.
Code: `report::digest::html::severity`. Evidence: `report::digest::html::tests`.

### D80 The doubled slash's reach is the digest's, and it moves what is blamed on the linking page, not what is counted
`judge.doubled_slash_reach` is `full` (the doubled-slash pages a link lands on and its relative links reach),
`immediate` (the landing alone) or `off`; the digest's `through` reads the run's recorded value, and `link_verdicts` and
its `counted` do not.
Why: the broken relative links on a twinless doubled page are no page's own count (D71), so the reach only decides
whether the linking page is charged for them, `off` leaving them counted nowhere. A reach in the macro would move every
report and the gate.
Rejected: `off` counting those links on their own page, blaming it for what its linking page wrote (D71); a reach per
host, the attribution being the report's.
Reopens: a gate failing on `through`.
Code: `report::digest::sections::page_counts`, `config::policy::Reach`. Evidence: `report::digest::tests`.

### D82 A site maps each kind to one of D76's levels in `[severity]`, and the run's recorded policy states itself in the digest
`[severity]` maps an outcome, `redirected` or a reason to a level or `none`, a reason's beating its outcome's. The HTML
reads the map off the run's recorded rows (D78), never the file. A kind at `none` is no finding and the legend names it.
Housekeeping lists the recorded settings off their defaults, the JSON all of them as `policy`.
Why: a level changes no outcome, so needs no comparison rule; stating the map keeps a digest honest to a reader of the
page alone.
Rejected: levels a site defines, making legend, ranking and print styling data; ranking or dropping `none` rows in the
JSON, making the data the policy; a redirect at `none` listed undotted, or the legend silent on a kind at `none`
(maintainer, 2026-10-31).
Code: `config::policy::severity`, `report::digest::html::standing`. Evidence: `report::digest::html::tests`,
`config::policy::tests`.

### D86 The digest groups its pages by the fact `[digest]` `group_by` names, and by nothing unless a site says so
`group_by` is a full fact key (D37), recorded as one `digest.group_by` row (D78) and read from the policy in force
(D84). `by_group` holds `by_repo`'s columns per value of the fact, and no row where the run groups by nothing or by a
fact no page carries; the HTML has no section for it.
Why: the core named one generator's fact (D32); the key is the site's, and a core default of `antora.component` would
put Antora back in it.
Rejected: that default; checking the key against the profiles' facts at load, which the loader does not know (an unknown
key groups nothing); `by_component` beside `by_group`, two names for one table.
Reopens: a site grouping by two facts, or the HTML wanting the section.
Code: `config::policy::digest`, `report::digest::sections::by_group`. Evidence: `report::digest::tests`,
`config::policy::tests`.
