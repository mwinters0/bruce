# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**bruce** is a Rust link checker for docs.fedoraproject.org. It validates
anchors (`page#fragment`), keeps a DuckDB link graph with run history, and
gates a content repo's CI on the breakage that repo introduced. Every use case
has an implementation: `docs/status/STATUS.md` says what is built, and
`docs/design/roadmap.md`'s phase index which phase is next. **The code is the
ground truth.**
Docs exist to locate it quickly and to record what it cannot explain.

## Commands

```sh
mise run check [--affected] [--verify]   # the round's checks once; summary here, logs in runs/check/
cd scripts && uv run citations.py        # every `<doc>.md`, "section" citation resolved
cd scripts && uv run deficiencies.py     # KD index vs code markers, vs STATUS checklist, vs phase index
cd scripts && uv run upstream.py         # UF register vs `upstream: UF<k>` markers
cd scripts && uv run seam_register.py    # S register vs `seam: S<k>` markers
cd scripts && uv run repoint.py          # the record's caps, and its growth since the last blind read
cd scripts && uv run notices.py          # THIRD-PARTY-NOTICES.txt regenerated from Cargo.lock
cd scripts && uv run workflows.py        # no `uses:` in any workflow; `script release.yml` rehearses one
cd scripts && uv run file_limits.py      # every .rs against the size limits (`docs/process.md`, "Seams")
cd scripts && uv run module_graph.py     # every module edge one-way, read off rust-analyzer
cd scripts && uv run concepts.py         # every production .rs claimed by one concept in docs/design/concepts.md
cd scripts && uv run python -m unittest test_<script>   # one script's own tests
mise run sim                             # the deployment guide's tagged blocks into sim/, config checked to load
mise run e2e                             # the fixture Antora site built, crawled and gated under runs/e2e/ (slow, needs node; `check` runs it when the binary is affected)
mise run build | test | clippy | fmt | bench   # one cargo command; Rust is pinned in mise.toml
```

Every `scripts/*.py` has a
`--help` and a `test_*.py`. The harness's own rules are asserted there, not
remembered here.

## Long-running processes

A job over about 10 minutes, such as a full `bruce crawl` of docs.fp.o at its
`Crawl-delay` (D18), is launched fully detached with `setsid`, logging under
`runs/` (gitignored). It is **never waited on, monitored, or given a completion
notification**: waiting expires the prompt cache and reloads the whole
conversation. Record what was launched, and its log path, in today's history
entry, and a later session reads the result. Stop a job by process group
(`pkill -g <pgid>`). Nothing else may build or test while a performance
measurement runs, as that disturbs its timing. A crawl paced by its hosts'
rates is not a measurement. Over 30 minutes inside `/gosub`, follow
`.claude/skills/gosub/handoff.md`.

## Where things are, and when to read them

- `docs/process.md`: the development process. **Don't read it directly.
  Invoke the `process` skill** before implementing or wrapping a phase or
  slice, writing a spec, repointing, or updating `STATUS.md`.
- `docs/design/decisions.md`: the `D<k>` register, capped at 700 lines.
  **Read the entries for a mechanism before changing it or proposing an
  optimization to it.** Cite an entry, never restate it.
- `docs/design/site-invariants.md` (`I<n>`): what docs.fp.o and Antora do
  that decisions rely on. **Add an entry when a decision starts depending on
  external behaviour. Walk the file at a new Antora version, or when the site
  moves.**
- `docs/design/roadmap.md`: goals, "Standing rules", and the phase index.
  Each phase is `P<k>`, allocated on discovery and never reused.
  - **The specs and slice notes of phases in flight live in `docs/wip/`** and
    move to `docs/design/` at the wrap.
  - A phase's inbox, `docs/design/roadmap-P<k>-<slug>-inbox.md`, is read and
    drained when the phase is grilled. **File into one the moment a slice turns
    up a fact a distant phase needs.**
  - `docs/design/historical/initial.md` is the frozen bootstrap plan. It is
    never edited.
- `docs/design/concepts.md`: the concept map. **Read it before deciding where
  new code goes**, grilling a phase, or a seams round.
- **Red meters have their own rounds.** When `mise run check` shows the
  repoint meter red, invoke `repoint`. When it shows the seams meter red, or a
  slice leaves a file over a limit, invoke `seams`. **Invoke `upstream-issue`
  before working around a dependency's defect.**
- `docs/design/out-of-band.md`: work belonging to no phase. **Read it whole
  before allocating an `M<k>`** and before picking up unscheduled work.
- `docs/status/STATUS.md`: what is built. `docs/status/deficiencies.md`
  (`KD<k>`), `seam-findings.md` (`S<k>`) and `upstream.md` (`UF<k>`) sit
  beside it. **Read the deficiencies before grilling a phase or changing a
  mechanism.**
- `docs/status/history/YYYY-MM-DD.md`: only what a future session must pick
  up, and discoveries that changed the plan. Rules are in that directory's
  `README.md`.

A citation names a document and a section, and `citations.py` resolves every
one. Run it after rewriting a heading.

## Standing rules

- **Design decisions are never made on implementation cost.** Where one option
  clearly maximizes what we deliver (features, code quality, performance), it is
  the choice and the recommendation, however much work it takes, and a defect
  found is fixed rather than deferred because fixing it is work. The
  counter-example: a review of D61 recommending the schedule be left crowding a
  build's reads, unmeasured, over fixing it.
- **Document what _is_, not what _was_**, inside `docs/status/history/` too. A
  fact is stated once, in one place, and cited everywhere else.
- **Claude Code's auto-memory (`~/.claude/projects/…/memory/`) is not used
  here.** It is machine-local, and development may move machines. A lesson
  goes into the repo, in the doc or skill that governs it. Propose it to the
  maintainer first if it adds a rule.
- **Commit straight onto `main`**, never on a branch. Push only when asked.
- **The session scratchpad is not a home.**
  - Before a session or round reports, whatever a later one needs moves to
    `runs/`, and the record (notes doc, history entry, handoff doc) names it
    there. Everything else in the scratchpad is deleted.
  - A file the record does not name is one no later session can find, so it
    is transient by definition.
  - Nothing expected past about 100 MiB is written to the scratchpad, even
    briefly.
  - A `SessionEnd` hook (`.claude/helpers/scratchpad-end.sh`) removes the
    session's directory as a backstop, not as the protocol.
