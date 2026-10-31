# P10 inbox

Facts addressed to the declarative extraction rules phase before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## The edit link's heuristics are English and theme-bound

- **Fact.** The edit-link ranking's phrases are English ("edit this page",
  "edit page", "edit on …") and its classes three themes'
  (`edit-this-page`, `td-page-meta--edit`, `theme-edit-this-page`)
  (`extract/edit_link.rs`); the generic profile's content region is `main` or
  `[role=main]` (`profile/generic.rs`); locales are learned only through
  Antora's language menu (D67).
- **Why P10 cares.** Each is extraction a site in another language or theme
  would extend by a rule rather than Rust: whether a rule can add an edit
  phrase or class beside D36's tiers, a region (D34), or a language menu's
  selector.
- **Origin.** The site policy's grilling, 2026-10-25, read off the source at `6327824`.
  Contingent on D36's tiers and D67 staying as they are.
