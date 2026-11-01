//! The digest's views (D95): a document made from the model by a template. The
//! default view, `view/default/`, is embedded and renders the one HTML
//! document D64 describes, so a release stays one file (D23); the model it
//! reads is [`model::context`](super::model), and nothing else is. Escaping
//! follows the view's name, as the HTML a `.html` view prints is escaped, and
//! a name that is none leaves text as it is.

mod filters;
#[cfg(test)]
mod macros;
#[cfg(test)]
mod tests;

use minijinja::{AutoEscape, Environment, Error, UndefinedBehavior, syntax::SyntaxConfig};

use super::document::Digest;
use super::model;

// deficiency: KD39 `default/digest.html` opens "By: Source Page" with an `<i>` and
// ends that note's `</p>` with no `<p>` before it, and opens the "Unchecked links"
// `<p>` without closing it, as the document it replaced did, the golden file and the
// tests holding the bytes. A browser repairs both. Adding the `<p>` and the `</p>`
// closes it, and moves the golden file.
const DIGEST: &str = include_str!("view/default/digest.html");
const STYLE: &str = include_str!("view/default/style.css");

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
        if name.ends_with(".html") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });
    env.set_formatter(filters::format);
    filters::add(&mut env);
    env
}

/// The default view's two files, loaded.
fn default_view() -> Environment<'static> {
    let mut env = environment();
    env.add_template("digest.html", DIGEST)
        .expect("the default view parses");
    env.add_template("style.css", STYLE)
        .expect("the default view's stylesheet parses");
    env
}

/// The digest rendered by the default view.
fn render_default(d: &Digest) -> Result<String, Error> {
    default_view()
        .get_template("digest.html")?
        .render(model::context(d))
}

impl Digest {
    /// The HTML document, the default view over the model (D64).
    pub fn html(&self) -> String {
        render_default(self).unwrap_or_else(|e| panic!("the default view failed: {e:#}"))
    }
}
