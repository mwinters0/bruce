use super::*;

#[test]
fn edit_phrases_ignore_case_whitespace_and_decoration() {
    for yes in [
        "Edit this Page",
        "edit this page",
        "  Edit\n  this   page ",
        "Edit page",
        "Edit on GitHub",
        " Edit on GitLab",
        "✎ Edit this page »",
        "Edit on forge.fedoraproject.org",
    ] {
        assert!(is_edit_phrase(yes), "{yes:?}");
    }
    for no in [
        "",
        "Edit",
        "Page history",
        "Report an issue",
        "Improve this page",
        "Edit this page history",
        "Do not edit this page",
        "Editing on GitHub",
    ] {
        assert!(!is_edit_phrase(no), "{no:?}");
    }
}
