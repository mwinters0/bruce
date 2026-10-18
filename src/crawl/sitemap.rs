//! Reading one sitemap file (D14): a `<urlset>` of pages or a
//! `<sitemapindex>` of further sitemaps, gzip-compressed or not
//! (`roadmap-P3-crawler.md`, "Sitemaps").
//!
//! A compressed file is told by its first two bytes, not its name or its
//! `Content-Type`: a `.xml.gz` file is served as `application/gzip`,
//! `application/x-gzip` or `application/octet-stream`, and one served with
//! `Content-Encoding: gzip` reaches here already decompressed. Only each
//! `<loc>`'s text is read, by local name, so a namespace prefix or none reads
//! alike; `<lastmod>` and the rest are ignored, docs.fp.o stamping every entry
//! with its build time.

use std::io::Read;

use flate2::read::GzDecoder;
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::Event;

/// What a sitemap file lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sitemap {
    /// A `<urlset>`: pages.
    Pages(Vec<String>),
    /// A `<sitemapindex>`: more sitemaps.
    Index(Vec<String>),
}

/// Why a file is not a sitemap that can be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unread {
    /// Its decompressed bytes run past the cap.
    TooLarge,
    /// It is not gzip, XML, or a `<urlset>` or `<sitemapindex>`.
    NotASitemap,
}

/// The `<loc>`s of the sitemap in `bytes`, which may be gzip, each trimmed and
/// with its entities resolved. `max` caps the decompressed size.
pub fn read(bytes: &[u8], max: usize) -> Result<Sitemap, Unread> {
    let inflated;
    let bytes = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        GzDecoder::new(bytes)
            .take(max as u64 + 1)
            .read_to_end(&mut out)
            .map_err(|_| Unread::NotASitemap)?;
        if out.len() > max {
            return Err(Unread::TooLarge);
        }
        inflated = out;
        &inflated[..]
    } else {
        bytes
    };
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    // The protocol requires UTF-8.
    let text = String::from_utf8_lossy(bytes);
    parse(&text)
}

fn parse(text: &str) -> Result<Sitemap, Unread> {
    let mut reader = Reader::from_str(text);
    let mut index = None;
    let mut locs = Vec::new();
    let mut loc: Option<String> = None;
    loop {
        match reader.read_event().map_err(|_| Unread::NotASitemap)? {
            Event::Start(e) => {
                let name = e.local_name();
                match (index, name.as_ref()) {
                    (None, "urlset") => index = Some(false),
                    (None, "sitemapindex") => index = Some(true),
                    (None, _) => return Err(Unread::NotASitemap),
                    (Some(_), "loc") => loc = Some(String::new()),
                    _ => {}
                }
            }
            Event::Empty(e) if index.is_none() => {
                let name = e.local_name();
                match name.as_ref() {
                    "urlset" => return Ok(Sitemap::Pages(Vec::new())),
                    "sitemapindex" => return Ok(Sitemap::Index(Vec::new())),
                    _ => return Err(Unread::NotASitemap),
                }
            }
            Event::Text(t) => {
                if let Some(loc) = &mut loc {
                    loc.push_str(&t.xml10_content());
                }
            }
            Event::CData(t) => {
                if let Some(loc) = &mut loc {
                    loc.push_str(&t.xml10_content());
                }
            }
            Event::GeneralRef(r) => {
                if let Some(loc) = &mut loc {
                    match r.resolve_char_ref() {
                        Ok(Some(c)) => loc.push(c),
                        Ok(None) => match resolve_predefined_entity(&r) {
                            Some(s) => loc.push_str(s),
                            None => return Err(Unread::NotASitemap),
                        },
                        Err(_) => return Err(Unread::NotASitemap),
                    }
                }
            }
            Event::End(e) if e.local_name().as_ref() == "loc" => {
                if let Some(text) = loc.take() {
                    let text = text.trim();
                    if !text.is_empty() {
                        locs.push(text.to_owned());
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    match index {
        Some(true) => Ok(Sitemap::Index(locs)),
        Some(false) => Ok(Sitemap::Pages(locs)),
        None => Err(Unread::NotASitemap),
    }
}

#[cfg(test)]
mod tests;
