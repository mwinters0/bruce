use super::super::tests::{compared, of, ranked, store, with_doubled_slashes, with_unchecked};
use crate::report::table::Cell;
use crate::report::tests::{registry, xrefs};
use crate::store::RunId;

/// The text of `html` from the section with `id` to its end tag.
fn section<'a>(html: &'a str, id: &str) -> &'a str {
    let from = html
        .find(&format!("<section id=\"{id}\""))
        .unwrap_or_else(|| panic!("no section {id}"));
    let to = html[from..].find("</section>").unwrap();
    &html[from..from + to]
}

fn html() -> String {
    of(&store()).html()
}

/// Every `name="..."` attribute value in `html`.
fn attributes<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let key = format!(" {name}=\"");
    html.match_indices(&key)
        .map(|(at, _)| {
            let rest = &html[at + key.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect()
}

#[test]
fn the_document_needs_no_script_and_no_outside_file() {
    let html = html();
    for banned in [
        "<script", "<link", "<iframe", " src=", "@import", "url(", "<img",
    ] {
        assert!(!html.contains(banned), "{banned}");
    }
    assert_eq!(html.matches("<style>").count(), 1);
    assert!(html.starts_with("<!doctype html>"));
}

#[test]
fn every_link_is_absolute_or_in_the_page() {
    let html = html();
    let hrefs = attributes(&html, "href");
    assert!(hrefs.len() > 20, "{}", hrefs.len());
    for href in hrefs {
        assert!(
            href.starts_with('#') || href.starts_with("https://"),
            "{href}"
        );
    }
}

#[test]
fn the_title_carries_the_date_and_the_times_are_utc() {
    let html = html();
    assert!(
        html.contains("<title>Link check digest 2026-10-23 \u{2013} e.test/ and 1 more</title>")
    );
    assert!(
        html.contains("<time datetime=\"2026-10-23T20:04:11Z\">2026-10-23 20:04:11 UTC</time>")
    );
    assert!(
        html.contains("<time datetime=\"2026-10-23T21:14:44Z\">2026-10-23 21:14:44 UTC</time>")
    );
    assert!(html.contains(">1 h 10 min 33 s<"));
    assert!(html.contains("no earlier run"));
}

#[test]
fn the_figures_are_the_totals() {
    let figures = section(&html(), "figures").to_owned();
    for (n, label) in [
        (5, "Pages checked"),
        (10, "Links on them"),
        (3, "Links to a broken URL"),
        (1, "Links to a missing fragment"),
        (1, "Links that errored"),
    ] {
        assert!(
            figures.contains(&format!(
                "<span class=\"n\">{n}</span><span class=\"l\">{label}</span>"
            )),
            "{label}"
        );
    }
    assert!(figures.contains("The 4 broken links are those to a broken URL"));
}

#[test]
fn fix_once_lists_the_targets_more_than_one_link_names_and_counts_the_rest() {
    let html = html();
    let fix = section(&html, "fix-once");
    assert!(fix.contains("https://e.test/10"));
    assert!(!fix.contains("https://e.test/11"), "named by one link");
    assert!(fix.contains("named by more than one link: 1, holding 2 of the 4 broken links."));
    assert!(fix.contains(
        "Targets named by one link each: 2. Each is in the drill-down of the page its link is on, \
         among the pages above."
    ));
}

#[test]
fn pages_follow_their_rank_each_with_its_links_and_sources() {
    let html = html();
    let pages = section(&html, "pages");
    let one = pages.find("<details id=\"p1\">").unwrap();
    let two = pages.find("<details id=\"p2\">").unwrap();
    assert!(one < two && !pages.contains("id=\"p3\""));
    assert!(pages[one..two].contains("Page 1"));
    assert!(pages[one..two].contains("<code>1&gt;10#</code>"));
    assert!(pages[one..two].contains("<code>1&gt;2#nope</code>"));
    assert!(pages[one..two].contains("href=\"https://forge.test/r/one/1.adoc\""));
    assert!(pages[two..].contains("<code>2&gt;10#</code>"));
    assert!(pages[one..two].contains(
        "<span class=\"cn\"><span class=\"sev broken\" role=\"img\" aria-label=\"Broken\"></span> \
         3 broken links</span>, <span class=\"cn\"><span class=\"sev attention\" role=\"img\" \
         aria-label=\"Attention\"></span> 1 missing fragment</span></span></summary>"
    ));
}

#[test]
fn a_summary_counts_red_then_amber_and_leaves_out_a_level_with_no_links() {
    let tally = super::tally;
    let red =
        "<span class=\"cn\"><span class=\"sev broken\" role=\"img\" aria-label=\"Broken\"></span>";
    let amber = "<span class=\"cn\"><span class=\"sev attention\" role=\"img\" aria-label=\"Attention\"></span>";
    assert_eq!(
        tally(181, 0, 2),
        format!("{red} 181 broken links</span>, {amber} 2 missing fragments</span>")
    );
    assert_eq!(tally(1, 0, 0), format!("{red} 1 broken link</span>"));
    assert_eq!(tally(0, 0, 1), format!("{amber} 1 missing fragment</span>"));
    // The links broken through a link are red, between the two.
    assert_eq!(
        tally(1, 579, 0),
        format!("{red} 1 broken link</span>, {red} 579 broken through it</span>")
    );
}

#[test]
fn a_page_lists_its_red_links_before_its_amber_ones_an_errored_link_among_the_red() {
    let store = ranked();
    let html = of(&store).html();
    let pages = section(&html, "pages");
    let three = pages.find("<details id=\"p2\">").unwrap();
    let four = pages.find("<details id=\"p3\">").unwrap();
    // Page 3 holds two errors and nothing broken: its rows are errors.
    let rows = &pages[three..four];
    assert_eq!(
        rows.matches("<td>error \u{b7} server_error</td>").count(),
        2
    );
    assert!(rows.contains("2 broken links</span></span></summary>"));
    // Page 4: a broken URL, then the fragment.
    let rows = &pages[four..];
    let url = rows.find("<td>URL broken</td>").unwrap();
    let fragment = rows.find("<td>fragment missing</td>").unwrap();
    assert!(url < fragment);
    // The repo of page 3 and 4 (none) counts both, errors among its red.
    let repos = section(&html, "repos");
    assert!(repos.contains("No source recorded"));
    assert!(repos.contains("3 broken links</span>, "), "{repos}");
}

#[test]
fn a_repo_links_to_its_pages_rows() {
    let html = html();
    let repos = section(&html, "repos");
    assert!(
        repos.contains("<span class=\"ttl\">r/one</span>"),
        "a repo that is no URL is text"
    );
    assert!(repos.contains("<a href=\"#p1\">Page 1</a>"));
    assert!(repos.contains("<a href=\"#p2\">Page 2</a>"));
    // The pages with no source are the row with no repo, and hold no breakage.
    assert!(repos.contains("more repos' pages hold no broken link"));
}

#[test]
fn errors_are_apart_from_the_breakage() {
    let html = html();
    let errors = section(&html, "errors");
    assert!(errors.contains("https://slow.test/12"));
    assert!(errors.contains("server_error"));
    assert!(!errors.contains("https://e.test/10"));
}

#[test]
fn the_sections_after_the_figures_are_pages_repos_targets_errors_housekeeping() {
    let html = html();
    let order = [
        "figures",
        "pages",
        "repos",
        "fix-once",
        "anchors-removed",
        "errors",
        "housekeeping",
    ];
    let at: Vec<usize> = order
        .iter()
        .map(|id| html.find(&format!("<section id=\"{id}\">")).unwrap())
        .collect();
    assert!(at.is_sorted(), "{at:?}");
    assert_eq!(html.matches("<section ").count(), order.len());
    // The nav names the sections in the order they stand.
    let nav = &html[html.find("<nav>").unwrap()..html.find("</nav>").unwrap()];
    let hrefs: Vec<&str> = attributes(nav, "href");
    let ids: Vec<String> = order.iter().map(|id| format!("#{id}")).collect();
    assert_eq!(hrefs, ids);
}

#[test]
fn every_section_after_the_figures_starts_collapsed_under_its_heading() {
    let html = html();
    let figures = section(&html, "figures");
    assert!(figures.contains("<h2>Figures</h2>") && !figures.contains("<details"));
    for id in [
        "pages",
        "repos",
        "fix-once",
        "anchors-removed",
        "errors",
        "housekeeping",
    ] {
        // The id is outside the `<details>`, so a link to it lands on the
        // heading where a browser opens no closed `<details>` for a fragment.
        let s = section(&html, id);
        assert!(
            s.starts_with(&format!(
                "<section id=\"{id}\">\n<details class=\"sect\">\n<summary>"
            )) && s.contains("<h2>"),
            "{id}"
        );
        assert!(s.ends_with("</details>\n"), "{id}");
    }
    assert!(!html.contains(" open"), "no `<details>` starts open");
}

#[test]
fn the_housekeeping_is_collapsed_and_holds_each_list() {
    let html = html();
    let house = section(&html, "housekeeping");
    assert!(house.contains("<h2>Housekeeping</h2><small>Redirects to update"));
    for needle in [
        "https://e.test/2",
        "https://e.test/20",
        "robots",
        "Read no sitemap",
        "https://other.test/",
        "skipped",
    ] {
        assert!(house.contains(needle), "{needle}");
    }
    assert!(house.contains("Orphans") && house.contains("Unlisted pages"));
}

#[test]
fn text_a_page_wrote_is_escaped_and_only_http_urls_are_links() {
    let mut d = of(&store());
    let at = d
        .broken
        .columns
        .iter()
        .position(|c| c == "raw_href")
        .unwrap();
    d.broken.rows[0][at] = Cell::Text("<script>alert(\"x\")</script>".into());
    let to = d.broken.columns.iter().position(|c| c == "target").unwrap();
    d.broken.rows[1][to] = Cell::Text("javascript:alert(1)".into());
    d.pages.rows[0][1] = Cell::Text("A <b>title</b> & more".into());
    let html = d.html();
    assert!(!html.contains("<script"));
    assert!(html.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;"));
    assert!(!html.contains("href=\"javascript"));
    assert!(html.contains("A &lt;b&gt;title&lt;/b&gt; &amp; more"));
}

#[test]
fn an_unfinished_run_says_so() {
    let store = store();
    let d =
        crate::report::digest(&store, Some(RunId(3)), crate::report::Against::Previous).unwrap();
    let html = d.html();
    assert!(html.contains("This run never finished"));
    assert!(html.contains("never finished</dd>"));
    assert!(!of(&store).html().contains("This run never finished"));
}

#[test]
fn a_run_with_nothing_to_report_says_so_in_each_section() {
    let store = crate::store::Store::open_in_memory(&registry()).unwrap();
    store.install(crate::report::VIEWS).unwrap();
    let id = store
        .begin_run(crate::store::Mode::Crawl, "h", None, &[])
        .unwrap();
    store.finish_run(id).unwrap();
    let html = crate::report::digest(&store, None, crate::report::Against::Previous)
        .unwrap()
        .html();
    for said in [
        "No broken target is named by more than one link.",
        "No page holds a broken link.",
        "No repo's pages hold a broken link.",
        "No link errored.",
    ] {
        assert!(html.contains(said), "{said}");
    }
}

#[test]
fn numbers_have_thousands_separators() {
    assert_eq!(super::num(0), "0");
    assert_eq!(super::num(999), "999");
    assert_eq!(super::num(1_000), "1,000");
    assert_eq!(super::num(586_175), "586,175");
    assert_eq!(super::num(-1_234_567), "-1,234,567");
    assert_eq!(super::duration(59), "59 s");
    assert_eq!(super::duration(125), "2 min 5 s");
}

#[test]
fn a_fold_counts_in_the_unit_of_its_title_and_an_empty_list_is_no_table() {
    let html = html();
    let house = section(&html, "housekeeping");
    // The unchecked reasons are rows; the figures count their links.
    let unchecked: i64 = rows_of(&of(&store()).unchecked, "links");
    assert!(unchecked > 0);
    assert!(house.contains(&format!(
        "Unchecked links</span><span class=\"cnt\">{unchecked} links<"
    )));
    assert!(!html.contains("<tbody>\n</tbody>"));

    let mut d = of(&store());
    d.orphans.rows.clear();
    let html = d.html();
    let orphans = &html[html.find("Pages the sitemap lists").unwrap()..];
    assert!(orphans.starts_with(
        "Pages the sitemap lists that no page links to.</p>\n<p class=\"muted\">None.</p>"
    ));
}

fn rows_of(t: &crate::report::table::Table, column: &str) -> i64 {
    let at = t.columns.iter().position(|c| c == column).unwrap();
    t.rows
        .iter()
        .map(|r| match r[at] {
            Cell::Int(n) => n,
            _ => 0,
        })
        .sum()
}

#[test]
fn anchor_names_only_the_element_and_every_count_says_which_way_it_points() {
    // `anchor` is the Kind column's `<a href>`; an absent fragment is a
    // fragment everywhere, and a count of links says to what or on what.
    let html = html();
    assert!(html.contains("1 broken link</span></span></summary>"));
    assert!(html.contains("<td>fragment missing</td>"));
    assert!(html.contains("<td>URL broken</td>"));
    assert!(!html.contains("anchor missing") && !html.contains("Broken anchors"));
    assert_eq!(super::counted(1, "broken link"), "1 broken link");
    assert_eq!(super::counted(1_000, "link"), "1,000 links");
    for header in [
        "Links to it",
        "Linking pages",
        "Target's source",
        "Written as",
        "Resolves to",
        "On page",
    ] {
        assert!(html.contains(&format!(">{header}</th>")), "{header}");
    }
}

#[test]
fn a_page_whose_links_hold_a_hint_shows_a_hint_column_and_no_other_does() {
    let hinted = of(&xrefs(&["install/gide.adoc", "nothing-like-any-page.adoc"])).html();
    assert_eq!(hinted.matches("<th>Hint</th>").count(), 1);
    assert!(hinted.contains("<td><code>hb::install/guide.adoc</code></td>"));
    assert!(hinted.contains("<td>-</td>"), "a link with none says so");
    let plain = of(&xrefs(&["nothing-like-any-page.adoc"])).html();
    assert!(!plain.contains("<th>Hint</th>"));
    assert!(!html().contains("<th>Hint</th>"));
}

#[test]
fn every_reason_the_crawler_writes_for_an_unchecked_link_is_explained() {
    // The frontier's (`Frontier::plan`, `SKIPPED`, `EXTERNALS_SKIPPED`), the
    // walk's robots.txt and sitemap depth (`crawl`), and the verdict's own
    // (`link_verdicts`).
    for reason in [
        "scheme",
        "skipped",
        "out_of_scope",
        "host_not_allowlisted",
        "externals_skipped",
        "robots",
        "robots_unavailable",
        "too_deep",
        "cut",
        "text_fragment",
        "no_anchors",
    ] {
        assert!(super::unchecked_meaning(reason).is_some(), "{reason}");
    }
    assert!(super::unchecked_meaning("broken").is_none());
}

#[test]
fn each_unchecked_reason_is_explained_beside_where_its_links_point_most() {
    let mut d = of(&with_unchecked());
    // Seven more hosts for `host_not_allowlisted`, past the five listed.
    let row = d.unchecked_groups.rows[3].clone();
    for n in 0..7 {
        let mut more = row.clone();
        more[1] = Cell::Text(format!("h{n}.test"));
        more[2] = Cell::Int(1);
        d.unchecked_groups.rows.insert(5, more);
    }
    let html = d.html();
    let house = section(&html, "housekeeping");
    let at = |needle: &str| house.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    let oos = at(
        "<h3><code>out_of_scope</code> <small>5 links, to 4 targets</small></h3>\n<p>Links to the site's pages in other languages",
    );
    assert!(house[..oos].contains("They are not counted as broken"));
    assert!(house[..oos].contains("a link it knows cannot work is listed as broken instead"));
    let hosts = at("<h3><code>host_not_allowlisted</code>");
    let skipped = at("<h3><code>skipped</code>");
    let scheme = at("<h3><code>scheme</code>");
    assert!(house[skipped..scheme].contains("<h3><code>no_anchors</code>"));
    let checked = at("<h3>External hosts checked</h3>");
    assert!(oos < hosts && hosts < skipped && skipped < scheme && scheme < checked);
    // The setting that would check each.
    assert!(house[oos..hosts].contains("<code>[[root]]</code>"));
    assert!(house[hosts..skipped].contains("add it to the <code>allowlist</code>"));
    assert!(house[skipped..scheme].contains("match a <code>skip</code> pattern"));
    assert!(house[oos..hosts].contains("<td>https://e.test/de/</td><td class=\"num\">3</td>"));
    assert!(house[hosts..skipped].contains("<td>github.test</td>"));
    assert!(house[hosts..skipped].contains("And 4 more, holding 4 links, none more than 1."));
    assert!(!house[oos..hosts].contains("more, holding"));
    assert!(house[scheme..checked].contains("<td>mailto:</td>"));
    assert!(house[scheme..checked].contains("cannot work for a reader and is listed as broken"));
    assert!(house[checked..].contains("<td>forge.test</td><td class=\"num\">2</td>"));
    assert!(house[checked..].contains("<td>slow.test</td>"));
}

#[test]
fn a_level_is_read_off_a_links_outcome_and_reason() {
    use super::severity::Level;
    let pedantic = Some(crate::store::PEDANTIC_TRAILING_SLASH);
    assert_eq!(
        Level::of_link(Some("broken_url"), None),
        Some(Level::Broken)
    );
    assert_eq!(
        Level::of_link(Some("error"), Some("timeout")),
        Some(Level::Broken)
    );
    assert_eq!(
        Level::of_link(Some("broken_anchor"), pedantic),
        Some(Level::Attention)
    );
    assert_eq!(Level::of_link(Some("ok"), pedantic), Some(Level::Pedantic));
    assert_eq!(Level::of_link(Some("ok"), None), None);
    assert_eq!(Level::of_link(Some("unchecked"), Some("robots")), None);
    assert_eq!(Level::of_redirect(pedantic), Level::Pedantic);
    assert_eq!(Level::of_redirect(None), Level::Attention);
}

#[test]
fn a_heading_carries_no_dot_and_each_dot_is_named() {
    let html = html();
    for id in ["pages", "repos", "fix-once", "errors", "housekeeping"] {
        let s = section(&html, id);
        let summary = &s[s.find("<summary>").unwrap()..s.find("</summary>").unwrap()];
        assert!(!summary.contains("sev"), "{id}");
    }
    assert!(html.contains("<span class=\"sev broken\" role=\"img\" aria-label=\"Broken\"></span>"));
    assert!(
        html.contains(
            "<span class=\"sev attention\" role=\"img\" aria-label=\"Attention\"></span>"
        )
    );

    let mut d = of(&store());
    let reason = d
        .redirects
        .columns
        .iter()
        .position(|c| c == "reason")
        .unwrap();
    d.redirects.rows[0][reason] = Cell::Text(crate::store::PEDANTIC_TRAILING_SLASH.into());
    assert!(section(&d.html(), "housekeeping").contains(
        "<summary><span class=\"sev pedantic\" role=\"img\" aria-label=\"Pedantic\"></span><span class=\"ttl\">"
    ));
}

#[test]
fn every_row_naming_a_link_or_a_target_opens_with_a_dot_under_no_heading() {
    let html = html();
    let dot = "<td class=\"dot\"><span class=\"sev ";
    // Targets (1 row), the page drill-downs (4, the errored link among them, and 1),
    // errors (1). A redirected link's landing is a `<details>` whose summary
    // opens with the dot.
    for (id, rows) in [("fix-once", 1), ("pages", 5), ("errors", 1)] {
        let s = section(&html, id);
        assert_eq!(s.matches(dot).count(), rows, "{id}");
        assert!(s.contains("<th class=\"dot\"></th>"), "{id}");
    }
    assert!(section(&html, "fix-once").contains(&format!("{dot}broken\"")));
    assert!(section(&html, "errors").contains(&format!("{dot}broken\"")));
    // The lists that are not links have none: orphans, unlisted pages, sitemap
    // entries and the unchecked counts.
    let house = section(&html, "housekeeping");
    assert!(house.contains("<summary><span class=\"sev attention\""));
    assert_eq!(house.matches("<span class=\"sev ").count(), 1);
    assert_eq!(house.matches("<th class=\"dot\">").count(), 0);
}

#[test]
fn one_legend_beside_the_figures_names_the_three_levels() {
    let html = html();
    let figures = section(&html, "figures");
    assert_eq!(html.matches("class=\"legend\"").count(), 1);
    assert!(figures.contains("class=\"legend\""));
    for level in ["broken", "attention", "pedantic"] {
        assert!(
            figures.contains(&format!(
                "<span class=\"sev {level}\" aria-hidden=\"true\">"
            )),
            "{level}"
        );
    }
    for name in ["<b>Broken</b>", "<b>Attention</b>", "<b>Pedantic</b>"] {
        assert!(figures.contains(name), "{name}");
    }
}

#[test]
fn each_redirected_landing_is_a_details_with_its_hops_and_pages_inside() {
    let d = of(&store());
    let html = d.html();
    let house = section(&html, "housekeeping");
    let from = house
        .find("<summary><span class=\"sev attention\"")
        .unwrap()
        - "<details>\n".len();
    let landing = &house[from..from + house[from..].find("</details>").unwrap()];
    let landing_url = crate::report::tests::url(2);
    assert!(landing.starts_with("<details>\n<summary>"));
    assert!(landing.contains("<span class=\"ttl\">e.test/2</span>"));
    assert!(landing.contains("<span class=\"cnt\">1 link, 1 redirecting URL</span>"));
    assert!(landing.contains(&format!("<p>Landing <a href=\"{landing_url}\">")));
    assert!(landing.contains("<p>First hops <code>"));
    assert!(landing.contains("<p>Linking pages: 1</p>\n<ul>\n<li><a href=\""));
    // No table holds the landings, so no cell grows as wide as a list of pages.
    assert!(
        !house[..from + house[from..].find("</details>").unwrap()]
            .contains("<th>Linking pages</th>")
    );
    assert!(!house.contains("First hops</th>"));
}

#[test]
fn a_doubled_slash_is_marked_in_the_path_alone() {
    let slashes = super::slashes;
    assert_eq!(
        slashes("https://fedoraproject.org//"),
        "https://fedoraproject.org<mark>//</mark>"
    );
    assert_eq!(slashes("e.test/a///b/"), "e.test/a<mark>///</mark>b/");
    // A scheme's `//`, a protocol-relative start, a query and a fragment
    // double none.
    for none in [
        "https://web.archive.org/web/1/https://e.test/",
        "//e.test/a",
        "e.test/a?b=//c",
        "e.test/a#//",
    ] {
        assert_eq!(slashes(none), none);
    }
    assert_eq!(slashes("<b>//c"), "&lt;b&gt;<mark>//</mark>c");
}

#[test]
fn a_doubled_slash_link_shows_where_its_slash_doubles() {
    let html = of(&with_doubled_slashes()).html();
    let pages = section(&html, "pages");
    // Page 7's `cc` lands on `cc//`: the landing is under the URL it names.
    assert!(pages.contains(
        "<td><a href=\"https://e.test/cc\">e.test/cc</a><br><small>\u{2192} lands on \
         <a href=\"https://e.test/cc//\">e.test/cc<mark>//</mark></a></small></td>"
    ));
    // A link not broken by a doubled slash is shown as written.
    assert!(pages.contains("<td><code>https://e.test/abs</code></td>"));
    assert!(!pages.contains("e.test/abs</a><br>"));

    let mut d = of(&with_doubled_slashes());
    let col = |name: &str| d.broken.columns.iter().position(|c| c == name).unwrap();
    let (raw, target, landing) = (col("raw_href"), col("target"), col("landing"));
    let at = d
        .broken
        .rows
        .iter()
        .position(|r| r[target] == Cell::Text("https://e.test/cc".into()))
        .unwrap();
    d.broken.rows[at][raw] = Cell::Text("https://e.test/x//y".into());
    d.broken.rows[at][target] = Cell::Text("https://e.test/x//y".into());
    d.broken.rows[at][landing] = Cell::Null;
    let html = d.html();
    let pages = section(&html, "pages");
    assert!(pages.contains(
        "<td><code>https://e.test/x<mark>//</mark>y</code></td>\
         <td><a href=\"https://e.test/x//y\">e.test/x<mark>//</mark>y</a></td>"
    ));
}

#[test]
fn a_redirect_onto_a_wrong_trailing_slash_shows_the_landing_with_its_slash_marked() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE targets SET url = 'https://e.test/old', final_url = 'https://e.test/new/', \
             reason = 'wrong_trailing_slash' WHERE url_id = 10",
            [],
        )
        .unwrap();
    let html = of(&store).html();
    for id in ["pages", "fix-once"] {
        assert!(
            section(&html, id).contains(
                "<a href=\"https://e.test/old\">e.test/old</a>\
                 <br><small>\u{2192} lands on <a href=\"https://e.test/new/\">e.test/new<mark>/</mark></a></small>\
                 <br><small>\u{2192} try <a href=\"https://e.test/new\">e.test/new</a></small>"
            ),
            "{id}"
        );
    }
}

#[test]
fn a_wrong_trailing_slash_link_names_the_url_that_answers() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE targets SET reason = 'wrong_trailing_slash' WHERE url_id = 10",
            [],
        )
        .unwrap();
    let html = of(&store).html();
    // The drill-down and the fix-once table both put the fix under the target.
    for id in ["pages", "fix-once"] {
        assert!(
            section(&html, id).contains(
                "<a href=\"https://e.test/10\">e.test/10</a><br><small>\u{2192} try \
                 <a href=\"https://e.test/10/\">e.test/10/</a></small>"
            ),
            "{id}"
        );
    }
    // A target broken for another reason is shown as it is.
    assert!(!section(&html, "pages").contains("e.test/11</a><br>"));
}

/// Run 1's digest compared with run 4.
fn compared_digest() -> crate::report::Digest {
    crate::report::digest(
        &compared(),
        Some(RunId(1)),
        crate::report::Against::Run(RunId(4)),
    )
    .unwrap()
}

fn compared_html() -> String {
    compared_digest().html()
}

#[test]
fn the_header_names_the_run_compared_with_and_each_figure_its_change() {
    let html = compared_html();
    assert!(html.contains(
        "<dt>Compared with</dt><dd>run 4, started <time datetime=\"2026-10-22T20:04:11Z\">"
    ));
    let figures = section(&html, "figures");
    // Run 4 held 2 pages and 1 link, 1 of them to a broken URL.
    for (n, label, change) in [
        (5, "Pages checked", "<span class=\"d\">+3</span>"),
        (10, "Links on them", "<span class=\"d\">+9</span>"),
        (
            3,
            "Links to a broken URL",
            "<span class=\"d worse\">+2</span>",
        ),
        (
            1,
            "Links to a missing fragment",
            "<span class=\"d worse\">+1</span>",
        ),
        (1, "Links that errored", "<span class=\"d worse\">+1</span>"),
    ] {
        assert!(
            figures.contains(&format!(
                "<span class=\"n\">{n}</span><span class=\"l\">{label}</span>{change}"
            )),
            "{label}"
        );
    }
    assert!(figures.contains("Each figure's change is since run 4."));
}

#[test]
fn a_fall_in_what_is_broken_is_for_the_better_and_no_change_is_said() {
    let mut d = compared_digest();
    let c = d.compared.as_mut().unwrap();
    c.totals.broken_url = 5;
    c.totals.pages = 5;
    let figures = d.html();
    let figures = section(&figures, "figures");
    assert!(figures.contains("<span class=\"d better\">\u{2212}2</span>"));
    assert!(figures.contains("<span class=\"d\">no change</span>"));
}

#[test]
fn the_anchors_removed_are_a_section_of_pages_with_their_dots() {
    let html = compared_html();
    let s = section(&html, "anchors-removed");
    assert!(s.contains("<code>#nope</code>"));
    assert!(s.contains("https://e.test/2"));
    assert_eq!(
        s.matches("<td class=\"dot\"><span class=\"sev attention\"")
            .count(),
        1
    );
    assert!(s.contains("<th class=\"dot\"></th>"));
}

#[test]
fn with_nothing_to_compare_with_the_section_and_the_header_say_so() {
    let html = html();
    assert!(html.contains("<dt>Compared with</dt><dd>no earlier run</dd>"));
    assert!(section(&html, "anchors-removed").contains("no earlier run to compare with"));
    assert!(
        !html.contains("<span class=\"d\"") && !html.contains("<span class=\"d "),
        "no change without a comparison"
    );
}

#[test]
fn a_baseline_files_run_is_named_for_its_file_and_an_unfinished_one_says_so() {
    let mut d = compared_digest();
    let c = d.compared.as_mut().unwrap();
    c.file = true;
    c.run.finished = None;
    let html = d.html();
    assert!(html.contains("<dd>the run of a baseline file, started"));
    assert!(html.contains("never finished, so its figures are incomplete"));
    assert!(html.contains("since the run of the baseline file."));
    assert!(section(&html, "anchors-removed").contains("the baseline file's run"));
}
