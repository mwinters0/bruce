use super::*;

#[test]
fn every_reason_names_itself_once_and_parses_back() {
    let mut seen = std::collections::BTreeSet::new();
    for reason in Reason::ALL {
        assert!(seen.insert(reason.as_str()), "{reason:?} repeats a name");
        assert_eq!(Reason::parse(reason.as_str()), Some(*reason));
    }
    assert_eq!(Reason::parse("dns"), None);
}

#[test]
fn the_reasons_a_severity_takes_are_the_ones_a_broken_or_errored_link_carries() {
    let levelled: Vec<&str> = Reason::levelled().map(Reason::as_str).collect();
    assert_eq!(
        levelled,
        [
            "nxdomain",
            "timeout",
            "tls",
            "connect",
            "reset",
            "other",
            "unreadable",
            "redirect_loop",
            "too_many_redirects",
            "bad_redirect",
            "too_large",
            "unresolved_xref",
            "not_built",
            "bad_scheme",
            "invalid_url",
            "doubled_slash",
            "wrong_trailing_slash",
            "pedantic_trailing_slash",
        ]
    );
}

#[test]
fn a_statement_names_a_reason_by_its_variant() {
    assert_eq!(
        expand("reason IN ($reason(scheme), $reason(bad_scheme)) AND x"),
        "reason IN ('scheme', 'bad_scheme') AND x"
    );
    assert_eq!(expand("no reason here"), "no reason here");
}

#[test]
#[should_panic(expected = "\"dns\" is no reason")]
fn a_statement_naming_an_unknown_reason_panics() {
    expand("reason = $reason(dns)");
}

/// The manual lists what `[severity]` accepts and what each `unchecked`
/// reason means, so a reason added to the enum is added there.
#[test]
fn the_manual_names_every_reason_where_it_belongs() {
    let config = include_str!("../../../../docs/manual/config.md");
    let digest = include_str!("../../../../docs/manual/digest.md");
    let after = config
        .split("A reason not listed has no level of its own")
        .nth(1)
        .and_then(|rest| rest.split("\n\n## ").next())
        .expect("config.md lists the reasons a severity takes");
    for reason in Reason::levelled() {
        let named = format!("`{}`", reason.as_str());
        let default = reason == Reason::PedanticTrailingSlash;
        assert!(
            after.contains(&named) || default,
            "{reason:?} is missing from config.md's list of reasons"
        );
    }
    let table = digest
        .split("## Why a link is unchecked")
        .nth(1)
        .and_then(|rest| rest.split("\n## ").next())
        .expect("digest.md explains the unchecked reasons");
    for reason in Reason::ALL.iter().filter(|r| !r.takes_level()) {
        let row = format!("| `{}` |", reason.as_str());
        assert!(table.contains(&row), "{reason:?} has no row in digest.md");
    }
}
