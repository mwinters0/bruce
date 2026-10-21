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
allocation order.

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

Allocated so far: M1–M10. <!-- out-of-band-watermark: M10 -->

## The ledger

| Item | Date | What changed | Blocks | Why |
|---|---|---|---|---|
| M4 | 2026-10-21 | A versionless refspec resolves to the version the baseline serves off its own segment; versions order numerically, case-folded (KD14 rewritten) | | [`2026-10-21`](../status/history/2026-10-21.md), "M4 landed" |
| M5 | | SIGINT and SIGTERM each stop `crawl` and `check-dir`; a stopped `check-dir` runs no gate; a stopped run exits 128 plus the signal (KD13, with M6) | | [`2026-10-21`](../status/history/2026-10-21.md), "The KD register grilled into M items" |
| M6 | | A bare `report` warns of a later unfinished run and refuses a baseline a later run imported (closes KD13, with M5) | | [`2026-10-21`](../status/history/2026-10-21.md), "The KD register grilled into M items" |
| M7 | | Two roots are one root by canonical URL (closes KD11) | | [`2026-10-21`](../status/history/2026-10-21.md), "The KD register grilled into M items" |
| M8 | | `prune` spares the baseline run a kept run names (closes KD12) | | [`2026-10-21`](../status/history/2026-10-21.md), "The KD register grilled into M items" |
| M9 | | A Pagure `/edit/<ref>/f/<path>` link parses as Pagure's shape (closes KD15) | | [`2026-10-21`](../status/history/2026-10-21.md), "The KD register grilled into M items" |
| M10 | | An explicit refspec whose version is served off its own segment is named under the served one, as a versionless one is (D57 rewritten) | | [`2026-10-21`](../status/history/2026-10-21.md), "M4's explicit-version asymmetry reviewed" |
