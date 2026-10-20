# P5.11.1 notes: an alias another component owns is not a removed page

Outcome: `report::inverse`'s `typed` CASE judges a `page` by the target's URL
only when the target did not redirect to a landing outside every held prefix;
such an alias (a meta refresh in a held component-version whose landing is
another component's page) is that owner's, and is removed only with its landing
(`landing_gone`). KD9 is struck, index line and marker together.

- **Evidence, re-run on 5.11's DB** (`runs/p5.11-quick-docs/ci.duckdb`, a copy,
  `bruce report inverse --run N`): as published, run 2, 0 links where it had 1
  (`neurofedora/contributing` to `creating-rpm-packages/`); planted, run 3, 22
  links on 22 pages where it had 23, the 16, 3 and 3 planted ones unchanged.
  The check-dir run itself was not repeated: `report inverse` reads the DB
  alone (D56).
- **Test**: `report::inverse::tests::an_alias_landing_outside_the_held_set_is_its_owners_and_one_landing_inside_is_not`
  fails with the guard removed.
- **Kept**: an alias landing on a held page still reads as removed when the
  build lacks the alias (a rename that drops its `page-aliases`); only a
  landing outside the held set hands the alias to its owner.

## What the next slice inherits

5.12's closing check is the as-published run exiting 0: the inverse half is now
0 rows; the four introduced rows are KD8's.
