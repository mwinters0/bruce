# Upstream fixes we are waiting on

A defect or limit whose fix belongs to a dependency (a crate, Antora, the sites
we check), what we do until the fix ships, and what to change once it does.
What an entry holds, when one is added and when it is struck are
[`.claude/skills/upstream-issue/SKILL.md`](../../.claude/skills/upstream-issue/SKILL.md),
and every dependency upgrade reads this file. Each site to revisit carries an
`upstream: UF<k>` comment.

`cd scripts && uv run upstream.py` resolves every entry to its markers and
back. Numbers up to the watermark are allocated, and none is reused.
<!-- upstream-watermark: UF0 -->
