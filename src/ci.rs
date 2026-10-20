//! CI mode: a local build checked against a baseline, failing only on the
//! breakage it introduced (D4, D9, D21, D22).
//!
//! A [`Build`] is a build directory read as the URLs it will have in
//! production: `--prefix` is the URL of the directory's root, and a file is
//! the prefix plus its path relative to the root. `foo/index.html` is also
//! `foo/` (D22), and the top `index.html` is the prefix itself. Every file is
//! a URL the build answers, an asset a page links to included, and **a seed**
//! of the run, a partial build's sitemap not being trusted (it has none): the
//! run then holds a target for each file the build has, which is how the
//! inverse check tells a file the change deleted from one it kept (D56).
//! Which file is which URL is this module's concept and no fetcher's: it
//! hands `fetch::local` a finished map.

use std::fmt;
use std::path::{Path, PathBuf};

use url::Url;

use crate::config::Config;

/// A build directory and the prefix it is served under.
#[derive(Debug, Clone)]
pub struct Build {
    prefix: Url,
    served: Vec<(Url, PathBuf)>,
    seeds: Vec<Url>,
}

/// Why a build could not be read.
#[derive(Debug)]
pub enum Error {
    /// The prefix is not an `http` or `https` URL without a query or a
    /// fragment.
    Prefix(String),
    /// The prefix is in no root's scope, so no baseline answers for the
    /// pages under it (D22).
    OutsideScope(Url),
    /// A directory or file could not be read.
    Io(PathBuf, std::io::Error),
    /// A file whose name is not UTF-8 is no URL.
    NotUtf8(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Prefix(why) => write!(f, "the prefix {why}"),
            Error::OutsideScope(url) => write!(
                f,
                "the prefix {url} is outside every root's scope, so the configuration would not \
                 plan its pages as it did for the baseline"
            ),
            Error::Io(path, e) => write!(f, "{}: {e}", path.display()),
            Error::NotUtf8(path) => write!(f, "{}: the name is not UTF-8", path.display()),
        }
    }
}

impl std::error::Error for Error {}

impl Build {
    /// The build in `dir`, served under `prefix`, which must be in the scope
    /// of one of `config`'s roots. A prefix without a trailing `/` gets one.
    pub fn scan(dir: &Path, prefix: &str, config: &Config) -> Result<Build, Error> {
        let prefix = parse_prefix(prefix)?;
        if !config
            .roots
            .iter()
            .any(|r| r.in_scope(&r.canonical(&prefix)))
        {
            return Err(Error::OutsideScope(prefix));
        }
        let mut files = Vec::new();
        walk(dir, &mut Vec::new(), &mut files)?;
        Ok(Build::from_files(prefix, files))
    }

    /// The build holding `files`, each as its path segments below the root
    /// and the file that holds it.
    fn from_files(prefix: Url, files: Vec<(Vec<String>, PathBuf)>) -> Build {
        let (mut served, mut seeds) = (Vec::new(), Vec::new());
        for (segments, file) in files {
            let url = url_of(&prefix, &segments, false);
            let index = segments.last().is_some_and(|s| s == "index.html");
            if index {
                let dir = url_of(&prefix, &segments[..segments.len() - 1], true);
                served.push((dir.clone(), file.clone()));
                seeds.push(dir);
            } else {
                seeds.push(url.clone());
            }
            served.push((url, file));
        }
        Build {
            prefix,
            served,
            seeds,
        }
    }

    /// The production URL of the build directory's root.
    pub fn prefix(&self) -> &Url {
        &self.prefix
    }

    /// The URL of each file, an index as its directory, in path order: where
    /// a run begins.
    pub fn seeds(&self) -> &[Url] {
        &self.seeds
    }

    /// Every URL the build answers, with the file that holds it, for
    /// `fetch::local`.
    pub fn served(&self) -> impl Iterator<Item = (Url, PathBuf)> + '_ {
        self.served.iter().cloned()
    }
}

/// `prefix` as a directory URL.
fn parse_prefix(prefix: &str) -> Result<Url, Error> {
    let mut url =
        Url::parse(prefix).map_err(|e| Error::Prefix(format!("{prefix} is not a URL: {e}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::Prefix(format!(
            "{prefix} is not an http or https URL"
        )));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(Error::Prefix(format!("{prefix} has a query or a fragment")));
    }
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

/// The URL `segments` below `prefix` name, with a trailing `/` for a
/// directory. Each segment is escaped, so a file named `a b#c.html` is one.
fn url_of(prefix: &Url, segments: &[String], directory: bool) -> Url {
    let mut url = prefix.clone();
    {
        let mut path = url.path_segments_mut().expect("an http URL has a path");
        path.pop_if_empty().extend(segments);
        if directory {
            path.push("");
        }
    }
    url
}

/// Every file below `dir`, as its segments below the root and its path, in
/// path order. A symlink to a file is a file, and one to a directory is not
/// walked.
fn walk(
    dir: &Path,
    below: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, PathBuf)>,
) -> Result<(), Error> {
    let io = |e| Error::Io(dir.to_owned(), e);
    let mut entries = std::fs::read_dir(dir)
        .map_err(io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(io)?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| Error::NotUtf8(path.clone()))?;
        let kind = std::fs::metadata(&path).map_err(|e| Error::Io(path.clone(), e))?;
        below.push(name);
        let real_dir = kind.is_dir()
            && entry
                .file_type()
                .map_err(|e| Error::Io(path.clone(), e))?
                .is_dir();
        if real_dir {
            walk(&path, below, out)?;
        } else if kind.is_file() {
            out.push((below.clone(), path));
        }
        below.pop();
    }
    Ok(())
}

#[cfg(test)]
mod tests;
