//! Correctness: each candidate's facts against the oracle's, on every fixture.
//! `cargo run --release` prints one line a page and a summary; `-- --dump
//! <page-substring>` prints one page's oracle facts as JSON.

use std::collections::BTreeMap;
use tokenizer_spike::{finalize, fixtures, gum, gum_cb, lol, oracle, Facts};

type Extract = fn(&str) -> tokenizer_spike::Raw;

const CANDIDATES: &[(&str, Extract)] =
    &[("html5gum", gum::extract), ("html5gum-cb", gum_cb::extract), ("lol_html", lol::extract)];

/// The first few differences between two lists, by position.
fn diff_list<T: serde::Serialize + PartialEq>(field: &str, want: &[T], got: &[T], out: &mut Vec<String>) {
    if want == got {
        return;
    }
    out.push(format!("  {field}: oracle {} vs {}", want.len(), got.len()));
    let mut shown = 0;
    for i in 0..want.len().max(got.len()) {
        let (w, g) = (want.get(i), got.get(i));
        if w != g && shown < 3 {
            shown += 1;
            let j = |x: Option<&T>| x.map(|v| serde_json::to_string(v).unwrap()).unwrap_or("-".into());
            out.push(format!("    [{i}] oracle {} | got {}", j(w), j(g)));
        }
    }
}

fn diff(want: &Facts, got: &Facts) -> Vec<String> {
    let mut out = Vec::new();
    let scalar = |f: &str, w: String, g: String, out: &mut Vec<String>| {
        if w != g {
            out.push(format!("  {f}: oracle {w} vs {g}"));
        }
    };
    scalar("generator", format!("{:?}", want.generator), format!("{:?}", got.generator), &mut out);
    scalar("profile", want.profile.into(), got.profile.into(), &mut out);
    scalar("component", format!("{:?}", want.component), format!("{:?}", got.component), &mut out);
    scalar("version", format!("{:?}", want.version), format!("{:?}", got.version), &mut out);
    scalar("edit_link", format!("{:?}", want.edit_link), format!("{:?}", got.edit_link), &mut out);
    diff_list("anchors", &want.anchors, &got.anchors, &mut out);
    diff_list("links", &want.links, &got.links, &mut out);
    diff_list("edit_candidates", &want.edit_candidates, &got.edit_candidates, &mut out);
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pages = fixtures(false);
    if args.get(1).map(String::as_str) == Some("--dump") {
        let (path, html) = pages.iter().find(|(p, _)| p.contains(&args[2])).expect("no such page");
        let f = finalize(oracle::extract(html));
        eprintln!("{path}");
        println!("{}", serde_json::to_string_pretty(&f).unwrap());
        return;
    }
    if args.get(1).map(String::as_str) == Some("--edge") {
        let (_, html) = EDGES.iter().find(|(c, _)| c.contains(&args[2])).expect("no such case");
        let j = |f: &Facts| serde_json::to_string(f).unwrap();
        println!("oracle: {}", j(&finalize(oracle::extract(html))));
        for (name, f) in CANDIDATES {
            println!("{name}: {}", j(&finalize(f(html))));
        }
        return;
    }
    let mut agree: BTreeMap<&str, usize> = BTreeMap::new();
    let (mut anchors, mut links) = (0, 0);
    for (path, html) in &pages {
        let want = finalize(oracle::extract(html));
        anchors += want.anchors.len();
        links += want.links.len();
        let placements = want.anchors.iter().fold(BTreeMap::<&str, usize>::new(), |mut m, a| {
            *m.entry(a.placement).or_default() += 1;
            m
        });
        println!(
            "{path}: {}, anchors {placements:?}, {} links, {} unresolved xrefs, edit {:?}",
            want.profile,
            want.links.len(),
            want.links.iter().filter(|l| l.unresolved_xref).count(),
            want.edit_link
        );
        for (name, f) in CANDIDATES {
            let d = diff(&want, &finalize(f(html)));
            if d.is_empty() {
                *agree.entry(name).or_default() += 1;
            } else {
                println!(" {name} differs:\n{}", d.join("\n"));
            }
        }
    }
    edges();
    println!("\n{} pages, {anchors} anchors, {links} links (oracle)", pages.len());
    for (name, _) in CANDIDATES {
        println!("{name}: {}/{} pages identical to the oracle", agree.get(name).unwrap_or(&0), pages.len());
    }
}

/// Markup the fixtures lack, where a tokenizer without a tree builder may part
/// from a browser. The oracle is html5ever's tree, so a disagreement here is
/// the candidate's unless the case says otherwise.
const EDGES: &[(&str, &str)] = &[
    ("script body", r#"<main><script>document.write('<a href="s" id="s">')</script></main>"#),
    ("style body", r#"<main><style>/* <div id="q"> */</style></main>"#),
    ("textarea body", r#"<main><textarea><a id="ta" href="t"></textarea></main>"#),
    ("title body", r#"<title><a id="ti"></title><main></main>"#),
    ("noscript body", r#"<main><noscript><a href="ns" id="n">x</a></noscript></main>"#),
    ("comment", r#"<main><!-- <a id="c" href="c"> --></main>"#),
    ("entities in href", r#"<main><a href="?a=1&amp;b=2&copy=3&notit;">x</a></main>"#),
    ("entity in id", r#"<main><h2 id="a&amp;b">x</h2></main>"#),
    ("entity in edit text", r#"<main><a href="e">Edit&nbsp;this&#32;page</a></main>"#),
    ("edit text in a child", r#"<main><a href="e"><span>Edit</span> this page</a></main>"#),
    ("uppercase", r#"<MAIN><A HREF="u" ID="U">x</A></MAIN>"#),
    ("duplicate attribute", r#"<main><a href="1" href="2" id="d" id="e">x</a></main>"#),
    ("a inside a", r#"<main><a href="1" title="Edit">one<a href="2">Edit two</a></a></main>"#),
    ("end tag closes children", r#"<main><div><h2 id="in"></main><h2 id="out"></h2>"#),
    ("unclosed p and li", r#"<main><p>x<ul><li>a<li><a id="l" href="l">b</a></ul><p id="p"></main><p id="o">"#),
    ("table foster parenting", r#"<main><table><a href="f" id="f">t</a><tr><td id="c">c</td></tr></table></main>"#),
    ("svg self-closing", r#"<main><svg><a href="v"/><g id="g"/></svg><h2 id="h"></h2></main><p id="o">"#),
    ("svg cdata", r#"<main><svg><![CDATA[ 1 > 0 <a id="cd" href="cd">]]></svg></main>"#),
    ("image becomes img", r#"<main><image src="i.png"></main>"#),
    ("template", r#"<main><template><h2 id="t"></h2></template></main>"#),
    ("srcset", r#"<main><img srcset="a.png 1x, b.png 2x"><picture><source srcset="c.webp 480w,d.webp"></picture></main>"#),
    ("name off a", r#"<main><div name="x"></div><a name="y"></a><a name=""></a><h2 id=""></h2></main>"#),
    ("plaintext", r#"<main><plaintext><a id="pt" href="pt"></main>"#),
    ("select ambiguity", r#"<main><select><xmp><script>"use strict";</script></select><h2 id="after"></h2></main>"#),
    ("role on article", r#"<p id="before"></p><article role="main" id="r"><h2 id="x"></h2></article>"#),
    ("antora region", r##"<meta name="generator" content="Antora 3.1.15"><div class="nav-container" data-component="c"></div><div class="edit-this-page"><a href="h" title="Page history"></a></div><div class="edit-this-page"><a href="e" title="Edit this Page"></a></div><article class="doc toc"><h2 id="a"></h2><a href="#x::y.adoc" class="xref unresolved">y</a></article><h2 id="b"></h2>"##),
];

fn edges() {
    println!("\nEdge cases (not in the fixtures):");
    let mut agree: BTreeMap<&str, usize> = BTreeMap::new();
    for (case, html) in EDGES {
        let want = finalize(oracle::extract(html));
        for (name, f) in CANDIDATES {
            let got = std::panic::catch_unwind(|| finalize(f(html)));
            match got {
                Ok(got) => {
                    let d = diff(&want, &got);
                    if d.is_empty() {
                        *agree.entry(name).or_default() += 1;
                    } else {
                        println!(" {case}: {name} differs:\n{}", d.join("\n"));
                    }
                }
                Err(_) => println!(" {case}: {name} panicked"),
            }
        }
    }
    for (name, _) in CANDIDATES {
        println!("{name}: {}/{} edge cases identical to the oracle", agree.get(name).unwrap_or(&0), EDGES.len());
    }
}
