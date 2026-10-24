use super::super::tests::{of, store};
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
    assert!(fix.contains("Targets named by one link each: 2."));
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
    assert!(pages[one..two].contains("<b>3 broken links</b>"));
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
    assert!(errors.contains("https://e.test/12"));
    assert!(errors.contains("server_error"));
    assert!(!errors.contains("https://e.test/10"));
}

#[test]
fn the_housekeeping_is_collapsed_and_holds_each_list() {
    let html = html();
    let house = section(&html, "housekeeping");
    assert!(house.contains("<details class=\"house\">") && !house.contains("open"));
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
    let d = crate::report::digest(&store, Some(RunId(3))).unwrap();
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
    let html = crate::report::digest(&store, None).unwrap().html();
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
    assert!(html.contains(
        "<b>1 broken link</b> <small>(1 to a broken URL, 0 to a missing fragment; of 3 links on the page)"
    ));
    assert!(html.contains("<td>fragment missing</td>"));
    assert!(html.contains("<td>URL broken</td>"));
    assert!(!html.contains("anchor missing") && !html.contains("Broken anchors"));
    assert_eq!(super::counted(1, "broken link"), "1 broken link");
    assert_eq!(super::counted(1_000, "link"), "1,000 links");
    assert_eq!(
        super::broken_counts(1, 2, 3),
        "1 to a broken URL, 2 to a missing fragment, 3 errored"
    );
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
