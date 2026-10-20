//! A response body to a `String` (`roadmap-P3-crawler.md`, "Which responses
//! are read"), which a fetcher other than HTTP can reuse: a byte-order mark,
//! then the `charset` of the `Content-Type` header, then a scan of the first
//! 1024 bytes for `<meta charset>` or `<meta http-equiv>`, then UTF-8. Bytes
//! the encoding cannot decode become U+FFFD.
//!
//! The scan follows the WHATWG prescan's shape (comments and every start
//! tag's attributes are walked, so a `charset` in a comment or in text is not
//! read) without its corners: an end tag's attributes are not skipped, a
//! `<meta>` is taken at its first `charset`, and the pragma's `content` is searched for a `charset=` as the header is.

use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252, X_USER_DEFINED};

/// How much of the body the `<meta>` scan reads.
const PRESCAN_BYTES: usize = 1024;

/// `bytes` as text, by the rules in the module doc.
pub fn decode(bytes: &[u8], content_type: Option<&str>) -> String {
    let (encoding, skip) = match Encoding::for_bom(bytes) {
        Some((encoding, len)) => (encoding, len),
        None => {
            let encoding = content_type
                .and_then(charset_param)
                .and_then(|label| Encoding::for_label(label.as_bytes()))
                .or_else(|| meta_charset(&bytes[..bytes.len().min(PRESCAN_BYTES)]))
                .unwrap_or(UTF_8);
            (encoding, 0)
        }
    };
    encoding
        .decode_without_bom_handling(&bytes[skip..])
        .0
        .into_owned()
}

/// The value of a `charset=` parameter in a header or a `content`
/// attribute, unquoted.
fn charset_param(text: &str) -> Option<&str> {
    let lower = text.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find("charset") {
        let after = from + found + "charset".len();
        let rest = text[after..].trim_start();
        if let Some(value) = rest.strip_prefix('=') {
            let value = value.trim_start();
            let value = match value.chars().next() {
                Some(q @ ('"' | '\'')) => value[1..].split(q).next().unwrap_or(""),
                _ => value
                    .split(|c: char| c.is_ascii_whitespace() || c == ';')
                    .next()
                    .unwrap_or(""),
            };
            return Some(value.trim());
        }
        from = after;
    }
    None
}

/// The encoding the first `<meta>` in `window` names, if it names a usable
/// one. A UTF-16 label read from the bytes is UTF-8, as the bytes are
/// ASCII-compatible to have been read at all, and `x-user-defined` is
/// windows-1252 (WHATWG).
fn meta_charset(window: &[u8]) -> Option<&'static Encoding> {
    let mut at = 0;
    while let Some(lt) = window[at..].iter().position(|&b| b == b'<') {
        at += lt;
        let rest = &window[at..];
        if rest.starts_with(b"<!--") {
            at += rest
                .windows(3)
                .position(|w| w == b"-->")
                .map_or(rest.len(), |p| p + 3);
            continue;
        }
        let Some(tag) = Tag::parse(rest) else {
            at += 1;
            continue;
        };
        at += tag.len;
        if tag.name != "meta" {
            continue;
        }
        let from_charset = tag.attr("charset").map(str::to_owned);
        let from_pragma = || {
            let pragma = tag.attr("http-equiv")?;
            if !pragma.eq_ignore_ascii_case("content-type") {
                return None;
            }
            charset_param(tag.attr("content")?).map(str::to_owned)
        };
        let Some(label) = from_charset.or_else(from_pragma) else {
            continue;
        };
        let encoding = Encoding::for_label(label.as_bytes())?;
        return Some(if encoding == UTF_16LE || encoding == UTF_16BE {
            UTF_8
        } else if encoding == X_USER_DEFINED {
            WINDOWS_1252
        } else {
            encoding
        });
    }
    None
}

/// A start tag read from bytes: its lowercased name, its attributes with
/// lowercased names and values as written, and how many bytes it took.
struct Tag {
    name: String,
    attrs: Vec<(String, String)>,
    len: usize,
}

impl Tag {
    /// Reads the tag at `bytes[0] == b'<'`, or `None` where it is not a
    /// start tag (an end tag, a doctype, a stray `<`).
    fn parse(bytes: &[u8]) -> Option<Tag> {
        let mut at = 1;
        if !bytes.get(at)?.is_ascii_alphabetic() {
            return None;
        }
        let name_len = bytes[at..]
            .iter()
            .position(|&b| b.is_ascii_whitespace() || b == b'/' || b == b'>')
            .unwrap_or(bytes.len() - at);
        let name = lossy(&bytes[at..at + name_len]).to_ascii_lowercase();
        at += name_len;
        let mut attrs = Vec::new();
        loop {
            while bytes
                .get(at)
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'/')
            {
                at += 1;
            }
            match bytes.get(at) {
                None => {
                    return Some(Tag {
                        name,
                        attrs,
                        len: at,
                    });
                }
                Some(b'>') => {
                    return Some(Tag {
                        name,
                        attrs,
                        len: at + 1,
                    });
                }
                Some(_) => {}
            }
            let key_len = bytes[at..]
                .iter()
                .position(|&b| b.is_ascii_whitespace() || b == b'=' || b == b'/' || b == b'>')
                .unwrap_or(bytes.len() - at)
                .max(1);
            let key = lossy(&bytes[at..at + key_len]).to_ascii_lowercase();
            at += key_len;
            while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                at += 1;
            }
            let mut value = String::new();
            if bytes.get(at) == Some(&b'=') {
                at += 1;
                while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                    at += 1;
                }
                match bytes.get(at) {
                    Some(&q @ (b'"' | b'\'')) => {
                        let body = &bytes[at + 1..];
                        let end = body.iter().position(|&b| b == q).unwrap_or(body.len());
                        value = lossy(&body[..end]);
                        at += 1 + end + 1;
                    }
                    _ => {
                        let body = &bytes[at..];
                        let end = body
                            .iter()
                            .position(|&b| b.is_ascii_whitespace() || b == b'>')
                            .unwrap_or(body.len());
                        value = lossy(&body[..end]);
                        at += end;
                    }
                }
            }
            attrs.push((key, value));
            at = at.min(bytes.len());
        }
    }

    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests;
