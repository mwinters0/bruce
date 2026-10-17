//! P1.2's tokenizer spike: one extraction workload, written three times.
//!
//! `gum` (html5gum) and `lol` (lol_html) are the candidates (D11); `oracle`
//! runs the same workload over html5ever's tree, a browser-grade parse, and is
//! the correctness reference. Each fills a [`Raw`] in document order, and
//! [`finalize`] turns it into [`Facts`] the same way for all three, so a
//! difference between two `Facts` is a difference in what the tokenizer saw.
//!
//! The workload, from the P1 spec's "P1.2 Tokenizer spike":
//! - core: ids, `name` on `<a>` (D12), links with their kinds (D13);
//! - a prototype Antora extract: generator meta, the nav container's data
//!   attributes, the edit link chosen by `title` (I8, I12), the
//!   `article.doc` region (D34), unresolved-xref tagging (I6);
//! - the generic heuristics: landmarks (D34) and ranked edit-link candidates
//!   (D36).
//!
//! Thrown away once D11 names the winner.

use serde::Serialize;

pub mod gum;
pub mod gum_cb;
pub mod lol;
pub mod oracle;

/// What one element-level observation recorded, before profile choice.
#[derive(Debug, Default)]
pub struct Raw {
    pub generator: Option<String>,
    pub nav: Option<(Option<String>, Option<String>)>,
    pub anchors: Vec<RawAnchor>,
    pub links: Vec<Link>,
    /// Every `<a href>`, for the edit-link heuristics.
    pub a_elems: Vec<RawA>,
    pub has_article_doc: bool,
    pub has_main: bool,
}

#[derive(Debug, Default)]
pub struct RawAnchor {
    pub value: String,
    pub tag: String,
    pub attr: &'static str,
    pub in_article_doc: bool,
    pub in_main: bool,
}

#[derive(Debug, Default)]
pub struct RawA {
    pub href: String,
    pub rel: String,
    pub class: String,
    pub title: String,
    /// The parent element's class attribute, for `.edit-this-page > a`.
    pub parent_class: String,
    /// Descendant text, whitespace-collapsed when finalized.
    pub text: String,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct Link {
    pub kind: &'static str,
    pub url: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rel: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unresolved_xref: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct Anchor {
    pub value: String,
    pub tag: String,
    pub attr: &'static str,
    pub placement: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct Candidate {
    pub rank: u8,
    pub href: String,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Default)]
pub struct Facts {
    pub generator: Option<String>,
    pub profile: &'static str,
    pub component: Option<String>,
    pub version: Option<String>,
    pub anchors: Vec<Anchor>,
    pub links: Vec<Link>,
    /// The core's ranked candidates (D36), best first, ties in document order.
    pub edit_candidates: Vec<Candidate>,
    /// The edit link after the profile's override, if any.
    pub edit_link: Option<String>,
}

/// D35: `Antora …` in the generator meta picks `antora`, anything else `generic`.
pub fn profile_of(generator: Option<&str>) -> &'static str {
    match generator {
        Some(g) if g.starts_with("Antora ") => "antora",
        _ => "generic",
    }
}

pub fn has_class(class: &str, token: &str) -> bool {
    class.split_ascii_whitespace().any(|t| t == token)
}

/// The `srcset` candidates' URLs: each comma-separated candidate's first token.
pub fn srcset_urls(srcset: &str) -> impl Iterator<Item = &str> {
    srcset
        .split(',')
        .filter_map(|c| c.split_ascii_whitespace().next())
}

/// The links an element carries (D13), pushed in attribute order the oracle
/// also uses: `href`/`src` first, then `srcset`.
pub fn push_links(
    out: &mut Vec<Link>,
    tag: &str,
    get: impl Fn(&str) -> Option<String>,
) {
    let mut push = |kind: &'static str, url: &str, rel: String, unresolved_xref: bool| {
        out.push(Link { kind, url: url.to_owned(), rel, unresolved_xref })
    };
    match tag {
        "a" => {
            if let Some(h) = get("href") {
                let class = get("class").unwrap_or_default();
                let unresolved = has_class(&class, "xref") && has_class(&class, "unresolved");
                push("anchor", &h, String::new(), unresolved);
            }
        }
        "link" => {
            if let Some(h) = get("href") {
                push("link", &h, get("rel").unwrap_or_default(), false);
            }
        }
        "img" | "source" => {
            let kind = if tag == "img" { "image" } else { "source" };
            if let Some(s) = get("src") {
                push(kind, &s, String::new(), false);
            }
            if let Some(ss) = get("srcset") {
                for u in srcset_urls(&ss) {
                    push(kind, u, String::new(), false);
                }
            }
        }
        "script" | "iframe" => {
            if let Some(s) = get("src") {
                push(tag_kind(tag), &s, String::new(), false);
            }
        }
        _ => {}
    }
}

fn tag_kind(tag: &str) -> &'static str {
    if tag == "script" { "script" } else { "iframe" }
}

/// The anchors an element carries (D12): a non-empty `id` on anything, a
/// non-empty `name` on `<a>` only.
pub fn anchor_values(tag: &str, id: Option<String>, name: Option<String>) -> Vec<(String, &'static str)> {
    let mut v = Vec::new();
    if let Some(id) = id.filter(|s| !s.is_empty()) {
        v.push((id, "id"));
    }
    if tag == "a" {
        if let Some(n) = name.filter(|s| !s.is_empty()) {
            v.push((n, "name"));
        }
    }
    v
}

pub fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// D36's prototype ranking: 1 `rel=edit`; 2 an edit class on the link or its
/// parent; 3 a `title` starting "Edit"; 4 text starting "Edit". 0 is no
/// candidate.
fn rank(a: &RawA) -> u8 {
    let edit_class = |c: &str| c.split_ascii_whitespace().any(|t| t.contains("edit"));
    if a.rel.split_ascii_whitespace().any(|t| t.eq_ignore_ascii_case("edit")) {
        1
    } else if edit_class(&a.class) || edit_class(&a.parent_class) {
        2
    } else if a.title.to_ascii_lowercase().starts_with("edit") {
        3
    } else if collapse(&a.text).to_ascii_lowercase().starts_with("edit") {
        4
    } else {
        0
    }
}

pub fn finalize(raw: Raw) -> Facts {
    let profile = profile_of(raw.generator.as_deref());
    let antora = profile == "antora";
    let (has_region, in_region): (bool, fn(&RawAnchor) -> bool) = if antora {
        (raw.has_article_doc, |a| a.in_article_doc)
    } else {
        (raw.has_main, |a| a.in_main)
    };
    let anchors = raw
        .anchors
        .iter()
        .map(|a| Anchor {
            value: a.value.clone(),
            tag: a.tag.clone(),
            attr: a.attr,
            placement: if !has_region {
                "unknown"
            } else if in_region(a) {
                "content"
            } else {
                "chrome"
            },
        })
        .collect();
    let links = raw
        .links
        .into_iter()
        .map(|mut l| {
            l.unresolved_xref &= antora;
            l
        })
        .collect();
    let mut edit_candidates: Vec<Candidate> = raw
        .a_elems
        .iter()
        .map(|a| Candidate { rank: rank(a), href: a.href.clone() })
        .filter(|c| c.rank > 0)
        .collect();
    edit_candidates.sort_by_key(|c| c.rank);
    let core_pick = edit_candidates.first().map(|c| c.href.clone());
    let antora_pick = antora
        .then(|| {
            let in_etp = |a: &&RawA| has_class(&a.parent_class, "edit-this-page");
            raw.a_elems
                .iter()
                .filter(in_etp)
                .find(|a| a.title == "Edit this Page")
                .or_else(|| raw.a_elems.iter().filter(in_etp).find(|a| collapse(&a.text) == "Edit this Page"))
                .map(|a| a.href.clone())
        })
        .flatten();
    let (component, version) = raw.nav.unwrap_or((None, None));
    Facts {
        generator: raw.generator,
        profile,
        component: component.filter(|_| antora),
        version: version.filter(|_| antora),
        anchors,
        links,
        edit_candidates,
        edit_link: antora_pick.or(core_pick),
    }
}

/// Every fixture page, as (repo-relative path, contents).
pub fn fixtures(docs_fp_o_only: bool) -> Vec<(String, String)> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures");
    let mut out = Vec::new();
    let mut walk = vec![std::path::PathBuf::from(root)];
    while let Some(dir) = walk.pop() {
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk.push(p);
            } else if p.extension().is_some_and(|x| x == "html") {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                if !docs_fp_o_only || rel.starts_with("docs-fp-o/") {
                    out.push((format!("fixtures/{rel}"), std::fs::read_to_string(&p).unwrap()));
                }
            }
        }
    }
    out.sort();
    out
}
