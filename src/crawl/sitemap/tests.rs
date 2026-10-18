use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;

use super::*;

const PAGES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<url><loc>https://docs.example.org/en-US/a/</loc><lastmod>2026-10-17T00:00:00Z</lastmod></url>
<url>
  <loc>
    https://docs.example.org/en-US/b/?x=1&amp;y=&#50;
  </loc>
</url>
<url><loc><![CDATA[https://docs.example.org/en-US/c/]]></loc></url>
<url><loc></loc></url>
</urlset>"#;

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes).unwrap();
    e.finish().unwrap()
}

#[test]
fn a_urlset_lists_its_pages_trimmed_and_unescaped() {
    assert_eq!(
        read(PAGES.as_bytes(), 1 << 20),
        Ok(Sitemap::Pages(vec![
            "https://docs.example.org/en-US/a/".to_owned(),
            "https://docs.example.org/en-US/b/?x=1&y=2".to_owned(),
            "https://docs.example.org/en-US/c/".to_owned(),
        ]))
    );
}

#[test]
fn an_index_lists_its_sitemaps_under_any_prefix() {
    let index = r#"<sm:sitemapindex xmlns:sm="http://www.sitemaps.org/schemas/sitemap/0.9">
        <sm:sitemap><sm:loc>https://docs.example.org/en-US/sitemap-a.xml</sm:loc></sm:sitemap>
        <sm:sitemap><sm:loc>https://docs.example.org/en-US/sitemap-b.xml.gz</sm:loc></sm:sitemap>
        </sm:sitemapindex>"#;
    assert_eq!(
        read(index.as_bytes(), 1 << 20),
        Ok(Sitemap::Index(vec![
            "https://docs.example.org/en-US/sitemap-a.xml".to_owned(),
            "https://docs.example.org/en-US/sitemap-b.xml.gz".to_owned(),
        ]))
    );
}

#[test]
fn a_gzip_file_is_told_by_its_bytes_and_capped_inflated() {
    let packed = gzip(PAGES.as_bytes());
    assert!(matches!(read(&packed, 1 << 20), Ok(Sitemap::Pages(p)) if p.len() == 3));
    assert_eq!(read(&packed, 100), Err(Unread::TooLarge));
    assert_eq!(
        read(&packed[..packed.len() / 2], 1 << 20),
        Err(Unread::NotASitemap),
        "a truncated file"
    );
}

#[test]
fn a_bom_and_an_empty_root_are_read() {
    let mut bom = b"\xef\xbb\xbf".to_vec();
    bom.extend_from_slice(PAGES.as_bytes());
    assert!(matches!(read(&bom, 1 << 20), Ok(Sitemap::Pages(p)) if p.len() == 3));
    assert_eq!(read(b"<urlset/>", 100), Ok(Sitemap::Pages(Vec::new())));
}

#[test]
fn anything_else_is_not_a_sitemap() {
    for body in [
        "<!DOCTYPE html><html><body>Not found</body></html>",
        "<rss><channel/></rss>",
        "plain text",
        "",
        "<urlset><url><loc>https://a.example/&nbsp;</loc></url></urlset>",
    ] {
        assert_eq!(
            read(body.as_bytes(), 1 << 20),
            Err(Unread::NotASitemap),
            "{body}"
        );
    }
}
