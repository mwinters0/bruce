# Out-of-band work

The ledger of work that belongs to no phase. It sits beside
[`roadmap.md`](roadmap.md)'s phase index rather than inside it because it
grows monotonically between keystones. A register that is read only as far as
someone's window reaches will reissue a number already spoken for, and the rows
here are a work queue as well as a record. The rule this file implements is
[`../process.md`](../process.md), "Out-of-band work".

An item is a change that fits one session and answers to no phase's intent: a
CLI ergonomics change, say, or a defect fix that changes no decision. It gets a
number `M<k>` and **one terse ledger line**: the date, what changed, whether it
blocks the open phase, and the history entry that says why. Nothing else. There
is no spec, because there was no intent doc to write. There is no notes doc,
because the history entry *is* the notes.

**A number is allocated on admission, not on landing.** The Date column stays
empty until the item lands, or names the day it was withdrawn. Row order is
allocation order. `scripts/repoint.py` holds the watermark at or above every
row, the prose stating it alike, and the rows in order.

**The admitting session sets the Blocks column.** It names the open phase when
that phase's remaining slices should not be landed around the item, and is
empty otherwise. [`go`](../../.claude/skills/go/SKILL.md) takes a blocking row
ahead of the next unticked slice. With no phase open, it takes every row in
allocation order. Blocks is cleared when the item lands.

**Admission rule.** An item is out-of-band only if it changes no decision any
spec records **and** fits one session. Anything else goes back through
grilling, spec amendment and a numbered slice.

**Ledger lines stay one line each.** The detail lives in the dated history
entry each line points at.

Allocated so far: M1–M45. <!-- out-of-band-watermark: M45 -->

## The ledger

| Item | Date | What changed | Blocks | Why |
|---|---|---|---|---|
| M44 | | `e2e.check_workflow` parses every workflow under `.github/workflows/` with js-yaml as it does the guide's, the steps held equal to `workflows.steps`; `.github/workflows/` joins `E2E_READS` |  | [2026-11-03](../status/history/2026-11-03.md), "Review of the CI workflow's YAML parse" |
| M45 | | The deployment repo's `nightly.yml` sets `KEEP_JSON=30`, and its `bruce-nightly.sh` is the guide's again (`KEEP_JSON` defaulting to 90) | P9 | [2026-11-03](../status/history/2026-11-03.md), "Review of the dev deployment's JSON retention" |
