//! Where a view's templates come from: the built-in view under `bruce:` names,
//! and a view's own directory under plain ones, nothing outside it (D96).

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use minijinja::{Environment, Error, ErrorKind as Kind};

/// The prefix of the built-in view's templates, as in `{% extends "bruce:digest.html" %}`.
pub(super) const BRUCE: &str = "bruce:";

/// The built-in view's files, each under its plain name.
pub(super) const DEFAULT: [(&str, &str); 3] = [
    ("digest.html", include_str!("default/digest.html")),
    ("style.css", include_str!("default/style.css")),
    ("palette.css", include_str!("default/palette.css")),
];

/// Lets `env` load `bruce:` names and, where `dir` is given, plain names from
/// it. The built-in view's own includes stay inside it: a plain name written in
/// a `bruce:` template is a `bruce:` one.
pub(super) fn install(env: &mut Environment<'static>, dir: Option<PathBuf>) {
    env.set_path_join_callback(|name, parent| {
        if parent.starts_with(BRUCE) && !name.starts_with(BRUCE) {
            format!("{BRUCE}{name}").into()
        } else {
            name.into()
        }
    });
    env.set_loader(move |name| match name.strip_prefix(BRUCE) {
        Some(file) => Ok(DEFAULT
            .iter()
            .find(|(n, _)| *n == file)
            .map(|(_, text)| (*text).to_owned())),
        None => dir.as_deref().map_or(Ok(None), |dir| read(dir, name)),
    });
}

/// The file `name` is in `dir`, a name the view wrote. `dir` is canonical, so a
/// link that leaves it is found by where the file really is.
fn read(dir: &Path, name: &str) -> Result<Option<String>, Error> {
    let own = Path::new(name)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
        && !name.contains('\\');
    if !own {
        return Err(Error::new(
            Kind::InvalidOperation,
            format!(
                "{name}: a view's templates are in its own directory, named without `..` or a leading `/`"
            ),
        ));
    }
    let real = match fs::canonicalize(dir.join(name)) {
        Ok(real) => real,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::new(Kind::InvalidOperation, format!("{name}: {e}"))),
    };
    if !real.starts_with(dir) {
        return Err(Error::new(
            Kind::InvalidOperation,
            format!("{name}: a link leaves the view's directory"),
        ));
    }
    fs::read_to_string(&real)
        .map(Some)
        .map_err(|e| Error::new(Kind::InvalidOperation, format!("{name}: {e}")))
}
