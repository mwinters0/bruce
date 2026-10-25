# P8.21 — What is new since the compared run, marked in every section: notes

Earned by 8.20's fifth finding; the rule is D75.

- **On the 2026-10-25 en-US crawl against its copy of 8.14's** (`runs/en-us-20261025-1301/`):
  no new break and one fixed, so no `broken` row is new; the two new errors
  are marked in "Links that errored" and in their pages' drill-downs, those
  pages and their repo counting "1 new" and "2 new"; 16 redirect landings are
  new. Rendered with `scripts/shot.py`, desktop and print, the bar reads in both.
- **`novelty::mark` runs last** and recomputes `digest_page_counts` for the
  compared run, overwriting the digest's own run's counts: every section
  reading them is built by then. A section added later that reads the counts
  is built before it.
- **No new row in section 5 is ranked first**: the drill-down's rows keep their
  severity order, the bar alone marking the new ones.
- **`by_component` has no `new`**: it is in the JSON for graphing and in no
  section of the HTML.
