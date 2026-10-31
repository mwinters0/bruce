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

Allocated so far: M1–M29. <!-- out-of-band-watermark: M29 -->

## The ledger

| Item | Date | What changed | Blocks | Why |
|---|---|---|---|---|
| M22 | 2026-10-29 | The digest's HTML ends with "How to read this report", its entries for a doubled slash and a pedantic redirect, and an ⓘ beside each term in the rows links to its entry | | [2026-10-29](../status/history/2026-10-29.md) |
| M23 | 2026-10-30 | `mise run shot` renders with Chrome's headless shell in a pinned container, Servo unpinned and `runs/headless-browser-trial/` deleted | | [2026-10-30](../status/history/2026-10-30.md) |
| M24 | 2026-10-30 | `mise run shot`'s phone render emulates a phone: `mobile=True` at 390 px, device scale 1 | | [2026-10-30](../status/history/2026-10-30.md) |
| M25 | 2026-10-31 | The digest's words under a severity map: a count part moved off its outcome's level names its reasons, the legend's *none* line and the manual stop saying the figures leave such a kind out, a list the map empties says so | | [2026-10-31](../status/history/2026-10-31.md) |
| M26 | | A digest's comparison judges both runs by the newer run's policy when `--from` names a later run (KD24) | P9 | [2026-10-31](../status/history/2026-10-31.md) |
| M27 | | Under `--externals=skip` a redirect's hop onto a URL on a root's host outside every scope asks nothing (KD25) | P9 | [2026-10-31](../status/history/2026-10-31.md) |
| M28 | | `[severity]` names `invalid_url` (KD26) | P9 | [2026-10-31](../status/history/2026-10-31.md) |
| M29 | | The page and message texts the P7 wrap's repoint found false: an empty baseline's refusal, the compared run's source columns, the digest's footer on the JSON | P9 | [2026-10-31](../status/history/2026-10-31.md) |
