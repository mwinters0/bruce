use super::*;

fn url(text: &str) -> Url {
    Url::parse(text).unwrap()
}

fn rule(
    over: Over,
    pattern: &str,
    repo: Option<&str>,
    git_ref: Option<&str>,
    path: Option<&str>,
) -> Rule {
    Rule::new(over, pattern, repo, git_ref, path).unwrap()
}

fn refused(pattern: &str, repo: Option<&str>, git_ref: Option<&str>, path: Option<&str>) -> String {
    Rule::new(Over::Url, pattern, repo, git_ref, path).unwrap_err()
}

/// A source as `(repo, ref, path)`.
fn shape(source: Option<Source>) -> Option<(String, Option<String>, String)> {
    source.map(|s| (s.repo, s.git_ref, s.path))
}

const SPHINX: &str = r"^https://www\.sphinx-doc\.org/en/(?P<ref>[^/]+)/(?P<page>.+)\.html$";

fn sphinx() -> Rule {
    rule(
        Over::Url,
        SPHINX,
        Some("https://github.com/sphinx-doc/sphinx"),
        None,
        Some("doc/${page}.rst"),
    )
}

#[test]
fn a_url_rule_states_what_the_url_does_not_by_templates() {
    let page = url("https://www.sphinx-doc.org/en/master/usage/quickstart.html");
    assert_eq!(
        shape(source(&[sphinx()], &page, None)),
        Some((
            "https://github.com/sphinx-doc/sphinx".into(),
            Some("master".into()),
            "doc/usage/quickstart.rst".into()
        ))
    );
}

#[test]
fn a_template_left_out_is_the_capture_of_its_own_name() {
    let rule = rule(
        Over::Url,
        r"^https://(?P<repo>docs\.example\.org)/(?P<path>.+)$",
        None,
        None,
        None,
    );
    // The captured `repo` is a bare host, no repo URL: the rule is passed over.
    assert_eq!(rule.apply(&url("https://docs.example.org/a/b.md")), None);
    let rule = self::rule(
        Over::Url,
        r"^https://example\.org/(?P<repo>https://github\.com/o/r)/(?P<ref>[^/]+)/(?P<path>.+)$",
        None,
        None,
        None,
    );
    let mut page = url("https://example.org/");
    page.set_path("/https://github.com/o/r/dev/a/b.md");
    assert_eq!(
        shape(rule.apply(&page)),
        Some((
            "https://github.com/o/r".into(),
            Some("dev".into()),
            "a/b.md".into()
        ))
    );
}

#[test]
fn a_rule_without_a_ref_gives_none() {
    let rule = rule(
        Over::Url,
        r"^https://example\.org/(?P<path>.+)$",
        Some("https://github.com/o/r"),
        None,
        None,
    );
    assert_eq!(
        shape(rule.apply(&url("https://example.org/a.md"))),
        Some(("https://github.com/o/r".into(), None, "a.md".into()))
    );
}

#[test]
fn an_edit_link_rule_reads_a_ref_holding_a_slash() {
    // The parse reads the ref `feature` and the path `x/docs/a.md`.
    let edit = url("https://github.com/o/r/edit/feature/x/docs/a.md");
    let rule = rule(
        Over::EditLink,
        r"^(?P<repo>https://github\.com/o/r)/edit/(?P<ref>feature/x)/(?P<path>.+)$",
        None,
        None,
        None,
    );
    let page = url("https://example.org/a.html");
    assert_eq!(
        shape(source(&[rule], &page, Some(&edit))),
        Some((
            "https://github.com/o/r".into(),
            Some("feature/x".into()),
            "docs/a.md".into()
        ))
    );
    assert_eq!(
        shape(source(&[], &page, Some(&edit))),
        Some((
            "https://github.com/o/r".into(),
            Some("feature".into()),
            "x/docs/a.md".into()
        )),
        "with no rule the parse reads it"
    );
}

#[test]
fn an_edit_link_rule_remedies_a_repo_named_for_a_marker() {
    // The parse reads the repo `https://github.com/o` and the ref `blob`.
    let edit = url("https://github.com/o/edit/blob/main/a.md");
    let rule = rule(
        Over::EditLink,
        r"^(?P<repo>https://github\.com/o/edit)/blob/(?P<ref>[^/]+)/(?P<path>.+)$",
        None,
        None,
        None,
    );
    let page = url("https://example.org/a.html");
    assert_eq!(
        shape(source(&[rule], &page, Some(&edit))),
        Some((
            "https://github.com/o/edit".into(),
            Some("main".into()),
            "a.md".into()
        ))
    );
}

#[test]
fn the_first_rule_that_gives_a_source_wins_and_the_parse_comes_last() {
    let edit = url("https://github.com/o/r/blob/main/a.md");
    let page = url("https://www.sphinx-doc.org/en/master/a.html");
    let other = rule(
        Over::Url,
        r"^https://elsewhere\.org/(?P<path>.+)$",
        Some("https://github.com/x/y"),
        None,
        None,
    );
    let rules = [other, sphinx()];
    assert_eq!(
        shape(source(&rules, &page, Some(&edit))),
        Some((
            "https://github.com/sphinx-doc/sphinx".into(),
            Some("master".into()),
            "doc/a.rst".into()
        )),
        "a matching rule beats the edit link"
    );
    let elsewhere = url("https://example.org/a.html");
    assert_eq!(
        shape(source(&rules, &elsewhere, Some(&edit))),
        Some((
            "https://github.com/o/r".into(),
            Some("main".into()),
            "a.md".into()
        )),
        "no rule matches, so the edit link is parsed"
    );
    assert_eq!(source(&rules, &elsewhere, None), None);
}

#[test]
fn an_edit_link_rule_passes_over_a_page_without_an_edit_link() {
    let rule = rule(
        Over::EditLink,
        r"^(?P<repo>https://h\.org/r)/(?P<path>.+)$",
        None,
        None,
        None,
    );
    assert_eq!(source(&[rule], &url("https://h.org/r/a.md"), None), None);
}

#[test]
fn an_expansion_that_is_no_source_is_passed_over() {
    let page = url("https://example.org/a/");
    for path in ["", "${dir}/"] {
        let rule = rule(
            Over::Url,
            r"^https://example\.org/(?P<dir>.*)$",
            Some("https://github.com/o/r"),
            None,
            Some(path),
        );
        assert_eq!(rule.apply(&page), None, "{path:?}");
    }
    for repo in [
        "not-a-url",
        "ftp://h.org/o/r",
        "https://h.org",
        "https://h.org/o/r?x=1",
    ] {
        let rule = rule(
            Over::Url,
            r"^https://example\.org/\?r=(?P<repo>.*)$",
            Some("${repo}"),
            None,
            Some("a.md"),
        );
        let page = url(&format!("https://example.org/?r={repo}"));
        assert_eq!(rule.apply(&page), None, "{repo:?}");
    }
}

#[test]
fn a_repo_is_kept_in_the_form_a_parse_gives() {
    let rule = rule(
        Over::Url,
        r"^https://example\.org/",
        Some("https://user:pw@GitHub.com/o/r/"),
        None,
        Some("a.md"),
    );
    assert_eq!(
        shape(rule.apply(&url("https://example.org/x"))),
        Some(("https://github.com/o/r".into(), None, "a.md".into()))
    );
}

#[test]
fn what_makes_no_rule_is_refused_by_name() {
    let c = Some;
    for (error, expect) in [
        (
            refused("(", c("https://h.org/o/r"), None, c("a")),
            "pattern:",
        ),
        (
            refused("^x$", c("https://h.org/o/r"), None, None),
            "path: the pattern has no capture named `path`",
        ),
        (
            refused("^x$", None, None, c("a")),
            "repo: the pattern has no capture named `repo`",
        ),
        (
            refused("^(?P<p>.)$", c("https://h.org/o/r"), None, c("${q}")),
            "path: \"${q}\" names `q`",
        ),
        (
            refused("^(.)$", c("https://h.org/o/r"), None, c("$2")),
            "names `2`",
        ),
        (
            refused(
                "^(?P<p>.)$",
                c("https://h.org/o/r"),
                c("${nope}"),
                c("${p}"),
            ),
            "ref: \"${nope}\" names `nope`",
        ),
        (
            refused("^x$", c("${unclosed"), None, c("a")),
            "never closes",
        ),
        (
            refused("^x$", c("github.com/o/r"), None, c("a")),
            "repo: \"github.com/o/r\" is not an http(s) URL",
        ),
        (
            refused("^x$", c("https://github.com"), None, c("a")),
            "repo:",
        ),
        (
            refused("^(?P<r>.)$", c("git@github.com:${r}"), None, c("a")),
            "does not begin as an http(s) URL does",
        ),
    ] {
        assert!(error.contains(expect), "{expect}: {error}");
    }
    assert!(
        Over::parse("page")
            .unwrap_err()
            .contains("`url` or `edit_link`")
    );
}

#[test]
fn a_template_may_escape_a_dollar_and_name_a_capture_by_number() {
    let rule = rule(
        Over::Url,
        r"^https://example\.org/(.+)\.html$",
        Some("https://github.com/o/r"),
        None,
        Some("$1.rst$$"),
    );
    assert_eq!(
        shape(rule.apply(&url("https://example.org/a/b.html"))),
        Some(("https://github.com/o/r".into(), None, "a/b.rst$".into()))
    );
}
