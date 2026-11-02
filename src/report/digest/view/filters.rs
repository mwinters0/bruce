//! What a view's templates are given beyond the engine's own: the formatter
//! that prints a value in the view's format, and the filters taking the
//! conventions a view states for its numbers and dates, so the model holds raw
//! values and no locale (D97).

use std::fmt::Write as _;

use minijinja::value::Value;
use minijinja::{AutoEscape, Environment, Error, ErrorKind, Output, State};

use crate::report::digest::document::utc;

/// Prints `value` in the format of the template that prints it: HTML's escape
/// for HTML, one quoted RFC 4180 field for CSV, nothing done for any other, and
/// nothing printed for a `none`, which is a cell the run holds no value for. A
/// value a template marked safe prints as it is.
pub(super) fn format(out: &mut Output, state: &mut State, value: &Value) -> Result<(), Error> {
    if value.is_none() {
        return Ok(());
    }
    let text = value.to_string();
    let escape = state.auto_escape().clone();
    if value.is_safe() || escape == AutoEscape::None {
        out.write_str(&text)?;
        return Ok(());
    }
    if escape != AutoEscape::Html {
        out.write_char('"')?;
        for c in text.chars() {
            if c == '"' {
                out.write_char('"')?;
            }
            out.write_char(c)?;
        }
        return Ok(out.write_char('"')?);
    }
    for c in text.chars() {
        match c {
            '&' => out.write_str("&amp;")?,
            '<' => out.write_str("&lt;")?,
            '>' => out.write_str("&gt;")?,
            '"' => out.write_str("&quot;")?,
            '\'' => out.write_str("&#39;")?,
            c => out.write_char(c)?,
        }
    }
    Ok(())
}

pub(super) fn add(env: &mut Environment<'_>) {
    env.add_filter("thousands", thousands);
    env.add_filter("time", time);
    env.add_filter("plural", plural);
    env.add_filter("short", short);
    env.add_filter("slash_runs", slash_runs);
    env.add_filter("final_slash_runs", final_slash_runs);
    env.add_test("web", web);
}

/// `n` with `sep` between its thousands.
fn thousands(n: i64, sep: &str) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push_str(sep);
        }
        out.push(c);
    }
    if n < 0 {
        out.insert(0, '-');
    }
    out
}

/// `one` where `n` is 1, else `other`: a two-form plural, a language with more
/// forms writing its own rule in the view.
fn plural(n: i64, one: Value, other: Value) -> Value {
    if n == 1 { one } else { other }
}

/// `seconds` since the epoch in UTC as `pattern` has it: `%Y`, `%m`, `%d`, `%H`,
/// `%M` and `%S`, zero-padded, and `%%`.
fn time(seconds: i64, pattern: &str) -> Result<String, Error> {
    let (_, rfc) = utc(seconds);
    let mut out = String::new();
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        out.push_str(match chars.next() {
            Some('Y') => &rfc[0..4],
            Some('m') => &rfc[5..7],
            Some('d') => &rfc[8..10],
            Some('H') => &rfc[11..13],
            Some('M') => &rfc[14..16],
            Some('S') => &rfc[17..19],
            Some('%') => "%",
            other => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!(
                        "time: `%{}` is not %Y %m %d %H %M %S or %%",
                        other.unwrap_or(' ')
                    ),
                ));
            }
        });
    }
    Ok(out)
}

/// `url` without its `https://` or `http://`, to show.
fn short(url: Option<&str>) -> String {
    let url = url.unwrap_or("");
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .to_owned()
}

/// Whether `url` is an `http` or `https` one, the only kind a view links.
fn web(url: Option<&str>) -> bool {
    url.is_some_and(|u| u.starts_with("https://") || u.starts_with("http://"))
}

/// `[text, marked]` pairs for the whole of `text`.
fn runs(parts: Vec<(String, bool)>) -> Vec<Value> {
    parts
        .into_iter()
        .map(|(text, marked)| Value::from(vec![Value::from(text), Value::from(marked)]))
        .collect()
}

/// `text` in runs, each doubled slash in its path marked: a run of `/` after
/// anything but a `:` or a `/`, ahead of any `?` or `#`, as `doubled_slash`
/// reads one (D71).
pub(super) fn slash_runs(text: &str) -> Vec<Value> {
    let path = &text[..text.find(['?', '#']).unwrap_or(text.len())];
    let bytes = path.as_bytes();
    let mut out = Vec::new();
    let (mut from, mut i) = (0, 1);
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' && !matches!(bytes[i - 1], b':' | b'/') {
            let end = i + bytes[i..].iter().take_while(|&&b| b == b'/').count();
            out.push((path[from..i].to_owned(), false));
            out.push((path[i..end].to_owned(), true));
            (from, i) = (end, end);
        } else {
            i += 1;
        }
    }
    out.push((text[from..].to_owned(), false));
    runs(out)
}

/// `text` in runs, the final `/` of its path marked where it has one, as a
/// `wrong_trailing_slash` landing's is (D72).
pub(super) fn final_slash_runs(text: &str) -> Vec<Value> {
    let at = text.find(['?', '#']).unwrap_or(text.len());
    runs(match text[..at].strip_suffix('/') {
        Some(bare) => vec![
            (bare.to_owned(), false),
            ("/".to_owned(), true),
            (text[at..].to_owned(), false),
        ],
        None => vec![(text.to_owned(), false)],
    })
}
