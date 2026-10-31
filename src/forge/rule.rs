//! A source rule (D36, D93): the deployment's statement of where a page's
//! source lives, tried before [`parse`] reads the edit link.
//!
//! A rule is a regex over the page's URL or over its edit link's, whose named
//! captures `repo`, `ref` and `path` give the [`Source`], or which `repo`,
//! `ref` and `path` templates (the `regex` crate's `${name}` expansion) state
//! where the URL does not. A template left out defaults to the capture of its
//! own name, and `ref` to none where there is no such capture.
//!
//! Rules are tried in the order they are given and the first that matches and
//! yields a repo and a path gives the source; one whose expansion is no
//! source is passed over, as [`parse`] passes over a URL with no file in it.
//! A rule asserts one format, so a site whose edit links are of several is
//! left to [`parse`] for the rest: the rules are tried first and a page no rule
//! matches is read as it was.

use regex::Regex;
use url::Url;

use super::{Source, parse};

/// What a rule's regex is matched against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Over {
    /// The page's own URL, for a site whose pages carry no edit link.
    Url,
    /// The page's edit link, for a forge shape [`parse`] does not know. A
    /// page without an edit link is passed over.
    EditLink,
}

impl Over {
    /// The `from` a config writes: `url` or `edit_link`.
    pub fn parse(text: &str) -> Result<Over, String> {
        match text {
            "url" => Ok(Over::Url),
            "edit_link" => Ok(Over::EditLink),
            other => Err(format!("{other:?} is not `url` or `edit_link`")),
        }
    }
}

/// One source rule with its regex compiled.
#[derive(Debug, Clone)]
pub struct Rule {
    over: Over,
    pattern: Regex,
    repo: String,
    git_ref: Option<String>,
    path: String,
}

impl Rule {
    /// The rule over `over` matching `pattern`. `Err` names what makes it no
    /// rule: a regex that does not compile, a template naming a capture the
    /// regex lacks, a `repo` or `path` neither stated nor captured, or a
    /// `repo` that cannot have the URL form [`Source::repo`] holds.
    pub fn new(
        over: Over,
        pattern: &str,
        repo: Option<&str>,
        git_ref: Option<&str>,
        path: Option<&str>,
    ) -> Result<Rule, String> {
        let regex = Regex::new(pattern).map_err(|e| format!("pattern: {e}"))?;
        let named = |name: &str| regex.capture_names().flatten().any(|n| n == name);
        let pick = |key: &str, template: Option<&str>, required: bool| match template {
            Some(t) => Ok(Some(t.to_owned())),
            None if named(key) => Ok(Some(format!("${{{key}}}"))),
            None if required => Err(format!(
                "{key}: the pattern has no capture named `{key}` and none is stated"
            )),
            None => Ok(None),
        };
        let repo = pick("repo", repo, true)?.unwrap_or_default();
        let git_ref = pick("ref", git_ref, false)?;
        let path = pick("path", path, true)?.unwrap_or_default();
        let templates = [
            ("repo", Some(&repo)),
            ("ref", git_ref.as_ref()),
            ("path", Some(&path)),
        ];
        for (key, template) in templates {
            if let Some(template) = template {
                references(template, &regex).map_err(|e| format!("{key}: {e}"))?;
            }
        }
        repo_stated(&repo).map_err(|e| format!("repo: {e}"))?;
        Ok(Rule {
            over,
            pattern: regex,
            repo,
            git_ref,
            path,
        })
    }

    /// What the rule's regex is matched against.
    pub fn over(&self) -> Over {
        self.over
    }

    /// The source `input` gives, if the regex matches it and the expansion is
    /// a source.
    fn apply(&self, input: &Url) -> Option<Source> {
        let captures = self.pattern.captures(input.as_str())?;
        let expand = |template: &str| {
            let mut out = String::new();
            captures.expand(template, &mut out);
            out
        };
        let path = expand(&self.path);
        if path.is_empty() || path.ends_with('/') {
            return None;
        }
        let git_ref = self
            .git_ref
            .as_deref()
            .map(expand)
            .filter(|r| !r.is_empty());
        Some(Source {
            repo: repo_form(&expand(&self.repo))?,
            git_ref,
            path,
        })
    }
}

/// The source of a page at `page` whose edit link, if it has one, is
/// `edit_link`: the first of `rules` that gives one, else what [`parse`]
/// reads of the edit link.
pub fn source(rules: &[Rule], page: &Url, edit_link: Option<&Url>) -> Option<Source> {
    rules
        .iter()
        .find_map(|rule| {
            let input = match rule.over {
                Over::Url => Some(page),
                Over::EditLink => edit_link,
            };
            input.and_then(|input| rule.apply(input))
        })
        .or_else(|| edit_link.and_then(parse))
}

/// `repo` in the form [`Source::repo`] holds: an `http` or `https` URL with a
/// host and a path, without userinfo, query, fragment or a final `/`.
fn repo_form(repo: &str) -> Option<String> {
    let url = Url::parse(repo).ok()?;
    let web = matches!(url.scheme(), "http" | "https") && url.host_str().is_some();
    if !web || url.query().is_some() || url.fragment().is_some() {
        return None;
    }
    let path = url.path().trim_end_matches('/');
    (!path.is_empty()).then(|| format!("{}{path}", url.origin().ascii_serialization()))
}

/// Whether a `repo` template can expand to [`repo_form`]: one without a
/// capture must be one; one with a capture must begin as a web URL does, or
/// with a capture.
fn repo_stated(template: &str) -> Result<(), String> {
    match template.split_once('$') {
        None if repo_form(template).is_some() => Ok(()),
        None => Err(format!(
            "{template:?} is not an http(s) URL with a host and a path, as `https://host/org/repo`"
        )),
        Some((literal, _))
            if literal.is_empty()
                || literal.starts_with("http://")
                || literal.starts_with("https://") =>
        {
            Ok(())
        }
        Some(_) => Err(format!(
            "{template:?} does not begin as an http(s) URL does, as `https://host/org/repo`"
        )),
    }
}

/// Checks that every capture `template` names, as `Captures::expand` reads
/// the name, exists in `regex`.
fn references(template: &str, regex: &Regex) -> Result<(), String> {
    let mut chars = template.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if c != '$' {
            continue;
        }
        let name = match chars.peek() {
            Some((_, '$')) => {
                chars.next();
                continue;
            }
            Some((_, '{')) => {
                let rest = &template[at + 2..];
                let Some(end) = rest.find('}') else {
                    return Err(format!("{template:?} opens `${{` and never closes it"));
                };
                for _ in 0..rest[..end].chars().count() + 2 {
                    chars.next();
                }
                &rest[..end]
            }
            _ => {
                let rest = &template[at + 1..];
                let len = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                for _ in 0..len {
                    chars.next();
                }
                &rest[..len]
            }
        };
        if name.is_empty() {
            continue;
        }
        let exists = match name.parse::<usize>() {
            Ok(index) => index < regex.captures_len(),
            Err(_) => regex.capture_names().flatten().any(|n| n == name),
        };
        if !exists {
            return Err(format!(
                "{template:?} names `{name}`, which the pattern does not capture"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
