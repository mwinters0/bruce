# P1.5 Store and the profile trait: notes

What landed, and what the next slices inherit. The row is `P1.5` in
[`../status/STATUS.md`](../status/STATUS.md), its intent in
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md).

## What landed

- **`store/`**: `Store::open` creates D26's schema as D34, D35, D36, D37 and
  D43 amend it, refuses a file holding tables but no `meta`, refuses a
  `schema_version` other than `SCHEMA_VERSION` naming both, and creates every
  registered profile's views. `begin_run` and `finish_run` own the `runs` row.
  `RunLookup` is the store's `Lookup` for one run.
- **The writer** (`store/writer.rs`): a thread owning the `Store`, a bounded
  `std::sync::mpsc` channel, one appender a table, a flush every 10,000
  rows. It takes `Row::Page` and `Row::Fact` only; the other tables' rows
  join `Row` with the phase that produces them. `Writer::finish` hands the
  store back, so the fragment-verdict SQL (D28) runs on the same connection.
- **`profile`**: the `Profile` trait (D38), `Facts`, `Lookup`, `View` and
  `Registry`. `generic` and `antora` implement Detect and Views; their
  Extract returns no handlers and their Resolve returns `None`.
  `cli::registry()` is the one place naming both (D39).
- **New dependencies:** `duckdb` (`bundled`, D23) and `lol_html`.

## What the next slices inherit

- **P2 fills `Profile::extract`.** Its type is lol_html's own,
  `Vec<(Cow<'static, Selector>, ElementContentHandlers<'h>)>`, so the core
  appends it to `Settings::element_content_handlers` unchanged. The facts
  sink is a `&RefCell<Facts>`, as lol_html's handlers are `FnMut` closures
  borrowing it. `Facts::record("component", ..)` stores `antora.component`,
  so a profile cannot write outside its namespace. Region, edit-link and
  tagged-link output have no sink yet: P2 adds them to what `extract`
  borrows, which changes the trait's signature and both profiles, and
  nothing else.
- **P2 reads the generator meta before choosing.** `Registry::choose` takes
  the meta's content as `Option<&str>`; the meta precedes anything a profile
  extracts (D35), so the core runs a first small pass for it, or reads it
  from the head before the rewriter starts.
- **`pages` has no URL column**: a page's URL is `targets.url` at the same
  `(run_id, url_id)`, one row per unique URL (D26). `RunLookup` joins the
  two; the writer needs a `Row::Target` before a page can be found.
- **P3 decides the channel's runtime.** The writer is a plain thread behind
  `std::sync::mpsc::sync_channel`, so a blocking send is the backpressure.
  From a tokio task the sender needs `spawn_blocking`, or the channel
  becomes tokio's with `blocking_recv` here; the thread, the appenders and
  `finish` stay.
- **`content_hash` is a `BLOB`**, since D15 names no hash and a 64-bit
  integer would fix the choice; P2 picks the function.
- **`page_facts` has no uniqueness**: a profile recording a key twice for one
  page gets two rows, and `antora_pages` shows `max(value)`. The Antora
  extract records each key once.
- **The D6 views** (`page_stats`, `broken_links`, the rest) are P4's. A
  profile's views are created with `CREATE OR REPLACE` on every open, so a
  changed view needs no schema version.

## Negative results

- **`information_schema` is the existence check**, not a failed query:
  `Store::open` asks for the `meta` table by name, and a file with other
  tables and no `meta` is left unwritten.
- **A profile's tests cannot name a concrete profile**: `module_graph.py`
  confines `generic` and `antora` to `cli`, so `profile`'s and `store`'s
  tests use stub profiles, and the Antora view is tested in `cli`, which
  sits above both.
- **D37 said `page_id`**; every other table, and so the code, keys a page by
  `url_id`. D37 now says that.
