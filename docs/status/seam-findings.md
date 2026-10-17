# Seam findings

The seam register: antipatterns a seams read ([`../process.md`](../process.md),
"Seams") found, ruled a defect, and left for a later pass. With this register a
phase can close on a tree that is not yet ideal, and the next session in a
module meets what is owed there. It sits beside
[`deficiencies.md`](deficiencies.md). The line between the two is who bears the
cost: a `KD<k>` costs a user something, an `S<k>` only the next person reading
or changing the code.

- **An entry is the symptom, never the remedy.** One line,
  `- **S<k>** — *<antipattern>.* <symptom>`, wrapped to at most four.
- **The detail is at the markers**: a `seam: S<k>` comment at each site the
  symptom spans.
- **An entry is struck by the change that removes the symptom**, index line
  and every marker together.
- **The read is the authority, not the register.** A seams round reads the
  tree without opening this file, then reconciles the two.

`cd scripts && uv run seam_register.py` resolves each entry to its markers and
back. Numbers up to the watermark are allocated, and none is reused.
<!-- seam-watermark: S0 -->
