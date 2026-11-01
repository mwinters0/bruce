//! The two litmus tests of P11, read from the fixtures the e2e site renders:
//! a view entirely in Czech, and a CSV view (`fixtures/antora/views/`). They
//! show what a view can do with the model alone, so no language and no format
//! is in the crate.

use std::path::PathBuf;

use minijinja::value::Value;

use super::super::document::Digest;
use super::super::tests::{
    compared, compared_with_a_fix, of, ranked, store, with_doubled_slashes, with_unchecked,
};
use super::{ViewSource, Views, model};
use crate::profile::SitemapNote;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/antora/views")
        .join(name)
}

fn views(name: &str) -> Views {
    Views::load(&[ViewSource::File(fixture(name))]).unwrap()
}

fn rendered(name: &str, digest: &Digest) -> String {
    views(name)
        .render(digest)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .text
}

/// What the Czech view's macro `name` prints for `args`.
fn call(name: &str, args: &[Value]) -> String {
    let views = views("cs.html");
    let view = &views.0[0];
    let mut out = view
        .env
        .get_template(&view.entry)
        .unwrap()
        .render_captured(model::context(&of(&store())))
        .unwrap();
    out.with_state_mut(|state| state.call_macro(name, args))
        .unwrap()
}

/// The digests the tests build, between them taking every branch of a view.
fn digests() -> Vec<Digest> {
    let mut trial = of(&compared());
    trial.trial = Some("trial.toml".to_owned());
    let mut unfinished = of(&store());
    unfinished.unfinished = true;
    vec![
        of(&store()),
        of(&ranked()),
        of(&compared()),
        of(&compared_with_a_fix()),
        of(&with_unchecked()),
        of(&with_doubled_slashes()),
        trial,
        unfinished,
    ]
}

#[test]
fn the_czech_view_renders_every_digest_and_holds_no_english() {
    // The words of the default view that no Czech sentence, URL or code holds.
    let english = [
        "Figures",
        "Housekeeping",
        "Orphans",
        "Unlisted",
        "Roots",
        "How to read",
        "Written as",
        "Resolves to",
        "Redirected links",
        "Unchecked links",
        "broken link",
        "None.",
        "No earlier run",
        "never finished",
        "Compared with",
        "Links to a",
        "lands on",
        "Settings",
    ];
    for digest in digests() {
        let html = rendered("cs.html", &digest);
        assert!(html.contains("<html lang=\"cs\">"));
        assert!(html.contains("Přehled kontroly odkazů"));
        for word in english {
            assert!(!html.contains(word), "{word}");
        }
    }
}

#[test]
fn czech_counts_take_the_form_of_their_number() {
    let said = |n: i64| {
        call(
            "counted",
            &[n.into(), "odkaz".into(), "odkazy".into(), "odkazů".into()],
        )
    };
    assert_eq!(said(1), "1 odkaz");
    for n in [2, 3, 4] {
        assert_eq!(said(n), format!("{n} odkazy"));
    }
    // 22 is not a few: only 2 to 4 are.
    for n in [0, 5, 11, 12, 22, 24, 100] {
        assert_eq!(said(n), format!("{n} odkazů"));
    }
    assert_eq!(said(1234), "1\u{a0}234 odkazů");
}

#[test]
fn czech_dates_and_decimals_are_the_views_own() {
    // 2026-10-23T20:04:11Z, the start of the test store's run.
    assert_eq!(call("cz_day", &[1_792_785_851_i64.into()]), "23. 10. 2026");
    assert_eq!(call("cz_day", &[1_793_491_200_i64.into()]), "1. 11. 2026");
    let percent = |part: i64, whole: i64| call("percent", &[part.into(), whole.into()]);
    assert_eq!(percent(1, 8), "12,5\u{a0}%");
    assert_eq!(percent(1, 3), "33,3\u{a0}%");
    assert_eq!(percent(0, 7), "0,0\u{a0}%");
    // A share rounds to the nearest tenth, a half up: 2/3 is 66,7 and 1/16 is 6,3, neither truncated.
    assert_eq!(percent(2, 3), "66,7\u{a0}%");
    assert_eq!(percent(1, 16), "6,3\u{a0}%");
}

#[test]
fn czech_keys_a_profiles_sitemap_note_and_falls_back_to_its_text() {
    let store = store();
    store
        .connection()
        .execute(
            "UPDATE pages SET profile = 'antora' WHERE url_id IN (1, 2)",
            [],
        )
        .unwrap();
    let digest = of(&store);
    assert_eq!(digest.sitemap_notes[0].key, "antora.sitemap_from_build");
    let html = rendered("cs.html", &digest);
    assert!(html.contains("Antora zapisuje mapu webu"));
    assert!(!html.contains(digest.sitemap_notes[0].text));
    let mut other = of(&store);
    other.sitemap_notes = vec![SitemapNote {
        key: "docusaurus.sitemap",
        text: "Docusaurus writes it.",
    }];
    assert!(rendered("cs.html", &other).contains("Docusaurus writes it."));
}

/// The records of RFC 4180 text: fields quoted or not, a quoted one holding
/// commas, doubled quotes and line ends.
fn records(text: &str) -> Vec<Vec<String>> {
    let (mut out, mut record, mut field) = (Vec::new(), Vec::new(), String::new());
    let mut chars = text.chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match (quoted, c) {
            (true, '"') if chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            (true, '"') => quoted = false,
            (true, c) => field.push(c),
            (false, '"') => quoted = true,
            (false, ',') => record.push(std::mem::take(&mut field)),
            (false, '\n') => {
                record.push(std::mem::take(&mut field));
                out.push(std::mem::take(&mut record));
            }
            (false, c) => field.push(c),
        }
    }
    assert!(!quoted, "a quote is left open");
    out
}

#[test]
fn the_csv_view_writes_one_record_to_each_finding_whatever_the_values_hold() {
    let store = ranked();
    store
        .connection()
        .execute(
            "UPDATE pages SET title = 'One, \"two\"\nthree' WHERE url_id = 1",
            [],
        )
        .unwrap();
    let digest = of(&store);
    let text = rendered("links.csv", &digest);
    let records = records(&text);
    let header = &records[0];
    assert_eq!(header[0], "page");
    assert_eq!(header.len(), 13);
    let findings: usize = model::pages(&digest).iter().map(|p| p.findings.len()).sum();
    assert!(findings > 5);
    assert_eq!(records.len(), 1 + findings);
    for record in &records {
        assert_eq!(record.len(), header.len(), "{record:?}");
    }
    assert!(
        records
            .iter()
            .any(|r| r[1] == "One, \"two\"\nthree" && r[8] == "broken_url")
    );
    assert!(records.iter().any(|r| r[8] == "error" && r[10] == "503"));
    // Every value is quoted, whatever it is; a missing one is empty.
    assert!(text.contains("\"404\""), "{text}");
    assert!(text.lines().nth(1).unwrap().starts_with("\"http"), "{text}");
}

#[test]
fn the_csv_view_renders_every_digest() {
    for digest in digests() {
        let records = records(&rendered("links.csv", &digest));
        assert!(records.iter().all(|r| r.len() == 13));
    }
}

#[test]
fn the_manual_names_exactly_the_top_level_names_of_the_model() {
    let manual = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/manual/digest.md"),
    )
    .unwrap();
    let section = manual
        .split("## The model a view reads")
        .nth(1)
        .unwrap()
        .split("\n## ")
        .next()
        .unwrap();
    let mut named: Vec<&str> = section
        .lines()
        .filter(|l| l.starts_with("| `"))
        .flat_map(|l| {
            let cell = l.trim_start_matches("| ").split(" | ").next().unwrap();
            cell.split(", ").map(|n| n.trim_matches('`'))
        })
        .collect();
    let model = model::context(&of(&store()));
    let mut held: Vec<String> = model
        .try_iter()
        .unwrap()
        .map(|key| key.to_string())
        .collect();
    named.sort_unstable();
    held.sort_unstable();
    assert_eq!(named, held);
}
