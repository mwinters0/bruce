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

Allocated so far: M1–M3. <!-- out-of-band-watermark: M3 -->

## The ledger

| Item | Date | What changed | Blocks | Why |
|---|---|---|---|---|
| M1 | 2026-10-17 | `--affected` runs a Rust test naming a docs file; the manual's `toml` examples are tested | | [`2026-10-17`](../status/history/2026-10-17.md), "M1: the manual's examples are tested" |
| M2 | 2026-10-20 | `link_verdicts` rule 1 reads a refspec as unresolved where the link points at its own page or its fragment is the refspec | | [`2026-10-20`](../status/history/2026-10-20.md), "The review of how a resolved xref is told" |
| M3 | | The gate (`check-dir`, `report introduced`) judges every link kind by default, and `check-dir` drops `--kind`; other reports keep `anchor` | P5 | [`2026-10-20`](../status/history/2026-10-20.md), "The review of the gate's default kind" |
