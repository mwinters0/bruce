//! The views a deployment writes: their names, their directory, their version
//! and the format their values are escaped for (D96).

use std::path::Path;

use minijinja::value::Value;
use tempfile::TempDir;

use super::super::tests::{of, store};
use super::{Rendered, ViewSource, Views, default_files, model};

/// A view directory holding `files`, each a name and its text.
fn directory(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (name, text) in files {
        let path = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

/// The digest of the test store through the file `name` of `dir`.
fn through(dir: &Path, name: &str) -> Result<String, String> {
    let views = Views::load(&[ViewSource::File(dir.join(name))])?;
    let rendered = views.render(&of(&store()))?;
    Ok(rendered.into_iter().next().unwrap().text)
}

#[test]
fn a_csv_view_quotes_each_value_it_prints_as_one_field() {
    let render = |source: &str| {
        let mut env = super::environment();
        env.add_template_owned("v.csv", source.to_owned()).unwrap();
        env.get_template("v.csv")
            .unwrap()
            .render(Value::from_pairs([
                ("title", Value::from("a, \"b\"\nc")),
                ("n", Value::from(1234)),
                ("nothing", Value::from(())),
                ("raw", Value::from("x,y")),
            ]))
    };
    assert_eq!(
        render("{{ title }},{{ n }},{{ nothing }},{{ raw|safe }}\n").unwrap(),
        "\"a, \"\"b\"\"\nc\",\"1234\",,x,y\n"
    );
    assert_eq!(
        render("{{ n|thousands(',') }}").unwrap(),
        "\"1,234\"",
        "a filter's text is one field"
    );
}

#[test]
fn a_views_extension_picks_its_escaping() {
    let said = |name: &str| {
        let mut env = super::environment();
        env.add_template_owned(name.to_owned(), "{{ t }}").unwrap();
        env.get_template(name)
            .unwrap()
            .render(Value::from_pairs([("t", Value::from("<a>\"'"))]))
            .unwrap()
    };
    for html in ["v.html", "v.htm", "v.xml"] {
        assert_eq!(said(html), "&lt;a&gt;&quot;&#39;", "{html}");
    }
    assert_eq!(said("v.csv"), "\"<a>\"\"'\"");
    for plain in ["v.txt", "v.json", "v"] {
        assert_eq!(said(plain), "<a>\"'", "{plain}");
    }
}

#[test]
fn a_two_form_plural_takes_one_or_the_other() {
    let said = |source: &str| {
        let mut env = super::environment();
        env.add_template_owned("v.txt", source.to_owned()).unwrap();
        env.get_template("v.txt")
            .unwrap()
            .render(Value::UNDEFINED)
            .unwrap()
    };
    assert_eq!(said("{{ 1|plural('link', 'links') }}"), "link");
    assert_eq!(said("{{ 0|plural('link', 'links') }}"), "links");
    assert_eq!(said("{{ 2|plural('link', 'links') }}"), "links");
    // A rule of more forms is the view's own: Polish counts 2 to 4 by the last
    // digit, and not 12 to 14.
    let polish = "{% macro f(n) %}{% if n == 1 %}one{% elif n % 10 in [2, 3, 4] and n % 100 not in [12, 13, 14] %}few{% else %}many{% endif %}{% endmacro %}{{ f(1) }} {{ f(22) }} {{ f(12) }} {{ f(5) }}";
    assert_eq!(said(polish), "one few many many");
}

#[test]
fn a_view_extends_the_built_in_one_and_changes_only_the_blocks_it_names() {
    let dir = directory(&[(
        "own.html",
        "{% set bruce_model = 2 %}{% extends \"bruce:digest.html\" %}{% block intro %}<p id=\"mine\">hello</p>\n{% endblock %}",
    )]);
    let own = through(dir.path(), "own.html").unwrap();
    let default = of(&store()).html();
    assert_eq!(
        own,
        default.replacen("<main>", "<p id=\"mine\">hello</p>\n<main>", 1)
    );
}

#[test]
fn the_built_in_views_includes_stay_in_it_beside_a_directory_of_the_same_names() {
    let dir = directory(&[
        ("own.html", "{% extends \"bruce:digest.html\" %}"),
        ("style.css", "body { color: red }"),
        ("palette.css", ":root { --x: 1 }"),
    ]);
    let own = through(dir.path(), "own.html").unwrap();
    assert_eq!(own, of(&store()).html());
    // The same names, asked for by the view, are its own.
    let dir = directory(&[
        ("a.css", "{% include \"b.css\" %}!"),
        ("b.css", "b"),
        ("deep/c.css", "c"),
        ("d.css", "{% include \"deep/c.css\" %}"),
    ]);
    assert_eq!(through(dir.path(), "a.css").unwrap(), "b!");
    assert_eq!(through(dir.path(), "d.css").unwrap(), "c");
}

#[test]
fn an_exported_view_renders_the_built_in_document_from_its_own_files() {
    let files = default_files();
    let names: Vec<&str> = files.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["digest.html", "style.css", "palette.css"]);
    let dir = TempDir::new().unwrap();
    for (name, text) in &files {
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    assert_eq!(
        through(dir.path(), "digest.html").unwrap(),
        of(&store()).html()
    );
    // An edit to the palette is the view's.
    std::fs::write(dir.path().join("palette.css"), ":root { --bg: red; }\n").unwrap();
    let edited = through(dir.path(), "digest.html").unwrap();
    assert!(edited.contains(":root { --bg: red; }\n"));
    assert!(!edited.contains("color-scheme: dark"));
}

#[test]
fn a_view_sees_its_directory_and_nothing_outside_it() {
    let outer = directory(&[("secret.txt", "outside")]);
    let dir = directory(&[
        ("up.txt", "{% include \"../secret.txt\" %}"),
        ("absolute.txt", "{% include \"/etc/hostname\" %}"),
        ("missing.txt", "{% include \"nowhere.txt\" %}"),
        ("unknown.txt", "{% include \"bruce:nowhere.txt\" %}"),
    ]);
    for name in ["up.txt", "absolute.txt"] {
        let error = through(dir.path(), name).unwrap_err();
        assert!(error.contains("its own directory"), "{error}");
    }
    for name in ["missing.txt", "unknown.txt"] {
        let error = through(dir.path(), name).unwrap_err();
        assert!(error.contains("nowhere.txt"), "{error}");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(outer.path().join("secret.txt"), dir.path().join("link.txt"))
            .unwrap();
        std::fs::write(dir.path().join("via.txt"), "{% include \"link.txt\" %}").unwrap();
        let error = through(dir.path(), "via.txt").unwrap_err();
        assert!(error.contains("leaves the view's directory"), "{error}");
    }
}

#[test]
fn a_view_declaring_another_model_version_is_refused_by_name_before_it_renders() {
    // The second tag names a field the model lacks: the version is what is said.
    for (source, said) in [
        (
            "{% set bruce_model = 1 %}{{ gone }}",
            "declares model version 1",
        ),
        ("{%- set  bruce_model=99 -%}", "declares model version 99"),
        ("{% set bruce_model = '2' %}", "no whole number"),
    ] {
        let dir = directory(&[("v.txt", source)]);
        let error = Views::load(&[ViewSource::File(dir.path().join("v.txt"))])
            .err()
            .expect("refused");
        assert!(error.contains("v.txt") && error.contains(said), "{error}");
        assert!(error.contains("docs/manual/digest.md"), "{error}");
    }
    for source in ["{% set bruce_model = 2 %}{{ date }}", "{{ date }}"] {
        let dir = directory(&[("v.txt", source)]);
        assert!(through(dir.path(), "v.txt").is_ok(), "{source}");
    }
    // The model says which version it is.
    assert_eq!(
        model::context(&of(&store()))
            .get_attr("bruce")
            .unwrap()
            .get_attr("model_version")
            .unwrap()
            .to_string(),
        "2"
    );
}

#[test]
fn a_failing_view_is_named_with_its_line_and_what_it_asked_for() {
    let dir = directory(&[("v.csv", "{{ date }}\n{{ nope.missing }}\n")]);
    let error = through(dir.path(), "v.csv").unwrap_err();
    assert!(
        error.starts_with(&dir.path().join("v.csv").display().to_string()),
        "{error}"
    );
    assert!(
        error.contains("v.csv:2") && error.contains("nope"),
        "{error}"
    );
    let dir = directory(&[("v.csv", "{% if %}")]);
    let error = through(dir.path(), "v.csv").unwrap_err();
    assert!(error.contains("v.csv"), "{error}");
}

#[test]
fn two_views_of_one_name_would_write_one_file_and_are_refused() {
    let a = directory(&[("links.csv", "a")]);
    let b = directory(&[("links.csv", "b")]);
    let error = Views::load(&[
        ViewSource::File(a.path().join("links.csv")),
        ViewSource::File(b.path().join("links.csv")),
    ])
    .err()
    .expect("refused");
    assert!(error.contains("one file"), "{error}");
    let error = Views::load(&[ViewSource::Default, ViewSource::Default])
        .err()
        .expect("refused");
    assert!(error.contains("one file"), "{error}");
}

#[test]
fn every_view_is_rendered_over_one_model_and_a_failure_returns_none_of_them() {
    let ok = directory(&[("ok.txt", "{{ date }}")]);
    let bad = directory(&[("bad.txt", "{{ nope }}")]);
    let views = Views::load(&[
        ViewSource::File(ok.path().join("ok.txt")),
        ViewSource::File(bad.path().join("bad.txt")),
    ])
    .unwrap();
    assert!(views.render(&of(&store())).is_err());
    let views = Views::load(&[
        ViewSource::Default,
        ViewSource::File(ok.path().join("ok.txt")),
    ])
    .unwrap();
    let rendered: Vec<Rendered> = views.render(&of(&store())).unwrap();
    assert_eq!(rendered[0].name, None);
    assert_eq!(rendered[1].name.as_deref(), Some("ok.txt"));
    assert_eq!(rendered[1].text, of(&store()).date());
}
