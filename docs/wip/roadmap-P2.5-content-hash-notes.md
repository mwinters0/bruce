# P2.5 The content hash: notes

What the next slices inherit, and what was tried and refused. The intent is
[`roadmap-P2-extractor.md`](roadmap-P2-extractor.md), "The content hash";
what exists is the code (`src/extract/hash.rs`, `src/config.rs`) and
[`../status/STATUS.md`](../status/STATUS.md).

## What the next slices inherit

- **`extract` takes the root's `config::HashScope`** beside its profile
  choice, and `Extracted::hash` is a `u128`. P1's `pages.content_hash` is a
  `BLOB`, so the store's writer (P3) picks the byte order; nothing reads the
  hash yet.
- **Each `hash_root` and `hash_exclude` entry is split at its top-level
  commas** in the loader, outside quotes, brackets and parentheses, so
  `"header, footer"` is two selectors and `p:not(.a, .b)` one. The pass
  needs single selectors, since it extends each into `<s> *` for the
  elements beneath it.
- **D33's vocabulary is what `lol_html` parses**: it refuses `+`, `~`,
  `:has()`, `:last-child` and namespaces with its own reason, which the
  loader passes on under the key's name.
- **What is hashed**: each start tag's name, its attributes' names and
  values decoded, and each text node decoded, every field length-prefixed.
  Comments, the doctype and end tags are not. Text is accumulated over its
  chunks before decoding, so an entity split across two writes hashes as
  one (`src/extract/hash/tests.rs`).
- **Fixture facts.** On all 30 docs.fp.o pages, rewriting the `Last build:`
  timestamp leaves the hash unchanged under the default scope
  (`article.doc`) and under `hash_exclude = ["footer"]`, and changes it
  under `hash_root = ["html"]`; a paragraph added to the article changes it
  under both (`tests/extract.rs`). The 40 fixtures' hashes are distinct and
  each snapshot gained a `hash:` line.

## Negative results

- **Scope by counting start and end tags** was not built: `lol_html` runs an
  end-tag handler only for an explicit end tag, so a root or exclusion
  closed implicitly (`<p>`, `<li>`) would leave its count wrong for the rest
  of the page. The scope is `lol_html`'s own selector matching instead:
  `<s>` and `<s> *` for elements, `<s>`'s text handler for text.
- **Hashing end tags** was dropped for the same reason. A tag moved within
  the text still changes the hash, since it splits the text nodes
  differently.
