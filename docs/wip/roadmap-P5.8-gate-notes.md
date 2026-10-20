# P5.8 notes: the gate

Outcome: `bruce check-dir` judges the finished run against its baseline and
exits 0, 1 (a link is introduced) or 2 (the tool could not run: every error
`check` returns). `bruce report introduced --run N` is the same judgement on a
stored run. The code is `report/gate.rs`: `report::gate` gives a `Gate`, whose
sections `check-dir` prints as text or as one JSON object, and
`Report::Introduced` gives the introduced rows as a table with the rest in
notes. The manual is `check-dir.md` and `report.md`'s `introduced`.

## What the next slices inherit

- **P5.9's inverse section is a field on `Gate`** and a line in
  `Gate::text`/`json`, between the introduced breakage and the redirected
  links (the spec's order). `Gate::fails` is where an inverse breakage fails
  the exit status, and `check_dir::run` is where `--inverse=warn` is read;
  `check-dir` takes no `--inverse` yet. `report inverse` needs its own
  `Report` arm beside `Introduced` in `query::run`.
- **`report::links`** holds `broken`'s and `errors`' query, moved out of
  `query` so `gate` can list the errors without a cycle (`module_graph.py`).
- **`report_redirects` gained `fragment`**, the gate's key. `redirects`
  selects its columns by name, so its output is unchanged.
- **The key is `diff`'s**: page `url_id`, target URL, fragment (`''` for
  none), one row per key. A resolved xref's target is the baseline's page, so
  the baseline, which has the link unresolved on its own page, reads it as
  `new` (P5.7's note); `raw_href` still shows it was an xref.
- **Only built pages' links are read, by construction.** A copied page has no
  `links` (D52), so the gate adds no filter on `answered_by`.
- **The text order is the spec's**: baseline line, introduced breakage,
  redirected, errors, breakage already in the baseline. `check-dir` prints
  `run N: T targets, P pages` before it, on stderr under `--format json`.
- **A key with several links in the baseline reads as its least certain
  outcome** (`error`, then `unchecked`, then the rest), for the `baseline`
  column only.

## Limits met, not removed

- **Errors are listed in full.** The spec says an `error` is listed; a link to
  a host that refuses every request (a 403 or 429 copied from the baseline)
  repeats on every page that has it. P5.11's run on quick-docs should say
  whether the section is noise; the count alone is a one-line change in
  `Gate::rest`.
- **`baseline_date` is the baseline run's day in the session's timezone**,
  cast from a `TIMESTAMPTZ`.
- **Every `Err` is exit 2**, a crawl that dies midway and a writer error
  included: the run is left unfinished, and the gate does not run.

## Negative results

- **A `gate` call from `query` and `links` from `gate` is a cycle.** Hence the
  `links` module.
- **`started_at::DATE` does not cast** in DuckDB (`TIMESTAMP WITH TIME ZONE`
  to `DATE` is unimplemented); `::TIMESTAMP::DATE` does.
