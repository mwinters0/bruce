//! Forge edit-URL shapes, read back to a source repo, ref and path (D36), and
//! the source rules a config states before them ([`rule`], D93).
//!
//! A shape is recognised by its path on any host, since a forge may sit on a
//! host nothing else names. The markers, with `<ref>` one path segment:
//!
//! | Forge | Path |
//! |---|---|
//! | Forgejo, Gitea | `/src/branch\|tag\|commit/<ref>/<path>` |
//! | GitLab | `/-/blob\|edit/<ref>/<path>` |
//! | Pagure | `/blob\|edit/<ref>/f/<path>` |
//! | GitHub | `/blob\|edit/<ref>/<path>` |
//!
//! The leftmost marker past the first segment that leaves a ref and a file
//! path decides. Pagure's is tried before GitHub's, whose `/blob\|edit/<ref>/f/…`
//! it shadows, a file `f` at the repo's root included. Query and fragment are dropped, and the path is kept as the
//! URL spells it, percent-encoding included.
//!
//! Two limits of that reading are properties, a source rule being their
//! remedy where a site meets one. The ref is one segment, so a ref holding a
//! `/` (a branch `feature/x`) splits into the ref `feature` and a path
//! `x/<path>`: telling them apart takes the forge's API, one call per repo,
//! which a parse of the URL cannot make. And the scan runs from the left, so a
//! repo or group named for a marker (`https://github.com/o/edit/blob/main/a.md`)
//! is read as one, giving the repo `https://github.com/o` and the ref `blob`;
//! scanning from the right would misread a file path holding an `edit` or
//! `blob` directory, which is likelier than a repo named so. In both
//! `source_url` keeps the whole URL, and `source_repo` is right for the first.

mod rule;

pub use rule::{Over, Rule, source};
use url::Url;

/// What a forge edit URL names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The URL up to the shape's marker, its userinfo dropped:
    /// `https://host/org/repo`. A URL with an opaque origin (not `http`,
    /// `https`, `ftp`, `ws` or `wss`, nor a `blob:` of one, which takes its
    /// inner URL's) gives `null` for the origin.
    pub repo: String,
    /// The ref; one segment where [`parse`] read it, none where a rule states
    /// no ref.
    pub git_ref: Option<String>,
    /// The file's path in the repo.
    pub path: String,
}

/// The repo, ref and path `url` names, if its path has a forge's shape.
pub fn parse(url: &Url) -> Option<Source> {
    let segments: Vec<&str> = url.path_segments()?.collect();
    for at in 1..segments.len() {
        let rest = &segments[at..];
        let (marker, pagure) = match rest {
            ["src", "branch" | "tag" | "commit", ..] | ["-", "blob" | "edit", ..] => (2, false),
            ["blob" | "edit", _, "f", ..] => (1, true),
            ["blob" | "edit", ..] => (1, false),
            _ => continue,
        };
        let [git_ref, path @ ..] = &rest[marker..] else {
            continue;
        };
        let path = path[usize::from(pagure)..].join("/");
        if git_ref.is_empty() || path.is_empty() || path.ends_with('/') {
            continue;
        }
        return Some(Source {
            repo: format!(
                "{}/{}",
                url.origin().ascii_serialization(),
                segments[..at].join("/")
            ),
            git_ref: Some((*git_ref).to_string()),
            path,
        });
    }
    None
}

#[cfg(test)]
mod tests;
