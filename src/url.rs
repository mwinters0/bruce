//! Canonical URLs and their IDs (D7, D26, D44).
//!
//! [`canonical`] is the one string a URL is known by: scope globs match it
//! (D44), and [`Canonical::id`] hashes it into the `url_id` every table keys
//! on. `url` already lowercases the scheme and host (punycode for a
//! non-ASCII one), drops a default port, resolves dot-segments and makes an
//! empty path `/`; what this adds is the escapes. A `%XX` in the path or the
//! query is decoded only where the byte is unreserved (`A-Z a-z 0-9 - . _ ~`), and every other
//! escape stays with its hex uppercased, so a `%2F` never becomes a `/`
//! that moves the path. The fragment is not part of it: it is
//! `links.fragment`. The query is, unless the root strips it.

use url::{Position, Url};
use xxhash_rust::xxh3::xxh3_64;

/// A URL in canonical form, see the module doc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Canonical(String);

impl Canonical {
    /// The canonical form, which globs match.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The `url_id`: `xxh3_64` of the canonical form, stable across runs
    /// (D26).
    pub fn id(&self) -> u64 {
        xxh3_64(self.0.as_bytes())
    }
}

/// `url` in canonical form, its query dropped when `strip_query` is set (a
/// root's key, so one URL has one form per root).
pub fn canonical(url: &Url, strip_query: bool) -> Canonical {
    let mut out = Vec::with_capacity(url.as_str().len());
    out.extend_from_slice(url[..Position::BeforePath].as_bytes());
    normalise_escapes(url.path(), &mut out);
    if !strip_query && let Some(query) = url.query() {
        out.push(b'?');
        normalise_escapes(query, &mut out);
    }
    // Only an ASCII `%XX` is ever replaced, and by ASCII, so UTF-8 holds.
    Canonical(String::from_utf8(out).expect("escapes are replaced by ASCII"))
}

fn normalise_escapes(s: &str, out: &mut Vec<u8>) {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(byte) = b.get(i + 1..i + 3).and_then(hex_pair)
        {
            if is_unreserved(byte) {
                out.push(byte);
            } else {
                out.push(b'%');
                out.extend_from_slice(&[HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 15)]]);
            }
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn hex_pair(pair: &[u8]) -> Option<u8> {
    let digit = |c: u8| char::from(c).to_digit(16);
    Some((digit(pair[0])? * 16 + digit(pair[1])?) as u8)
}

fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

#[cfg(test)]
mod tests;
