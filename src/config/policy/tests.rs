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
    assert_eq!(defaults.len(), 1 + 12 + 1 + 3);
    for row in [
        "/pedantic_trailing_slash=true",
        "/schemes=mailto",
        "/schemes=javascript",
        "/doubled_slash=true",
        "/status.403=error",
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
        ("[judge]\nunknown = 1", "unknown"),
    ] {
        let err = parse(extra).unwrap_err();
        assert!(err.contains(named), "{extra}: {err}");
    }
}
