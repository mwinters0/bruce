# Profiles

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. What a site generator adds to the core: detection by
generator meta, facts under namespaced keys, the edit-link tie-back to a source file, Antora's resource IDs and xref
resolution, and the views and sentences the reports read through a profile. `profile` holds the trait and `generic` and
`antora`; `extract::edit_link` the edit-link ranking; `forge` the edit-URL shapes.

### D32 Two extension layers: generator profiles in code, the deployment in config
The core knows the web; a profile knows a site generator (`generic`, `antora`); a deployment (docs.fp.o) is config
only, owned outside this repo (D41), its markup an extension (D94). Forge edit-URL shapes are core, as every
generator links to a forge. Compiled `antora` holds what Antora, its default UI and official extensions emit: all
docs.antora.org needs; Fedora's bundle's and build's are config, a line drawn by best effort until other generators
are compiled. Rejected: a compiled `fedora` profile above `antora`; a report `--component` filter, Antora in the core,
where `--fact antora.component=…` reads `page_facts` (D37). Reopens: a deployment quirk config cannot express.
Evidence: `tests/extract.rs` on `fixtures/docs-antora-org`; the en-US crawl of 2026-11-01, unchanged by the guide's
extension (`runs/en-us-20261101-0005/compare.sql`).

### D33 Profiles are compiled in, or declared in config as a `[[profile]]` array
A compiled profile is a module behind a trait, so a new generator is a PR. A declarative one is that trait from config,
named by `name`, writing key/value facts (D37): a fact rule's value is the first element yielding one, an empty
attribute being one and empty text not; source rules are D93, locale links D67. Each selector is lol_html's streaming
CSS (D11), refused at load, under a key named for it, so a richer language is a key beside it. Resolve is its base's
(D94) or none, and Views none. Rejected: WASM or out-of-process plugins, no second user; a table keyed by name, TOML not
ordering keys where D35 tries them in file order; rules per root, a profile being what D35 chooses by; the first match
alone, ignoring `pattern`. Reopens: a multi-valued fact. Code: `config::profile`, `profile::declarative`. Evidence:
`profile::declarative::tests`.

### D35 A root's profile defaults to `auto`, which picks one per page by its generator meta
`profile = "auto" | "generic" | "antora" | <a declared name>` per root. Under `auto` the page's
`<meta name="generator">`, read by a pre-pass before anything a profile extracts, picks the profile, else `generic`
(Docsy and Sphinx emit none, I12); a declared profile's `generator` regex claims it, else its base's detection does
(D94), the file's profiles tried in file order before the compiled ones. `pages.profile` keeps the choice. Rejected:
deciding once per root, which mislabels a root spanning generators; config only, which makes every Antora user opt in.
Code: `profile::Registry::choose`, `extract::generator`, `cli::registry_for`.

### D36 The page → source tie-back is core: an edit-link heuristic and forge URL shapes
Every generator's edit link points at a forge, so any site gets page → file with no config. A link earning no tier
(`extract::edit_link`) is none, so a plain repository link never wins by being the only one; a forge shape parses on any
host, its ref one segment, which a source rule overrides (D93). docs.fp.o needs no override (I8). Rejected: discovery
in the Antora profile only, which leaves routing a breakage to its repo (D17) to one generator; a `title` match in
`antora`, a docs.fp.o quirk in a generator profile (D32). Code: `extract::edit_link`, `forge::parse`.

### D93 A profile's source rules are tried in file order before an edit link is parsed, each a regex with templates over its captures
A `[[profile.source]]` is `from = "url" | "edit_link"` and a regex whose captures `repo`, `ref` and `path`, or
`${name}` templates over them, give a source; the first rule giving a repo and a path wins, and `forge::parse` reads the
edit link only where none does. Why: a site with no edit link has only its URL, which names neither the repo nor the
file's extension (Sphinx's `/en/<ref>/<page>.html`), and the parse cannot tell a ref holding `/` from the path without
a forge call per repo (I23). Rejected: captures alone, stating no literal; a rule per root, a profile being what D35
chooses by; recorded rows keyed by a rule's `name`, not its place, which is its meaning. Reopens: a site mixing
formats under one profile, the first match too coarse. Code: `forge::rule`, `config::profile`. Evidence:
`forge::rule::tests`.

### D94 A declarative profile may extend one compiled profile, adding its rules to all the base does
`extends = "antora" | "generic"` wraps the base: its region replaces the base's where it states one, its `generator`
the base's detection, and a report reads the base off the run's `profile.<name>.extends` row for its sitemap note.
Facts: the base's keep its namespace (`antora.*`), the extension's own are `<name>.*`; `pages.profile` is the
extension's name. Why: a deployment then states only what is its own, and `group_by` and the base's views are unchanged
(D32, D33). Rejected: an extension of a declared profile; the base's facts renamed under the extension; the
extension's in the base's namespace, read by name (D54, D56) and colliding with compiled facts. Reopens: a second level
wanted. Code: `profile::declarative`, `cli::registry_for`, `report::digest::sections`. Evidence:
`profile::declarative::tests`, `report::digest::tests`.

### D17 Each page stores its source file and repo, read from its edit link
This routes a breakage to the repo and file that own it, and groups more precisely than component (some repos supply
several). A CI build's pages carry none (I15), so CI maps a build by its component-versions (D54, D56). Finding the link
is D36; docs.fp.o's markup is I8. Code: `pages.source_*` (`store::schema`).

### D37 Profile facts live in one core table, `page_facts`, under namespaced keys
`(run_id, url_id, key, value)`, keys like `antora.component`; a profile's views pivot its keys into columns, and D19
resolves through `Lookup` over the table. D33's rules can then write only a key and a value. Rejected: a typed table
per profile, a migration story each; a JSON column, no typed joins and no simpler.

### D38 A profile has four hooks: detect, extract, resolve, views
Detect claims a generator string (D35). Extract adds a content region (D34), handlers, source rules (D93) and a finish
step to the core's one lol_html pass (D11), writing only to a `Sink`. Resolve maps a tagged reference to a URL through
the store's `Lookup` (D19, D20, D22). Views is SQL (D37), one of which may be the profile's hints (D87). Beside them,
`sitemap_note` gives the digest a sentence (D87). `generic` leaves Resolve and Views empty, having no reference that is
not a URL and no facts. Rejected: region handlers in each profile, each re-deciding `unknown` (D34), which only a page's
end settles; a profile driving the pass (D39). Reopens: a hook empty in `generic` for any other reason, the trait
bending around Antora. Code: `profile::Profile`, `profile::Sink`.

### D19 Pages carry Antora resource IDs, and unresolved refspecs resolve through them
Antora records `(component, version, module, rel_path)` as facts (D37): module and `rel_path` from the source path (I8,
I26, D93); component and version verbatim from the nav container, or on a page with none the component from the URL
under the site root (I13) and no version. A refspec with no version takes the only or latest version among the pages
holding its module and path, else the component's latest (KD18). Rejected: the module from the URL, ambiguous with
subdirectories; no component without a nav container, leaving xrefs into it unresolvable; a version from the URL, no
versioned component lacking one; the fallback keyed on a nav container lacking `data-component`, inert where that
happens (I12). Code: `profile::antora`, `profile::antora::resolve`.

### D65 An unresolved xref's hint is the nearest page ID by edit distance, within three tenths of the longer, read in the page's own context
The refspec is completed as Antora read it on its page, so a component-less one finds its page, then compared with the
run's `component:module:rel_path` IDs. Why: Levenshtein finds typos, missing directories and moved modules, where
Jaro-Winkler weighs the shared prefix. Of the 2026-10-23 crawl's 92, bounds 0.2 to 0.4 by 0.05 hint 31, 41, 47, 49 and
83 links, 25, 30, 34, 36 and 48 the page meant (one reader). Rejected: a tighter bound, losing the `:w` typos; 0.35,
two more right but one edit from ten `install-guide` refspecs wrongly hinted at 11/31; 0.4, whose 36 more hold 14
right; a hint's version, which the build may lack (KD10). Reopens: wrong hints often on another site; Resolve taking
over the refspec grammar (I21). Code: `profile::antora::hints`. Evidence:
`report::tests::an_unresolved_xref_names_the_nearest_page_id_within_the_distance`.

### D87 The reports read a profile's generator knowledge through the profile: `link_hints` for a hint, `sitemap_note` for the sitemap's sentence
`link_hints` unions each profile's `<profile>_hints` view, empty where none ships one, for `report_broken` to join;
`Profile::sitemap_note` is a stable key and its English text, the digest's note above the sitemap entries no page was
fetched for: a view in another language words its own sentence by the key. Why: a core view naming `antora_hints`
failed to create without Antora, and the digest said Antora of every site; neither is core's to know (D32). Rejected:
a fifth hook for hints; the note as a fact or a view; the default view holding each profile's sentence. Code:
`store::views::link_hints`, `report::digest::sections::sitemap_notes`. Evidence:
`report::tests::a_registry_whose_profiles_ship_no_hints_still_has_the_broken_view`,
`report::digest::tests::the_sitemap_note_is_what_the_profiles_of_the_crawled_pages_say`.
