# P1.4.1 Config keys per host: notes

What landed, and what the next slices inherit. The row is `P1.4.1` in
[`../status/STATUS.md`](../status/STATUS.md), its intent in
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md).

## What landed

- **`[host."<h>"]`** replaces `rate.host` and the root's `robots` and
  `crawl_delay`: `Config::hosts` maps a normalised host to `HostSettings`
  (`per_second: Option<f64>`, `robots: bool`). `Rate` is the default alone.
  Which rate and robots rule applies is `HostSettings`'s rustdoc and the
  manual's "Hosts", D8 and D18 giving the reasons.
- **A root's `url` must be in its own scope**, checked in `Root::from_raw`
  with `in_scope`, so a default-scoped root cannot fail it and an `include`
  or `exclude` can.
- **`robots = false` on a host no root starts on is refused**, by
  `Config::check_robots_hosts`.

## What the next slices inherit

- **P3:** a file naming no root skips that check at parse time, since the
  command line may supply the root; whoever adds it runs
  `check_robots_hosts` again (`../design/roadmap-P3-crawler-inbox.md`).
- **The precedence is unimplemented.** Nothing reads `HostSettings` yet.

## Negative results

- **The manual's `include` example failed the new rule**: its root started at
  `/en-US/`, outside `*/fedora/**`. The example and one test now start at
  `/en-US/fedora/`.
