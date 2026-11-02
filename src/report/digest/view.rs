//! The digest's views (D95): a document made from the model by a template. The
//! default view, `view/default/`, is embedded and renders the one HTML
//! document D64 describes, so a release stays one file (D23); a deployment's
//! own is a file, alone or extending the default as `bruce:digest.html`, and
//! sees its own directory and nothing outside it (D96). The model it reads is
//! [`model::context`](super::model), and nothing else is. Escaping follows the
//! name of the template printing: HTML's for `.html`, `.htm` and `.xml`, one RFC 4180 field for
//! `.csv`, and none for any other.

mod declared;
mod filters;
#[cfg(test)]
mod litmus;
mod loader;
#[cfg(test)]
mod macros;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod views;

use std::fs;
use std::path::PathBuf;

use minijinja::{AutoEscape, Environment, UndefinedBehavior, syntax::SyntaxConfig};

use super::document::Digest;
use super::json::FORMAT_VERSION;
use super::model;

// deficiency: KD39 `default/digest.html` opens "By: Source Page" with an `<i>` and
// ends that note's `</p>` with no `<p>` before it, and opens the "Unchecked links"
// `<p>` without closing it, as the document it replaced did, the golden file and the
// tests holding the bytes. A browser repairs both. Adding the `<p>` and the `</p>`
// closes it, and moves the golden file.

/// The built-in view's entry template.
const ENTRY: &str = "bruce:digest.html";

/// The engine every view runs in. Undefined names are errors, so a view naming
/// a field the model lacks fails with its place; a block tag on a line of its
/// own prints nothing, so a template is laid out as its document is.
fn environment() -> Environment<'static> {
    let mut env = Environment::new();
    let syntax = SyntaxConfig::builder()
        .trim_blocks(true)
        .lstrip_blocks(true)
        .keep_trailing_newline(true)
        .build()
        .expect("the default delimiters are valid");
    env.set_syntax(syntax);
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_auto_escape_callback(|name| {
        let extension = name.rsplit_once('.').map(|(_, e)| e);
        match extension {
            Some("html" | "htm" | "xml") => AutoEscape::Html,
            Some("csv") => AutoEscape::Custom("csv".into()),
            _ => AutoEscape::None,
        }
    });
    env.set_formatter(filters::format);
    filters::add(&mut env);
    env
}

/// The built-in view's three files, under their `bruce:` names.
fn default_view() -> Environment<'static> {
    let mut env = environment();
    loader::install(&mut env, None);
    env
}

/// What `report digest --view` names: the built-in view, or a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewSource {
    Default,
    File(PathBuf),
}

/// One view's output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// The view's file name, or `None` for the built-in view.
    pub name: Option<String>,
    pub text: String,
}

/// A view ready to render: its engine, its entry template and what to call it
/// in a failure.
struct Loaded {
    label: String,
    name: Option<String>,
    env: Environment<'static>,
    entry: String,
}

/// The views a digest is written through, each loaded and checked (D96).
pub struct Views(Vec<Loaded>);

impl Views {
    /// Loads `sources`, refusing a file that cannot be read or parsed, one
    /// declaring another model version than this `bruce`'s, and two that would
    /// write one file. Nothing is rendered.
    pub fn load(sources: &[ViewSource]) -> Result<Views, String> {
        let mut views: Vec<Loaded> = Vec::new();
        for source in sources {
            let view = match source {
                ViewSource::Default => Loaded {
                    label: "the built-in view".to_owned(),
                    name: None,
                    env: default_view(),
                    entry: ENTRY.to_owned(),
                },
                ViewSource::File(path) => load_file(path)?,
            };
            if let Some(twin) = views.iter().find(|v| v.name == view.name) {
                return Err(format!(
                    "{} and {} would write one file",
                    twin.label, view.label
                ));
            }
            views.push(view);
        }
        Ok(Views(views))
    }

    /// The digest through each view, over one model, in order. One failing
    /// view fails the whole call, and nothing is returned of the rest.
    pub fn render(&self, digest: &Digest) -> Result<Vec<Rendered>, String> {
        let model = model::context(digest);
        self.0
            .iter()
            .map(|view| {
                let text = view
                    .env
                    .get_template(&view.entry)
                    .and_then(|t| t.render(model.clone()))
                    .map_err(|e| format!("{}: {e:#}", view.label))?;
                Ok(Rendered {
                    name: view.name.clone(),
                    text,
                })
            })
            .collect()
    }
}

/// A view that is a file: its directory is what its templates see.
fn load_file(path: &std::path::Path) -> Result<Loaded, String> {
    let label = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|e| format!("--view {label}: {e}"))?;
    declared::check(&source).map_err(|e| format!("--view {label}: {e}"))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("--view {label}: no file name"))?
        .to_owned();
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = fs::canonicalize(dir.unwrap_or(std::path::Path::new(".")))
        .map_err(|e| format!("--view {label}: {e}"))?;
    let mut env = environment();
    loader::install(&mut env, Some(dir));
    env.add_template_owned(name.clone(), source)
        .map_err(|e| format!("--view {label}: {e:#}"))?;
    Ok(Loaded {
        label,
        entry: name.clone(),
        name: Some(name),
        env,
    })
}

/// The built-in view's files as `--write-default-view` writes them: each by its
/// plain name, the entry opening with its model version declaration (D96).
pub fn default_files() -> Vec<(&'static str, String)> {
    loader::DEFAULT
        .iter()
        .map(|&(name, text)| {
            let text = if name == "digest.html" {
                format!("{{% set bruce_model = {FORMAT_VERSION} %}}\n{text}")
            } else {
                text.to_owned()
            };
            (name, text)
        })
        .collect()
}

impl Digest {
    /// The HTML document, the built-in view over the model (D64): the one the
    /// tests read.
    #[cfg(test)]
    pub fn html(&self) -> String {
        default_view()
            .get_template(ENTRY)
            .and_then(|t| t.render(model::context(self)))
            .unwrap_or_else(|e| panic!("the default view failed: {e:#}"))
    }
}
