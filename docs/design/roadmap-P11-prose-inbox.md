# P11 inbox

Facts addressed to the phase putting the digest's prose in config before it
is grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## The guide's text already states the policy

- **Fact.** The guide's entries, and the words stating a setting in them,
  follow the run's recorded policy (D91, `report::digest::html::guide`).
- **Why P11 cares.** An override replacing an entry's text replaces text that
  states a setting, so whether a template reads the run's recorded policy
  (the sketch's open question) is already answered for the built-in text: an
  override that does not can be false for the run it is printed in.
- **Origin.** The site policy's wrap, 2026-10-31, read off the source at
  `ce2b1b4`.
  Contingent on the guide still reading the policy.

## The digest's palette leaves the code, and the deployment may want it back

- **Fact.** The digest's palette is a neutral default. The palette it
  replaced, which docs.fp.o's deployment may want to restate in config, is
  `src/report/digest/html/style.css` at `e30483c`, `:root`'s custom
  properties: `--bg: #12151c; --surface: #1a1e28; --raised: #222735;
  --text: #e4e7ee; --muted: #9aa3b5; --line: #2d3445; --accent: #51a2da;
  --accent-deep: #294172; --bad: #f0868b; --warn: #e5be6b; --good: #7fcf9a;
  --new: #51a2da`, `--shade: rgba(81, 162, 218, .45)` (the accent at .45),
  dark `color-scheme`; under `@media print`, light: `--bg: #fff; --surface:
  #fff; --raised: #f2f4f8; --text: #111; --muted: #555; --line: #ccd;
  --accent: #1f5f93; --accent-deep: #fff; --bad: #a11; --warn: #7a5a00;
  --good: #1a6b36; --new: #1f5f93`. `git show e30483c:src/report/digest/html/style.css`
  recovers the whole sheet. The digest's sentence on a forge's new-issue
  form or a login page answering 200 stayed built in, a login wall being the
  web's (I17).
- **Why P11 cares.** Theming the digest from the deployment's config is the
  digest's presentation in config, beside its prose: which properties a
  deployment sets, and whether print's are set apart, are its grilling's.
- **Origin.** The declarative extraction rules' grilling, 2026-10-31
  (maintainer: P11 must be able to recall the scheme). Contingent on the
  neutral palette staying the default.
