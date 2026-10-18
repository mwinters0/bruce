# P3.6.1 Sitemap files as links: notes

What P3.7 onward inherits, and what was refused.

- **`Kind::SitemapFile`** (`sitemap_file`) is the second kind no page
  yields. `links.kind` has no `CHECK`, so the schema stays at version 2
  (D43). A sitemap's `Item::kind` is now `SitemapFile`.
- **`Shared::discover` collects each root's links and sends them once
  discovery ends, from the blocking pool** (D28), the probes' own targets
  having gone first. A link's source is the root's canonical URL, whose
  target row is written in the second pass like any seed's.
- **One link per root per file, by `url_id`**: robots.txt naming a file
  twice gives one link, and two roots on a host give one each though the
  file is read once (`each_root_links_once_to_each_sitemap_it_read`). A probe
  another root already claimed is linked too, without a request
  (`a_probe_another_root_found_is_linked_from_both`).
- **`raw_href`** is the robots.txt line as written, or the probe as
  written (`sitemap.xml`, `/sitemap.xml`). For a set `sitemap` it is the
  parsed URL's serialisation: `Config` keeps no raw text, and the two differ
  only where `url` normalises.
- **A link targets the URL named, not where it redirected**; the target's
  `final_url` says where. A moved index's children link from where it
  landed, as its entries already did in P3.6.
- **An index's child is `sitemap_file` whatever became of it**: read,
  `too_deep`, `scheme` or `invalid_url`. `sitemap` is left to an entry of a
  `<urlset>`.
- `Summary` is unchanged; `sitemaps` still counts files read.
