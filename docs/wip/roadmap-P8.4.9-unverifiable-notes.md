# P8.4.9 — A summary counts its unverifiable links apart: notes

Earned by 8.20's ninth finding. `tally` takes the broken URLs, the links
broken through a doubled slash, the `error` links and the missing fragments,
in that order, the first three red. Ranking is unchanged: `pages` and
`by_repo` still rank by `broken_url` plus `error` plus `through`, then
`broken_anchor`, so a page of two unverifiable links outranks one of a
missing fragment, as before. "Unverifiable" is an adjective, so it takes no
plural; the section and the figure keep "could not be verified" (8.4.8).
