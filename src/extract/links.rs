//! Links: every reference a page makes, with its kind (D13).
//!
//! One handler reads each element that can name a URL, in document order, and
//! resolves what it finds against the page's URL, or against the first
//! `<base href>` once one has been read, or, where a local build's site root is
//! given, an href beginning `/` against that root (D53). A second handler on the same
//! elements runs after the profile's, takes the tag the profile left in the
//! sink and puts it on the links the element yielded (D38), which a profile's
//! own handler could not do from before the link existed.

use std::cell::RefCell;

use lol_html::element;
use lol_html::html_content::Element;
use url::Url;

use super::attribute;
use crate::profile::{ElementHandlers, Sink};

/// What a link is, as `links.kind` stores it (D13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// `a[href]` and `area[href]`: the kind reports default to.
    Anchor,
    /// `img[src]`, every `srcset` URL, `source[srcset]`.
    Image,
    /// `link[rel~=canonical]`.
    Canonical,
    /// `link[rel~=stylesheet]`.
    Stylesheet,
    /// `link[rel~=icon]`.
    Icon,
    /// Any other `link[rel]` that names a resource.
    Link,
    /// `script[src]`.
    Script,
    /// `iframe[src]`.
    Frame,
    /// `src` on `video`, `audio`, `source` and `track`; `poster`.
    Media,
    /// The `url=` of `meta[http-equiv=refresh]`.
    Refresh,
    /// An entry of a sitemap naming a page, its source the sitemap file. The
    /// extraction pass never yields it: the crawler gives a sitemap's entries
    /// this kind.
    Sitemap,
    /// A sitemap file a root read, its source the root's URL, or a sitemap an
    /// index lists, its source the index. The extraction pass never yields it.
    SitemapFile,
}

impl Kind {
    /// Every kind, in declaration order.
    pub const ALL: [Kind; 12] = [
        Kind::Anchor,
        Kind::Image,
        Kind::Canonical,
        Kind::Stylesheet,
        Kind::Icon,
        Kind::Link,
        Kind::Script,
        Kind::Frame,
        Kind::Media,
        Kind::Refresh,
        Kind::Sitemap,
        Kind::SitemapFile,
    ];

    /// The value the `links` table holds.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Anchor => "anchor",
            Kind::Image => "image",
            Kind::Canonical => "canonical",
            Kind::Stylesheet => "stylesheet",
            Kind::Icon => "icon",
            Kind::Link => "link",
            Kind::Script => "script",
            Kind::Frame => "frame",
            Kind::Media => "media",
            Kind::Refresh => "refresh",
            Kind::Sitemap => "sitemap",
            Kind::SitemapFile => "sitemap_file",
        }
    }
}

/// One reference on a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The attribute's decoded value, as written: for `srcset` one
    /// candidate's URL, for a refresh the URL in `content`, unquoted.
    pub raw: String,
    /// `raw` resolved against the page's URL or its `<base href>`, or in a
    /// local build an href beginning `/` against the site root (D53); `None`
    /// where it does not parse as a URL.
    pub url: Option<Url>,
    pub kind: Kind,
    /// What the profile's handlers tagged the link with (I6).
    pub tag: Option<String>,
    /// A refresh whose delay is 0: a browser leaves the page at once, so the
    /// crawler reads the page as an alias, one more hop (D7). False for
    /// every other link.
    pub immediate: bool,
}

/// What the link handlers collect over one pass.
pub(super) struct Links {
    page: Url,
    /// Where an href beginning `/` points, in a local build (D53).
    site_root: Option<Url>,
    base: Option<Url>,
    base_seen: bool,
    found: Vec<Link>,
    /// Where in `found` the element being read began, if it yielded any.
    mark: Option<usize>,
}

/// The elements that can name a URL.
const READ: &str = "a[href], area[href], img, source, link, script[src], iframe[src], \
                    video, audio, track, meta";

impl Links {
    pub(super) fn new(page: &Url, site_root: Option<&Url>) -> Links {
        Links {
            page: page.clone(),
            site_root: site_root.cloned(),
            base: None,
            base_seen: false,
            found: Vec::new(),
            mark: None,
        }
    }

    /// The handlers: one reading every element in [`READ`] and `<base>`, and,
    /// registered after the profile's so they have run, one tagging what it
    /// read.
    /// `read` goes before the profile's handlers and `tag` after.
    pub(super) fn handlers<'h>(
        this: &'h RefCell<Links>,
        sink: &'h RefCell<Sink>,
    ) -> (ElementHandlers<'h>, ElementHandlers<'h>) {
        let read = vec![
            element!("base[href]", move |el| {
                let mut state = this.borrow_mut();
                if !state.base_seen {
                    state.base_seen = true;
                    let href = attribute(el, "href").unwrap_or_default();
                    state.base = state.resolve(&href);
                }
                Ok(())
            }),
            element!(READ, move |el| {
                // A tag left by a handler on some element that is no link
                // is no link's.
                sink.borrow_mut().take_link_tag();
                let mut state = this.borrow_mut();
                let before = state.found.len();
                for (raw, kind) in references(el) {
                    state.push(raw, kind);
                }
                if state.found.len() > before && immediate_refresh(el) {
                    state.found[before].immediate = true;
                }
                Ok(())
            }),
        ];
        let tag = vec![element!(READ, move |_| {
            let tag = sink.borrow_mut().take_link_tag();
            let mut state = this.borrow_mut();
            if let (Some(from), Some(tag)) = (state.mark.take(), tag) {
                for link in &mut state.found[from..] {
                    link.tag = Some(tag.clone());
                }
            }
            Ok(())
        })];
        (read, tag)
    }

    fn push(&mut self, raw: String, kind: Kind) {
        let url = self.resolve(&raw);
        self.mark.get_or_insert(self.found.len());
        self.found.push(Link {
            raw,
            url,
            kind,
            tag: None,
            immediate: false,
        });
    }

    /// Where in [`Links::found`] the last link read sits.
    pub(super) fn last_index(&self) -> Option<usize> {
        self.found.len().checked_sub(1)
    }

    /// The links read so far.
    pub(super) fn found(&self) -> &[Link] {
        &self.found
    }

    /// `raw` resolved as a link read now would be.
    pub(super) fn resolve(&self, raw: &str) -> Option<Url> {
        // A protocol-relative `//host/` is not root-relative.
        if let (Some(root), Some(rest)) = (&self.site_root, raw.strip_prefix('/'))
            && !rest.starts_with('/')
        {
            return root.join(rest).ok();
        }
        self.base.as_ref().unwrap_or(&self.page).join(raw).ok()
    }

    pub(super) fn finish(self) -> Vec<Link> {
        self.found
    }
}

/// The references one element makes, in a fixed order per tag. An
/// empty `src`, `srcset` or `href` on anything but an anchor names no
/// resource, and a browser requests none.
fn references(el: &Element) -> Vec<(String, Kind)> {
    let get = |name: &str| attribute(el, name).filter(|v| !v.trim().is_empty());
    let mut out = Vec::new();
    match el.tag_name().as_str() {
        "a" | "area" => {
            // Unlike the rest, an empty `href` is the page itself.
            out.extend(attribute(el, "href").map(|v| (v, Kind::Anchor)));
        }
        "img" => {
            out.extend(get("src").map(|v| (v, Kind::Image)));
            out.extend(srcset(el).map(|v| (v, Kind::Image)));
        }
        "source" => {
            out.extend(get("src").map(|v| (v, Kind::Media)));
            out.extend(srcset(el).map(|v| (v, Kind::Image)));
        }
        "link" => {
            let kind = attribute(el, "rel").and_then(|rel| link_kind(&rel));
            out.extend(kind.zip(get("href")).map(|(kind, v)| (v, kind)));
        }
        "script" => out.extend(get("src").map(|v| (v, Kind::Script))),
        "iframe" => out.extend(get("src").map(|v| (v, Kind::Frame))),
        "video" => {
            out.extend(get("src").map(|v| (v, Kind::Media)));
            out.extend(get("poster").map(|v| (v, Kind::Media)));
        }
        "audio" | "track" => out.extend(get("src").map(|v| (v, Kind::Media))),
        "meta" => {
            let refresh = attribute(el, "http-equiv")
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("refresh"));
            if refresh {
                out.extend(
                    attribute(el, "content")
                        .and_then(|c| refresh_url(&c))
                        .map(|v| (v, Kind::Refresh)),
                );
            }
        }
        _ => {}
    }
    out
}

fn srcset(el: &Element) -> impl Iterator<Item = String> {
    attribute(el, "srcset")
        .map(|s| srcset_urls(&s))
        .unwrap_or_default()
        .into_iter()
}

/// The kind of a `<link>` by its `rel` tokens, `None` for one that names no
/// resource: no token, or only `preconnect` and `dns-prefetch`, which name an
/// origin.
fn link_kind(rel: &str) -> Option<Kind> {
    let tokens: Vec<String> = rel
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect();
    let has = |token: &str| tokens.iter().any(|t| t == token);
    if has("canonical") {
        Some(Kind::Canonical)
    } else if has("stylesheet") {
        Some(Kind::Stylesheet)
    } else if has("icon") {
        Some(Kind::Icon)
    } else if tokens
        .iter()
        .any(|t| t != "preconnect" && t != "dns-prefetch")
    {
        Some(Kind::Link)
    } else {
        None
    }
}

/// The URLs of a `srcset` value, as HTML parses it: a candidate is a run of
/// non-whitespace, whose trailing commas end it, else followed by a
/// descriptor up to a comma outside parentheses. A URL may hold commas.
fn srcset_urls(value: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut rest = value;
    loop {
        rest = rest.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
        if rest.is_empty() {
            return urls;
        }
        let end = rest
            .find(|c: char| c.is_ascii_whitespace())
            .unwrap_or(rest.len());
        let (url, after) = rest.split_at(end);
        rest = after;
        if url.ends_with(',') {
            let url = url.trim_end_matches(',');
            if !url.is_empty() {
                urls.push(url.to_string());
            }
            continue;
        }
        urls.push(url.to_string());
        let mut depth = 0u32;
        let stop = rest.char_indices().find(|&(_, c)| {
            match c {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => return true,
                _ => {}
            }
            false
        });
        rest = stop.map_or("", |(i, _)| &rest[i + 1..]);
    }
}

/// Whether `el` is a `<meta http-equiv=refresh>` whose delay is 0.
fn immediate_refresh(el: &Element) -> bool {
    let refresh =
        attribute(el, "http-equiv").is_some_and(|v| v.trim().eq_ignore_ascii_case("refresh"));
    refresh && attribute(el, "content").is_some_and(|c| refresh_delay(&c) == Some(0.0))
}

/// The delay in seconds a refresh's `content` starts with.
fn refresh_delay(content: &str) -> Option<f64> {
    let start = content.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let digits = start
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(start.len());
    start[..digits].parse().ok()
}

/// The URL of a `<meta http-equiv=refresh>`'s `content`, as HTML parses it:
/// a delay, a `;` or `,` or space, then the URL, after an optional `url=`,
/// and possibly quoted. `None` for a plain reload (`content="30"`) or content that does
/// not start with a delay.
fn refresh_url(content: &str) -> Option<String> {
    let ws = |c: char| c.is_ascii_whitespace();
    let start = content.trim_start_matches(ws);
    let after = start.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
    if after.len() == start.len() {
        return None;
    }
    if !after.is_empty() && !after.starts_with(|c: char| ws(c) || c == ';' || c == ',') {
        return None;
    }
    let mut rest = after.trim_start_matches(ws);
    rest = rest.strip_prefix([';', ',']).unwrap_or(rest);
    rest = rest.trim_start_matches(ws);
    if rest.get(..3).is_some_and(|k| k.eq_ignore_ascii_case("url"))
        && let Some(url) = rest[3..].trim_start_matches(ws).strip_prefix('=')
    {
        rest = url.trim_start_matches(ws);
    }
    let url = match rest.chars().next() {
        Some(q @ ('"' | '\'')) => {
            let inner = &rest[1..];
            &inner[..inner.find(q).unwrap_or(inner.len())]
        }
        _ => rest,
    };
    Some(url.to_string()).filter(|u| !u.is_empty())
}

#[cfg(test)]
mod tests;
