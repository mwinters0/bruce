# P7 — Site policy in config

What P7 does and why, settled in the grilling of 2026-10-25 and 26. The
decisions are in [`../design/decisions.md`](../design/decisions.md) and are
cited here, never restated. Its inbox, the maintainer's rulings from the
2026-10-24 digest review on severity (D76) and the doubled slash (D71), is
folded in below and deleted.

## Intent

A site sets the policy `bruce` judges and reports it by, without Rust: what
an outcome or reason is, whether a reason applies, and how loudly the digest
shows each kind. Today every such choice is a constant (below, "Facts the
grilling found"), each made for docs.fp.o and defensible for most sites, not
for every one.

## P7 is policy; extraction rules are P10

Settled 2026-10-25 (maintainer). **P7 is site policy without Rust**, its user
Fedora's own digest and gate today. **The sketch's declarative extraction
rules** (a selector and attribute giving a fact, a region selector, a URL
regex giving a source, D33) **are P10**, waiting for the trigger the sketch
named: a site the compiled profiles do not read. No such site is in view, and
rules designed with no page to test them against repeat what D33 refused
plugins for. Rejected: extraction rules alone (the sketch), leaving the
policy with no phase; both in one phase with no second site's pages.

## The deployment's config owns policy

Settled 2026-10-25 (maintainer). **A policy setting is the site's, set in
config; the core holds today's values as its defaults, and a compiled profile
sets none** (D32: a profile knows a generator, the deployment its site).
Whether a doubled slash is a fault, how loud an amber finding is and whether
a 403 counts are facts about a site and its readers, two Antora sites able to
differ on each. The inbox's "the profile's" reads as "the site's". Rejected:
a profile overriding the core's defaults with config over it, until a
default a generator itself decides turns up, which adds that layer without
breaking this one; a separate rules file named from config.

## A setting may change the gate

Settled 2026-10-25 (maintainer). **Three reaches are in scope**: (i) a kind's
severity in the HTML digest, touching no outcome; (ii) a reason switched off,
the link then judged on what it answered; (iii) an outcome reclassified. A
setting of reach (ii) or (iii) changes what D9 calls breakage, so **a run
records its policy, and the baseline file carries it** (D21), and a
comparison judges both runs by one policy (below).

## Judging is read time; asking is crawl time

Settled 2026-10-25 (maintainer). **Settings are two families.** *How to
judge* is applied when a run is read, in SQL over the facts the run stored,
as D28 judges fragments: a run records its own policy, a report on one run
judges it by that, and **a comparison (the gate, D9; the digest's since,
D73; `diff`) judges both runs by the newer run's policy**, so the gate needs
no refusal and a digest after a policy change compares like with like.
*What to ask* (D72's probe, `skip`, the allowlist) acts at crawl time and is
recorded per run; a run that did not ask is never re-judged as if it had. A
judging setting never needs a fact the crawl did not keep: where one would,
the crawl keeps it, and a run lacking it says so rather than reading clean.
Rejected: verdicts fixed at crawl time, each comparison then refusing a
mismatch or naming the policy change as a cause (D74).

## A setting sits at the scope of what it judges

Settled 2026-10-25 (maintainer). **A target's answer and a URL's shape are
the target's host's**, under `[host."<h>"]` beside its politeness, with a
top-level default; **severity is top-level**, the report's; **nothing is per
root** unless it judges the root's own pages, which no P7 setting yet does.
"Per-root" left the phase's name with the extraction rules (P10). Folds in
the inbox's question whether a site can turn `doubled_slash` off: per host.

## Severity: three fixed levels, the site mapping each kind

Settled 2026-10-25 (maintainer). **D76's levels stay the HTML's design**
(*broken*, *attention*, *pedantic*), **plus *none***, no dot and no finding;
**a site maps each outcome and reason to one**, D76's mapping being the
default. Colours are D76's accessibility contract, not policy. Rejected:
site-defined levels, which make the legend, the ranking and the print
styling data; a fixed mapping with colours configurable. Drains the inbox's
severity entry.

## What becomes a setting

Settled 2026-10-25 (maintainer). **A constant becomes a setting where a
reasonable site would rightly choose differently and the choice is meaning,
not mechanism.** What browsers and the web do stays a constant: D12's
fragment rules, D7's ten hops, RFC 9309's robots.txt limits. Every default is
today's value, so a run with no new key judges as it does now.

**Judging** (read time):
1. a status code's outcome, per host, D16's 401, 403 and 429 as `error` the
   default;
2. `doubled_slash` on or off, per host;
3. the doubled slash's reach, top-level: `full`, every doubled page the
   linking page's link reaches by relative links (554 on the 2026-10-24
   crawl, today's); `immediate`, the landing page alone (183); `off`, the
   link red and the doubled pages' broken relative links counted nowhere
   (D71). The depth costs nothing measurable, so the choice is meaning; this
   drains the inbox's last entry;
4. `pedantic_trailing_slash` on or off, off making such a redirect an amber
   one;
5. the schemes counted unchecked rather than `bad_scheme` (D66), a list the
   site extends;
6. the severity map (above);
7. **the gate's `fail` list of outcomes**, top-level, `broken_url` and
   `broken_anchor` its default, the newer run's (the CI run's) deciding: a
   site can keep a finding reported and not failing, as the severity map
   keeps one listed and quiet. `--inverse fail|warn` stays a flag, one
   content repo's job choosing rather than the site.

**Asking** (crawl time, recorded per run): per host, beside `per_second`, the
connect and total timeouts (10 s, 30 s), the retries (three, at 1, 2 and
4 s), the `Retry-After` ceiling (60 s, D16) and whether D72's probe is sent;
top-level, the body cap (16 MiB), a site of large pages otherwise reading
`too_large`.

**Kept constants**, each with its reason beside it in the code: D7's ten
hops and the sitemap index depth of three (the web's and the protocol's);
the 64 workers (mechanism, D61); the 3600 s `Crawl-delay` ceiling and D48's
30 s stop grace (the tool's own safety).

Rejected: `wrong_trailing_slash` switchable, which only hides the fix on a
link broken anyway, its probe being the asking setting; a new pedantic class
such as `http`→`https`, a new reason (D70's reopener) rather than a
decoupling; a fixed gate with reclassification its only lever, which drops a
finding from every report to stop it failing CI.

## P7 runs before P9

Settled 2026-10-25 (maintainer). P9's guide documents the config P7 extends,
and P7 changes the schema, so P9 follows it
([`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "P9 follows P7").

## Antora leaves the core's output

Settled 2026-10-25 (maintainer). **The generator knowledge in the reports is
P7's**: the digest's grouping becomes a setting, `[digest]` `group_by =
"<fact key>"`, **grouping by nothing by default**, P9's guide setting
`antora.component`; the digest's sentence that Antora writes the sitemap is
read off the profile; `report_broken`'s hint comes through the profile's
Views (D38) rather than a core view joining `antora_hints`. **The crawl's
Antora reads stay under KD6 and KD17**, a gate working without Antora facts
being another problem, each entry naming what would promote it. Rejected: a
default of `antora.component`, Antora back in a core default; the whole of
it as a layering defect for a seams round or an out-of-band item.

## The config's shape

Settled 2026-10-25 (maintainer). **`[rate]` becomes `[default]`**, holding
every `[host."<h>"]` key's default, asking and judging alike; the loader
refuses `[rate]` with a message naming `[default]`, a 0.x release breaking
configs knowingly. Top-level judging sits in `[judge]`
(`pedantic_trailing_slash`, `doubled_slash_reach`, `schemes`), then
`[severity]` (an outcome or reason to a level), `[gate]` (`fail`) and
`[digest]` (`group_by`); the body cap is a top-level key. **`status` maps
single codes**, never a class such as `4xx`; **`schemes` extends** the
built-in list. Key names are settled as each lands, within this shape, the
standing rule documenting each in `../manual/config.md`. A sketch:

```toml
[default]
per_second = 2
timeout = 30
slash_probe = true
doubled_slash = true
status = { 401 = "error", 403 = "error", 429 = "error" }

[host."pagure.io"]
timeout = 60
status = { 403 = "broken_url" }

[judge]
doubled_slash_reach = "full"
schemes = ["matrix"]

[severity]
error = "broken"

[gate]
fail = ["broken_url", "broken_anchor"]

[digest]
group_by = "antora.component"
```

## Where the policy lives

Settled 2026-10-25 (maintainer). **One concept is added, site policy, its
parts along D39's layering**: `config` parses it, holds its defaults and
validates it, in a `config/policy` child; `store` records a run's policy (a
schema change, D43, append-only as D2) and judges by it, `link_verdicts`
re-reading a target's stored outcome through the status map and switching
the reasons, with **a parameterised form, a DuckDB table macro, for a
comparison's verdicts under another run's policy**, every view still taking
none (D6; the macro earns its own `D<k>`). The asking settings sit where
their constants do (`fetch/http/host`, `crawl/slash`, the body cap in
`fetch`). `report` reads a run's recorded policy, never the file: the
severity map in `digest/html/severity`, the guide's entries in
`digest/html/guide`, the `fail` list in `gate`, `group_by` in
`digest/sections`. `report` and `crawl` consume the concept
and hold no part of it.

## The digest states its policy

Settled 2026-10-25 (maintainer). **The HTML's Housekeeping lists every
setting differing from its default; the JSON carries the whole policy it
judged by.** A judgement the reader cannot see is a broken experience of the
report itself. Rejected: the JSON alone; neither.

## The guide follows the policy

Settled 2026-10-29 (maintainer). **The HTML's guide ("How to read this
report") is P7's wherever its text states a judgement**: an entry is shown
only while the reason it explains is judged, and the words stating a setting
are read off the run's recorded policy, as the severity map is. The
doubled-slash entry states `doubled_slash` and `doubled_slash_reach` (the
link broken, the doubled page's relative links counted on the linking page),
and lands with them in P7.3; the pedantic entry states
`pedantic_trailing_slash` (P7.3) and the redirect's level and place under
Housekeeping (P7.5). Rejected: the text left to P11's overrides, the guide
false for a site's own digest from P7.3 until then.

## A report may judge by a trial policy

Settled 2026-10-25 (maintainer). **`bruce report --config <file>` judges the
run, and any run compared with it, by that file's judging settings**, its
asking settings ignored, a run being unable to be asked again; the
Housekeeping names the policy a trial's. Without it, seeing what a severity
map or `fail` list does takes a crawl (75 minutes for en-US, about 20 hours
for every locale). D5 holds: a file stands in whole, no flag overriding a
key.

## What a status and a severity key may say

Settled 2026-10-26 (maintainer). **A status maps to `ok`, `broken_url` or
`error`**, for any code but a 3xx, which is a hop, remapping it being asking
(D7) rather than judging; an unmapped code keeps D16's class. A fragment on
a target remapped to `ok` had no body read and is `unchecked`, reason
`no_anchors`, as on any `ok` target with no page. **`[severity]` keys an
outcome (`broken_url`, `broken_anchor`, `error`), `redirected`, or any
reason, a reason's key beating its outcome's**, so `doubled_slash =
"attention"` softens that `broken_url` alone; `unchecked` keeps no level
(D76), and an unknown key is refused at load.

## The slices

Settled 2026-10-26 (maintainer). Evidence first, and the rework of
`link_verdicts`, a tested core path, apart from the keys it carries. D43
refuses a DB of another version, so the refactor lands before the schema
change, while the 2026-10-25 crawl's DB can prove it. Each slice amends the
`D<k>` entries it changes (D5, D9, D16, D70, D71, D76 among them) in its own
change.

1. **P7.1 — Verdicts judged through a policy, defaults only.**
   `link_verdicts` becomes the table macro and its view, today's constants
   its literal defaults, no schema change. Evidence: every report and the
   digest's JSON byte-identical on the 2026-10-25 crawl's DB before and
   after, and the digest's time measured beside it.
2. **P7.2 — A run records its policy.** `config/policy` with its defaults,
   `[default]` replacing `[rate]`, the schema change, the baseline file
   carrying the policy; judging unchanged.
3. **P7.3 — The judging keys**: `status`, `doubled_slash`,
   `pedantic_trailing_slash`, `schemes`, `doubled_slash_reach`, one run
   judged by its own policy, the guide's entries following them.
4. **P7.4 — Comparisons judged by the newer run's policy**: the gate, the
   digest's since, `diff`.
5. **P7.5 — The severity map, the gate's `fail` list, and the policy
   stated** in the Housekeeping, the JSON and the guide.
6. **P7.6 — `bruce report --config`**, the trial.
7. **P7.7 — The asking settings**: per-host timeouts, retries, the
   `Retry-After` ceiling and D72's probe, and the body cap.
8. **P7.8 — Antora leaves the output**: `group_by`, the profile's sitemap
   sentence, `report_broken`'s hint through Views.

P7.7 and P7.8 depend on nothing before them; they come last so the judging
path settles before P9 reads the config.

## Facts the grilling found

Read off the source at `6327824`, 2026-10-25.

- **Config holds scope, politeness and the hash scope, and no policy.** Keys:
  a root's `url`, `include`, `exclude`, `strip_query`, `sitemap`, `profile`,
  `hash_root`, `hash_exclude`; top-level `allowlist`, `skip`, `user_agent`;
  `[rate]`; `[host."<h>"]`'s `per_second`, `concurrency`, `robots`
  (`config.rs`). `check-dir`'s `--externals` and `--inverse` are the only
  flags choosing a policy.
- **Every outcome and reason is a constant.** 401/403/429 as `error`
  (`crawl/outcome.rs`); `doubled_slash`, `pedantic_trailing_slash`,
  `wrong_trailing_slash` always on (`store/views.rs`, `crawl/slash.rs`); the
  twelve unchecked schemes, `appstream` among them (`crawl/frontier.rs`);
  ten hops (`crawl/hops.rs`); sitemap depth 3; 16 MiB body cap.
- **The gate's rule is fixed**: an introduced `broken_url` or `broken_anchor`
  of any kind fails, `error` and redirects never do (`report/gate.rs`), the
  inverse check failing unless `--inverse warn`.
- **Severity and presentation are constants**: D76's levels
  (`report/digest/html/severity.rs`), its colours (`html/style.css`), the
  section order and ranking (`html.rs`, `sections.rs`), the top five per
  unchecked reason and ten external hosts (`html.rs`).
- **Extraction heuristics are constants**: the content regions (`main`,
  `[role=main]`; `article.doc`), the edit link's classes and English phrases
  (`extract/edit_link.rs`), the forge shapes, Pagure among them (`forge.rs`),
  and Antora's nav, search-script, language-menu and unresolved-xref
  selectors (`profile/antora.rs`). Locales are learned only through Antora's
  language menu (D67).
- **Politeness constants**: 10 s connect and 30 s total timeouts, three
  retries at 1, 2 and 4 s, a 60 s `Retry-After` ceiling (`fetch/http.rs`),
  64 workers (`crawl.rs`).
- **Antora's facts are read by key outside `profile/antora`**:
  `antora.component` and `antora.version` in `crawl.rs` (KD6),
  `crawl/xref.rs`, `crawl/baseline.rs` (KD17) and `report/inverse.rs`; the
  digest's `by_component` (`report/digest/sections.rs`) and its sitemap
  sentence naming Antora (`report/digest/html.rs`); `report_broken` joining
  `antora_hints` (`report/views.rs`). No host name or `en-US` steers
  behaviour.
