use super::*;
use crate::config::Config;

fn parse(extra: &str) -> Result<Policy, String> {
    Config::parse(&format!("[[root]]\nurl = \"https://a.example/\"\n{extra}"))
        .map(|c| c.policy)
        .map_err(|e| e.to_string())
}

/// The rows as `host/key=value`.
fn rows(policy: &Policy) -> Vec<String> {
    policy
        .entries()
        .iter()
        .map(|e| format!("{}/{}={}", e.host.as_deref().unwrap_or(""), e.key, e.value))
        .collect()
}

#[test]
fn the_default_policy_is_what_bruce_has_always_judged_by() {
    let policy = Policy::default();
    assert!(policy.pedantic_trailing_slash && policy.doubled_slash);
    assert_eq!(policy.schemes, WORKING_SCHEMES);
    assert_eq!(
        policy.status.keys().copied().collect::<Vec<_>>(),
        [401, 403, 429]
    );
    assert!(policy.status.values().all(|a| *a == Answer::Error));
    assert!(policy.hosts.is_empty());
}

#[test]
fn every_setting_is_an_entry_with_its_value_written_out() {
    let mut policy = Policy::default();
    let defaults = rows(&policy);
    assert_eq!(defaults.len(), 1 + 12 + 1 + 1 + 3 + 5 + 1 + 1 + 6);
    for row in [
        "/pedantic_trailing_slash=true",
        "/schemes=mailto",
        "/schemes=javascript",
        "/doubled_slash=true",
        "/doubled_slash_reach=full",
        "/status.403=error",
        "/gate.fail=broken_url,broken_anchor",
        "/digest.group_by=",
        "/timeout=30",
        "/retries=3",
        "/slash_probe=true",
        "/max_body_mib=16",
    ] {
        assert!(defaults.contains(&row.to_string()), "{row}");
    }
    policy.pedantic_trailing_slash = false;
    assert!(rows(&policy).contains(&"/pedantic_trailing_slash=false".to_string()));
}

#[test]
fn a_file_sets_the_judging_keys_over_the_defaults() {
    let policy = parse(
        r#"
        [judge]
        pedantic_trailing_slash = false
        doubled_slash_reach = "immediate"
        schemes = ["Matrix", "mailto", "web+x"]

        [default]
        doubled_slash = false
        status = { 403 = "ok", 500 = "broken_url" }

        [host."Pagure.io"]
        doubled_slash = true
        status = { 403 = "broken_url", 404 = "error" }

        [host."quiet.example"]
        per_second = 1
        "#,
    )
    .unwrap();
    assert!(!policy.pedantic_trailing_slash && !policy.doubled_slash);
    assert_eq!(policy.doubled_slash_reach, Reach::Immediate);
    assert!(rows(&policy).contains(&"/doubled_slash_reach=immediate".to_string()));
    assert_eq!(policy.schemes.len(), 14);
    assert_eq!(&policy.schemes[12..], ["matrix", "web+x"]);
    // The file's codes are laid over the built-in ones, per code.
    assert_eq!(policy.status[&401], Answer::Error);
    assert_eq!(policy.status[&403], Answer::Ok);
    assert_eq!(policy.status[&500], Answer::BrokenUrl);
    // Only a host that sets a judging key is recorded, by the host's form.
    assert_eq!(policy.hosts.keys().collect::<Vec<_>>(), ["pagure.io"]);
    let rows = rows(&policy);
    for row in [
        "/status.403=ok",
        "pagure.io/doubled_slash=true",
        "pagure.io/status.403=broken_url",
        "pagure.io/status.404=error",
    ] {
        assert!(rows.contains(&row.to_string()), "{row}");
    }
}

#[test]
fn a_judging_key_that_cannot_mean_anything_is_refused_naming_it() {
    for (extra, named) in [
        ("[default]\nstatus = { 302 = \"ok\" }", "default.status"),
        ("[default]\nstatus = { 99 = \"ok\" }", "default.status"),
        ("[default]\nstatus = { four = \"ok\" }", "default.status"),
        ("[default]\nstatus = { 404 = \"fine\" }", "default.status"),
        (
            "[default]\nstatus = { 404 = \"unchecked\" }",
            "default.status",
        ),
        (
            "[host.\"a.example\"]\nstatus = { 4xx = \"ok\" }",
            "host.\"a.example\".status",
        ),
        ("[judge]\nschemes = [\"http\"]", "judge.schemes"),
        ("[judge]\nschemes = [\"1x\"]", "judge.schemes"),
        ("[judge]\nschemes = [\"a b\"]", "judge.schemes"),
        (
            "[judge]\ndoubled_slash_reach = \"deep\"",
            "judge.doubled_slash_reach",
        ),
        ("[judge]\nunknown = 1", "unknown"),
    ] {
        let err = parse(extra).unwrap_err();
        assert!(err.contains(named), "{extra}: {err}");
    }
}

#[test]
fn a_file_moves_a_kind_to_another_level_and_a_reason_beats_its_outcome() {
    let policy = parse(
        r#"
        [severity]
        broken_anchor = "pedantic"
        doubled_slash = "attention"
        pedantic_trailing_slash = "none"
        "#,
    )
    .unwrap();
    let map = &policy.severity;
    assert_eq!(map.of_link(Some("broken_anchor"), None), Severity::Pedantic);
    // The reason's level is that kind of `broken_url`'s alone.
    assert_eq!(
        map.of_link(Some("broken_url"), Some("doubled_slash")),
        Severity::Attention
    );
    assert_eq!(map.of_link(Some("broken_url"), None), Severity::Broken);
    assert_eq!(
        map.of_redirect(Some("pedantic_trailing_slash")),
        Severity::None
    );
    assert_eq!(map.of_redirect(None), Severity::Attention);
    // The rows record the whole map, and a map read back from them is it.
    let rows = policy.entries();
    assert!(
        rows.iter()
            .any(|e| e.key == "severity.doubled_slash" && e.value == "attention")
    );
    assert!(
        rows.iter()
            .any(|e| e.key == "severity.error" && e.value == "broken")
    );
    assert_eq!(SeverityMap::from_entries(&rows), *map);
    // Rows a run recorded before the map existed read as the default.
    assert_eq!(SeverityMap::from_entries(&[]), SeverityMap::default());
}

#[test]
fn every_reason_but_an_unchecked_links_takes_a_level() {
    // Every reason a level may be set for is accepted, and none of the
    // `unchecked` ones is.
    for reason in Reason::ALL {
        let parsed = parse(&format!("[severity]\n{} = \"none\"", reason.as_str()));
        assert_eq!(parsed.is_ok(), reason.takes_level(), "{reason:?}");
    }
    let policy = parse("[severity]\nunreadable = \"attention\"\nnot_built = \"none\"").unwrap();
    assert_eq!(
        policy.severity.of_link(Some("error"), Some("unreadable")),
        Severity::Attention
    );
    assert_eq!(
        policy
            .severity
            .of_link(Some("broken_url"), Some("not_built")),
        Severity::None
    );
}

#[test]
fn a_severity_key_that_cannot_mean_anything_is_refused_naming_it() {
    for (extra, named) in [
        (
            "[severity]\nunchecked = \"none\"",
            "unchecked keeps no level",
        ),
        ("[severity]\nok = \"none\"", "\"ok\" is not"),
        (
            "[severity]\nout_of_scope = \"none\"",
            "\"out_of_scope\" is not",
        ),
        (
            "[severity]\nbroken_url = \"red\"",
            "broken_url: \"red\" is not a level",
        ),
    ] {
        let err = parse(extra).unwrap_err();
        assert!(
            err.contains("severity") && err.contains(named),
            "{extra}: {err}"
        );
    }
}

#[test]
fn a_setting_is_written_as_the_file_writes_it() {
    let written = |host: Option<&str>, key: &str, value: &str| {
        Entry {
            host: host.map(str::to_owned),
            key: key.to_owned(),
            value: value.to_owned(),
        }
        .written_as()
    };
    assert_eq!(
        written(None, "pedantic_trailing_slash", "false"),
        "judge.pedantic_trailing_slash = false"
    );
    assert_eq!(
        written(None, "schemes", "matrix"),
        "judge.schemes += \"matrix\""
    );
    assert_eq!(
        written(None, "doubled_slash_reach", "off"),
        "judge.doubled_slash_reach = \"off\""
    );
    assert_eq!(
        written(None, "status.403", "ok"),
        "default.status.403 = \"ok\""
    );
    assert_eq!(
        written(None, "doubled_slash", "false"),
        "default.doubled_slash = false"
    );
    assert_eq!(
        written(None, "severity.error", "none"),
        "severity.error = \"none\""
    );
    assert_eq!(
        written(Some("pagure.io"), "status.403", "broken_url"),
        "host.\"pagure.io\".status.403 = \"broken_url\""
    );
}

#[test]
fn a_file_names_the_fact_the_digest_groups_by_and_it_is_one_row() {
    let policy = parse("[digest]\ngroup_by = \"antora.component\"").unwrap();
    assert_eq!(policy.group_by.as_deref(), Some("antora.component"));
    let row = policy
        .entries()
        .into_iter()
        .find(|e| e.key == "digest.group_by")
        .unwrap();
    assert_eq!(row.value, "antora.component");
    assert_eq!(row.written_as(), "digest.group_by = \"antora.component\"");
    assert_eq!(group_by_from_row(Some(&row.value)), policy.group_by);
    // The default groups by nothing, and its row is empty.
    let none = parse("").unwrap();
    assert_eq!(none.group_by, None);
    assert!(rows(&none).contains(&"/digest.group_by=".to_owned()));
    assert_eq!(group_by_from_row(Some("")), None);
    assert_eq!(group_by_from_row(None), None);
}

#[test]
fn a_group_by_that_is_no_fact_key_is_refused_at_load() {
    for bad in [
        "",
        "component",
        ".component",
        "antora.",
        "antora..x",
        "an tora.x",
    ] {
        let err = parse(&format!("[digest]\ngroup_by = {bad:?}")).unwrap_err();
        assert!(err.contains("digest.group_by"), "{bad}: {err}");
    }
    assert!(parse("[digest]\nsort = 1").is_err());
}

#[test]
fn a_file_names_the_outcomes_the_gate_fails_on_and_it_is_one_row() {
    let policy = parse("[gate]\nfail = [\"error\", \"broken_url\", \"error\"]").unwrap();
    assert_eq!(policy.gate_fail, [FailOn::BrokenUrl, FailOn::Error]);
    let row = policy
        .entries()
        .into_iter()
        .find(|e| e.key == "gate.fail")
        .unwrap();
    assert_eq!(row.value, "broken_url,error");
    assert_eq!(row.written_as(), "gate.fail = [\"broken_url\", \"error\"]");
    assert_eq!(FailOn::from_row(Some(&row.value)), policy.gate_fail);
    // A run with no row is at the default, and a name this version lacks is skipped.
    assert_eq!(FailOn::from_row(None), DEFAULT_FAIL);
    assert_eq!(
        FailOn::from_row(Some("broken_anchor,warning")),
        [FailOn::BrokenAnchor]
    );
    assert_eq!(parse("").unwrap().gate_fail, DEFAULT_FAIL);
}

#[test]
fn an_empty_fail_list_is_report_only_and_is_recorded_as_an_empty_row() {
    let policy = parse("[gate]\nfail = []").unwrap();
    assert!(policy.gate_fail.is_empty());
    let row = policy
        .entries()
        .into_iter()
        .find(|e| e.key == "gate.fail")
        .unwrap();
    assert_eq!(row.value, "");
    assert_eq!(row.written_as(), "gate.fail = []");
    // An empty row is a list naming nothing, not a run with no row.
    assert!(FailOn::from_row(Some("")).is_empty());
}

#[test]
fn a_fail_list_that_cannot_mean_anything_is_refused_naming_it() {
    for (extra, named) in [
        (
            "[gate]\nfail = [\"redirected\"]",
            "\"redirected\" is not an outcome",
        ),
        ("[gate]\nfail = [\"unchecked\"]", "gate.fail"),
        ("[gate]\nwarn = [\"error\"]", "warn"),
    ] {
        let err = parse(extra).unwrap_err();
        assert!(err.contains(named), "{extra}: {err}");
    }
}

#[test]
fn a_file_sets_how_a_crawl_asks_over_the_defaults_and_a_host_over_both() {
    let policy = Config::parse(
        r#"
        max_body_mib = 64
        [default]
        timeout = 60
        retries = 0
        [host."Pagure.io"]
        connect_timeout = 5
        slash_probe = false
        "#,
    )
    .unwrap()
    .policy;
    let asking = &policy.asking;
    assert_eq!(
        (asking.connect_timeout, asking.timeout, asking.retries),
        (10, 60, 0)
    );
    assert_eq!((asking.max_retry_after, asking.max_body_mib), (60, 64));
    assert_eq!(asking.max_body(), 64 * 1024 * 1024);
    assert!(asking.probes("docs.example.org") && !asking.probes("pagure.io"));
    let written = |key: &str, host: Option<&str>| {
        policy
            .entries()
            .into_iter()
            .find(|e| e.key == key && e.host.as_deref() == host)
            .map(|e| (e.written_as(), e.is_asking()))
    };
    assert_eq!(
        written("timeout", None),
        Some(("default.timeout = 60".to_owned(), true))
    );
    assert_eq!(
        written("max_body_mib", None),
        Some(("max_body_mib = 64".to_owned(), true))
    );
    assert_eq!(
        written("connect_timeout", Some("pagure.io")),
        Some(("host.\"pagure.io\".connect_timeout = 5".to_owned(), true))
    );
    assert_eq!(
        written("slash_probe", Some("pagure.io")),
        Some(("host.\"pagure.io\".slash_probe = false".to_owned(), true))
    );
    // A host's row exists only for a key it names.
    assert_eq!(written("timeout", Some("pagure.io")), None);
    assert!(!Policy::default().entries().iter().any(|e| e.host.is_some()));
}

#[test]
fn a_judging_row_is_not_an_asking_one() {
    for row in Policy::default().entries() {
        assert_eq!(
            row.is_asking(),
            ASKING_KEYS.contains(&row.key.as_str()),
            "{}",
            row.key
        );
    }
    assert!(!Policy::default().entries().iter().all(Entry::is_asking));
}

#[test]
fn an_asking_key_that_cannot_mean_anything_is_refused_naming_it() {
    for (extra, named) in [
        ("[default]\ntimeout = 0", "default.timeout"),
        ("[default]\nconnect_timeout = 0", "default.connect_timeout"),
        ("[default]\nretries = 11", "default.retries"),
        (
            "[host.\"a.example\"]\ntimeout = 0",
            "host.\"a.example\".timeout",
        ),
        (
            "[host.\"a.example\"]\nretries = 99",
            "host.\"a.example\".retries",
        ),
        ("[default]\nretries = -1", "retries"),
    ] {
        let err = parse(extra).unwrap_err();
        assert!(err.contains(named), "{extra}: {err}");
    }
    let err = Config::parse("max_body_mib = 0").unwrap_err().to_string();
    assert!(err.contains("max_body_mib"), "{err}");
    // Zero is a ceiling that waits out only a Retry-After of 0, and no retries.
    let policy = parse("[default]\nretries = 0\nmax_retry_after = 0").unwrap();
    assert_eq!(
        (policy.asking.retries, policy.asking.max_retry_after),
        (0, 0)
    );
}
