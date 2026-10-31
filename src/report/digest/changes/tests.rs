use super::*;

fn row(key: &str, value: &str) -> Entry {
    Entry {
        host: None,
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

/// The rows of `site`'s source rules, each `(from, pattern)`.
fn rules(rules: &[(&str, &str)]) -> Vec<Entry> {
    rules
        .iter()
        .enumerate()
        .flat_map(|(i, (from, pattern))| {
            [
                row(&format!("profile.site.source.{i}.from"), from),
                row(&format!("profile.site.source.{i}.pattern"), pattern),
            ]
        })
        .collect()
}

fn used(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

/// Each change as `-<key> = <value>`, `+...`, or `<from key> ~ <to key>: <from> -> <to>`.
fn shown(changes: &[Change]) -> Vec<String> {
    changes
        .iter()
        .map(|c| match (&c.from, &c.to) {
            (Some(a), Some(b)) => format!("{} ~ {}: {} -> {}", a.key, b.key, a.value, b.value),
            (Some(a), None) => format!("-{} = {}", a.key, a.value),
            (None, Some(b)) => format!("+{} = {}", b.key, b.value),
            (None, None) => unreachable!(),
        })
        .collect()
}

#[test]
fn equal_rows_differ_in_nothing() {
    let rows = [row("timeout", "30"), row("profile.site.generator", "g")];
    assert!(differences(&rows, &rows, &used(&["site"])).is_empty());
}

#[test]
fn a_row_one_run_lacks_is_added_or_removed_and_a_changed_value_is_one_change() {
    let was = [row("timeout", "30"), row("max_body_mib", "8")];
    let now = [
        row("timeout", "60"),
        row("digest.group_by", "antora.component"),
    ];
    assert_eq!(
        shown(&differences(&was, &now, &used(&[]))),
        [
            "+digest.group_by = antora.component",
            "-max_body_mib = 8",
            "timeout ~ timeout: 30 -> 60",
        ]
    );
}

#[test]
fn a_key_with_several_values_lists_those_lost_and_gained_apart() {
    let was = [row("schemes", "a"), row("schemes", "b")];
    let now = [
        row("schemes", "a"),
        row("schemes", "c"),
        row("schemes", "d"),
    ];
    assert_eq!(
        shown(&differences(&was, &now, &used(&[]))),
        ["-schemes = b", "+schemes = c", "+schemes = d"]
    );
}

#[test]
fn a_profile_no_page_of_either_run_used_is_not_compared() {
    let was = [row("profile.site.generator", "a")];
    let now = [row("profile.site.generator", "b"), row("timeout", "9")];
    assert_eq!(shown(&differences(&was, &now, &used(&["antora"]))).len(), 1);
    assert_eq!(shown(&differences(&was, &now, &used(&["site"]))).len(), 2);
}

#[test]
fn a_rule_inserted_is_named_alone_and_the_rules_after_it_are_not() {
    let was = rules(&[("url", "a"), ("url", "b")]);
    let now = rules(&[("url", "a"), ("edit_link", "new"), ("url", "b")]);
    assert_eq!(
        shown(&differences(&was, &now, &used(&["site"]))),
        [
            "+profile.site.source.1.from = edit_link",
            "+profile.site.source.1.pattern = new"
        ]
    );
}

#[test]
fn a_rule_removed_is_named_alone() {
    let was = rules(&[("url", "a"), ("url", "b"), ("url", "c")]);
    let now = rules(&[("url", "a"), ("url", "c")]);
    assert_eq!(
        shown(&differences(&was, &now, &used(&["site"]))),
        [
            "-profile.site.source.1.from = url",
            "-profile.site.source.1.pattern = b"
        ]
    );
}

#[test]
fn rules_removed_and_added_in_one_gap_are_paired_in_order_naming_the_fields_changed() {
    let was = rules(&[("url", "a"), ("url", "b"), ("url", "z")]);
    let now = rules(&[("url", "a"), ("url", "b2"), ("url", "z")]);
    assert_eq!(
        shown(&differences(&was, &now, &used(&["site"]))),
        ["profile.site.source.1.pattern ~ profile.site.source.1.pattern: b -> b2"]
    );
    // A rule moved to another place by an insertion before it keeps both places.
    let was = rules(&[("url", "b")]);
    let now = rules(&[("url", "a"), ("url", "b2")]);
    assert_eq!(
        shown(&differences(&was, &now, &used(&["site"]))),
        [
            "profile.site.source.0.pattern ~ profile.site.source.0.pattern: b -> a",
            "+profile.site.source.1.from = url",
            "+profile.site.source.1.pattern = b2",
        ]
    );
}

#[test]
fn a_template_a_rule_gains_is_an_added_field_of_the_same_rule() {
    let was = rules(&[("url", "a")]);
    let mut now = rules(&[("url", "a")]);
    now.push(row("profile.site.source.0.repo", "https://forge.test/r"));
    assert_eq!(
        shown(&differences(&was, &now, &used(&["site"]))),
        ["+profile.site.source.0.repo = https://forge.test/r"]
    );
}

#[test]
fn rules_are_listed_in_the_order_of_their_places() {
    let now: Vec<Entry> = (0..11)
        .map(|i| row(&format!("profile.site.source.{i}.from"), "url"))
        .collect();
    let listed = shown(&differences(&[], &now, &used(&["site"])));
    assert_eq!(listed.len(), 11);
    assert!(listed[2].contains("source.2."));
    assert!(listed[10].contains("source.10."));
}
