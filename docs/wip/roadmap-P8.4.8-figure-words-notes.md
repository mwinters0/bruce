# P8.4.8 — The figures' unchanged figures silent, an errored link "could not be verified": notes

Earned by 8.20's findings 7 and 8.

- **"External links that errored" was refused** as untrue by definition: D16's
  `error` takes any host's refusal, 5xx or silence. On the 2026-10-25 en-US
  crawl all 89 are off docs.fp.o (29 × 401, 26 × 403, 22 × 503, 6 TLS, 5
  timeouts, 2 other, 1 connect), so the word would read true there and lie on
  the night docs.fp.o answers 503.
- **"Could not be verified" sits between "broken" and "not checked"**: a broken
  link's target answered that it is not there, an unverified one was asked and
  gave no verdict, an unchecked one was not asked. The nav reads "Unverified";
  the section keeps its id `errors`, and the JSON its `error`, a rename being
  one a loader would misread (D64).
- **Left as it is**: a page's and a repo's summary count an unverified link
  among its "broken links" (8.4.5); STATUS's "Decisions worth another look"
  holds it.
