# CI mode

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. Checking a local build against a published baseline: the
single-run baseline file, how a built page's links resolve (locally, through the baseline, then live), what the build is
taken to hold, the gate that fails only on introduced breakage, and the inverse check. `cli::check_dir` runs it and `ci`
maps the build to URLs; `crawl::baseline` and `crawl::xref` answer the links; `report::gate` and `report::inverse` judge
them.

### D4 CI resolves targets outside its build against a published baseline, then live
A scheduled full crawl publishes a baseline holding each page's anchor set; CI answers from it what its build lacks,
else asks live, and runs the inverse check too (D56). Rejected: live-only, slow and non-deterministic; skipping
cross-component links, the breakage this tool exists for.

### D9 CI fails only on breakage the change introduced
A link is `(source page, target URL, fragment)` of any kind (D13), the build mapped to production URLs, and only built
pages' links count; a baseline link's target may differ by a final `/` (D101). It is introduced where it fails here and
the baseline lacks its key at a failing outcome (D83). The inverse check fails too unless `--inverse=warn`. `report
introduced` and `report inverse` judge every kind as well, so re-reporting a CI run gives the rows that set its exit
under `--inverse=fail`. Accepted noise: a renamed page's links all look new. Rejected: failing on all breakage, which
keeps CI permanently red; `check-dir --kind` narrowing the gate, D50's `skip` silencing a noisy URL instead. Code:
`report::gate`. Evidence: `report::gate::tests::the_gate_reads_every_kind_unless_a_report_narrows_it`.

### D21 CI consumes a single-run baseline file, not the history DB
`bruce export-baseline` writes one finished run into a fresh DB of the current schema, so the views, reports and D43's
refusal read it unchanged and the gate compares like with like (D28). Where it is hosted is the deployment's (D41). Its
verdicts stand whatever their age; the gate prints the crawl's date. Rejected: shipping the history DB to CI; a slim
schema of its own, a second version and set of queries; a `--max-age` refusal, failing a PR for a reason its author
cannot fix, until one is asked for. Code: `store::baseline`.

### D22 A local build's links resolve locally, then against the baseline, then by allowlist
A built URL is checked locally (`foo/` is `foo/index.html`, bar a top `index.html` that is a meta refresh, served at
its own path alone so the prefix is not built, D101); an in-scope URL the build lacks is `broken_url` in a
component-version the build holds, else the baseline's, else asked live (D54); an unresolved xref outside what the
build holds resolves through the baseline's pages (D19, D55). Externals default to the baseline's verdict
(`--externals`), so a slow host never blocks a PR. Rejected: a held-set exception in `crawl::baseline` for the refresh;
the baseline's answer for a held page, which passes a link to a page the change deletes; `--externals` governing an
in-scope URL the build lacks, a URL on the site new to the gate being the one most worth asking. Code: `ci::Build`,
`crawl::baseline`.

### D101 The gate's key matches a baseline link whose target differs only by a final `/`
A CI link matches a baseline link from the same page and fragment whose target is the same or differs by its path's
final `/`, in every section, and hides only where the baseline's was in the same state: a `/foo` broken where `/foo/`
answered is introduced. Why: production's `site.url` has no final `/` and a partial build, having none (I11), links
`.../en-US/`; both redirect, and 236 of a quick-docs run's 239 redirected rows were the home link. Rejected: the toggle
in `redirected` alone, leaving the home link introduced under a `max_redirects` that breaks it; matching by landing,
which hides an edit between two outdated hrefs; the guide telling a repo to expect the rows or to build with `site.url`
(I15, D53). Code: `report::gate`. Evidence:
`report::gate::tests::a_link_matches_the_baselines_with_its_final_slash_toggled_d101`.

### D53 In a local build, an href beginning `/` resolves against the prefix
The build directory's root is the site's root, as a static server serving it at `/` shows it, so such an href (not
`//`), and a root-relative `<base href>`, join onto `--prefix`. Why: a partial build has no `site.url` (I11), so the UI
footer's `/legal/privacy/` against the production origin is a 404 on every built page, a break the tool made. Rejected:
the production origin, the guide telling deployers to `skip` those URLs. Reopens: a build whose root-relative hrefs are
meant for the origin. Code: `extract::links::Links::resolve`. Evidence:
`extract::tests::in_a_local_build_an_href_beginning_a_slash_resolves_against_the_site_root_d53`.

### D54 A URL the build lacks waits for the held set, and a copy from the baseline waits for the end
An in-scope URL under the prefix and not built is judged once every built page is read, the held set (I16) final only
then: in a held component-version it is `broken_url`, reason `not_built`; else it is answered as a URL outside the build
is, by the baseline's rows if held, else a request. Copies go last, so a copied landing page never doubles a built one,
`targets` having no key. `--externals=skip` asks nothing of an allowlisted URL, or one on a root's host outside every
scope and locale: a link gets the baseline's rows where held, a hop never, else `unchecked`, `externals_skipped`;
in-scope URLs are not reached. Rejected: a pre-scan of the build (D38); `skip` as no verdict, passing a new link to a
URL the baseline has broken. Reopens: a prefix below the site root. Code: `crawl::baseline`, `crawl::frontier`.
Evidence: `crawl::baseline::tests`.

### D55 An xref waits for the held set, resolves through the baseline's pages, and keeps its refspec
A refspec whose component-version the build holds stays `unresolved_xref`, Antora having had that catalog whole. Else,
once the held set is final (D54), Resolve maps it through the baseline's pages to that page with the refspec's fragment,
`xref_refspec` kept, so `link_verdicts` still reads it unresolved where it points at its own page or its fragment is the
refspec (I6). No version: the one served off its own segment (I5), else Antora's order (I20). Rejected: a column, or a
cleared refspec, marking a resolved link: a schema bump (D43); a config key of prerelease versions; `rawhide` by name
(D32). Reopens: Antora marking an unresolved xref other than as `#<refspec>` (I6). Code: `crawl::xref`,
`profile::antora::resolve`. Evidence:
`crawl::baseline::tests::an_xref_into_a_component_the_build_lacks_resolves_through_the_baseline`.

### D56 The inverse check reads the held component-versions back from the CI run, and counts the links of pages outside them
A built page's `antora.*` facts and URL give the held prefixes (I16), so `report inverse --run` needs only the DB. The
linking pages are the baseline's crawled pages in no held prefix and not built; a removed file is a baseline file in a
held prefix with no `local` `ok` target, every build file being a seed (`ci`). Rejected: every page not built, a deleted
page failing on links gone too (what this misses is KD5's); `Held` moved where `crawl` and `report` both reach it, a
`crawl`-to-`ci` edge against D39; storing the held set in the run, a schema bump (D43); touched pages by `source_repo`,
CI's pages carrying no edit link (I15); seeding the baseline's targets under the held prefixes, unknown until the built
pages are read. Reopens: I16 changing, its layout read in three places (D57). Code: `report::inverse`. Evidence:
`report::inverse::tests`.

### D57 An xref no page's facts find is the URL its refspec names, answered as any URL the build lacks
Antora resolves a refspec through a page-alias, and to either of two files sharing an output URL; the baseline holds
facts once per URL, and an alias only if crawled. So where Resolve's facts find no page it names the URL by I16's
layout, its prefix read off a baseline page of the component-version (I5), and D22's chain answers it. A URL answering
`broken_url` makes the link `unresolved_xref`. Rejected: the baseline's redirect targets alone, missing an uncrawled
alias; `newest` skipping candidates that give no prefix, which hides the bare-version case; any outcome but `ok` as
unresolved, failing a PR on a timeout its author cannot fix (D16) and judging a derived URL more harshly than a found
page (D55). Code: `profile::antora::resolve`. Evidence:
`crawl::baseline::tests::an_xref_no_page_has_the_coordinates_of_resolves_by_the_url_it_names`.

### D83 The gate fails on the outcomes the CI run's `gate.fail` names, recorded as one row
`[gate]` `fail` defaults to `broken_url` and `broken_anchor`, so `error` fails only where named (D16). The CI run's row
is read (D81). It is one comma-joined row (D78); an empty list is report only. An outcome left off is listed, not
failing, inverse rows too. Why: a site keeps a finding in every report and stops it failing CI, which reclassifying
would not; one row lets the digest's Housekeeping state a list that lost a default element, where a row per element
differs in nothing and an empty list has none. Rejected: the left-off outcome in no gate section, leaving a CI author
nothing to read; an empty list refused, a rollout then being an ignored exit status that hides status 2 too. Reopens: a
site wanting the left-off section to hold a link the baseline had failing. Code: `config::policy::gate`, `report::gate`.
Evidence: `report::gate::tests`.
