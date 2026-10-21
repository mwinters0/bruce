use super::*;

fn parsed(url: &str) -> Option<(String, String, String)> {
    parse(&Url::parse(url).unwrap()).map(|s| (s.repo, s.git_ref, s.path))
}

fn shape(repo: &str, git_ref: &str, path: &str) -> Option<(String, String, String)> {
    Some((repo.into(), git_ref.into(), path.into()))
}

#[test]
fn forgejo_and_gitea_shapes_on_any_host() {
    for kind in ["branch", "tag", "commit"] {
        assert_eq!(
            parsed(&format!(
                "https://forge.fedoraproject.org/docs/site/src/{kind}/main/modules/ROOT/pages/index.adoc"
            )),
            shape(
                "https://forge.fedoraproject.org/docs/site",
                "main",
                "modules/ROOT/pages/index.adoc"
            )
        );
    }
    assert_eq!(
        parsed("http://git.example.org:3000/o/r/src/branch/dev/a.md?display=source#L3"),
        shape("http://git.example.org:3000/o/r", "dev", "a.md"),
        "port kept, query and fragment dropped"
    );
}

#[test]
fn gitlab_shapes_keep_nested_groups_in_the_repo() {
    assert_eq!(
        parsed("https://gitlab.com/fedora/docs/docs-website/-/edit/main/a/b.adoc"),
        shape(
            "https://gitlab.com/fedora/docs/docs-website",
            "main",
            "a/b.adoc"
        )
    );
    assert_eq!(
        parsed("https://gitlab.com/g/r/-/blob/v1/x.md"),
        shape("https://gitlab.com/g/r", "v1", "x.md")
    );
}

#[test]
fn github_and_pagure_shapes() {
    assert_eq!(
        parsed("https://github.com/example/site/edit/main/docs/a.md"),
        shape("https://github.com/example/site", "main", "docs/a.md")
    );
    assert_eq!(
        parsed("https://github.com/example/site/blob/main/docs/a.md"),
        shape("https://github.com/example/site", "main", "docs/a.md")
    );
    assert_eq!(
        parsed("https://pagure.io/fedora-docs/blob/master/f/docs/a.adoc"),
        shape("https://pagure.io/fedora-docs", "master", "docs/a.adoc"),
        "Pagure's f/ is not part of the path"
    );
    assert_eq!(
        parsed("https://pagure.io/fedora-docs/edit/master/f/docs/a.adoc"),
        shape("https://pagure.io/fedora-docs", "master", "docs/a.adoc"),
        "Pagure's edit page has the same f/"
    );
}

#[test]
fn a_ref_with_a_slash_splits_at_its_first_segment() {
    // KD1: the parse cannot know where the ref ends.
    assert_eq!(
        parsed("https://github.com/o/r/edit/feature/x/docs/a.md"),
        shape("https://github.com/o/r", "feature", "x/docs/a.md")
    );
}

#[test]
fn a_repo_named_for_a_marker_is_read_as_one() {
    // KD2: the leftmost marker decides, and a path's own `edit` does not.
    assert_eq!(
        parsed("https://github.com/o/edit/blob/main/a.md"),
        shape("https://github.com/o", "blob", "main/a.md")
    );
    assert_eq!(
        parsed("https://github.com/o/r/blob/main/edit/a.md"),
        shape("https://github.com/o/r", "main", "edit/a.md")
    );
}

#[test]
fn a_url_without_a_shape_or_without_a_file_is_none() {
    for url in [
        "https://forge.example.org/docs/site",
        "https://forge.example.org/docs/site/issues/new",
        "https://forge.example.org/docs/site/src/branch/main",
        "https://forge.example.org/docs/site/src/branch/main/",
        "https://forge.example.org/docs/site/src/branch/main/dir/",
        "https://github.com/o/r/edit/main",
        "https://github.com/edit/main/a.md",
        "mailto:a@example.org",
    ] {
        assert_eq!(parsed(url), None, "{url}");
    }
}

#[test]
fn an_edit_link_keeps_the_whole_url_with_or_without_a_shape() {
    let url = Url::parse("https://forge.example.org/docs/site/issues/new?x=1").unwrap();
    let link = EditLink::new(url.clone());
    assert_eq!(link.url, url);
    assert_eq!(link.source, None);
}
