# P8.19.2 — Each figure coloured as its links' dots: notes

Earned by 8.20's fourth finding. A figure's class follows `Level::of_link`:
`bad` (red) for the broken URLs and the errored links (`Level::Broken`), `warn`
(amber) for the missing fragments (`Level::Attention`). The figures' class is
written beside each label in `html::figures`, not derived from the level, the
figures being five fixed cells.
