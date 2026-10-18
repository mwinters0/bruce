use super::*;

const LATIN1_E_ACUTE: u8 = 0xE9;

fn page(head: &str, tail: &[u8]) -> Vec<u8> {
    let mut bytes = head.as_bytes().to_vec();
    bytes.extend_from_slice(tail);
    bytes
}

#[test]
fn utf8_is_the_default() {
    assert_eq!(decode("é".as_bytes(), None), "é");
    assert_eq!(decode("é".as_bytes(), Some("text/html")), "é");
}

#[test]
fn invalid_bytes_become_the_replacement_character() {
    assert_eq!(decode(&[b'a', 0xFF, b'b'], None), "a\u{FFFD}b");
}

#[test]
fn a_bom_beats_the_header_and_is_dropped() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend("é".as_bytes());
    assert_eq!(decode(&bytes, Some("text/html; charset=iso-8859-1")), "é");

    let utf16: Vec<u8> = [0xFF, 0xFE, b'h', 0, b'i', 0].to_vec();
    assert_eq!(decode(&utf16, None), "hi");
}

#[test]
fn the_header_names_the_charset() {
    let bytes = [b'a', LATIN1_E_ACUTE];
    for header in [
        "text/html; charset=iso-8859-1",
        "text/html;CHARSET=\"ISO-8859-1\"",
        "text/html; charset = latin1; format=flowed",
    ] {
        assert_eq!(decode(&bytes, Some(header)), "aé", "{header}");
    }
}

#[test]
fn the_header_beats_a_meta() {
    let bytes = page("<meta charset=utf-8>", &[LATIN1_E_ACUTE]);
    assert!(decode(&bytes, Some("text/html; charset=windows-1252")).ends_with('é'));
}

#[test]
fn an_unknown_header_label_falls_through_to_the_meta() {
    let bytes = page("<meta charset=latin1>", &[LATIN1_E_ACUTE]);
    assert!(decode(&bytes, Some("text/html; charset=bogus")).ends_with('é'));
}

#[test]
fn a_meta_charset_names_the_charset() {
    for head in [
        "<!doctype html><html><head><meta charset=\"ISO-8859-1\">",
        "<META CHARSET='latin1'>",
        "<meta  charset = latin1 />",
        "<meta http-equiv=\"Content-Type\" content=\"text/html; charset=iso-8859-1\">",
        "<meta content='text/html; charset=latin1' http-equiv=content-type>",
    ] {
        let bytes = page(head, &[LATIN1_E_ACUTE]);
        assert!(decode(&bytes, None).ends_with('é'), "{head}");
    }
}

#[test]
fn a_meta_in_a_comment_or_without_a_charset_is_not_read() {
    for head in [
        "<!-- <meta charset=latin1> -->",
        "<meta name=viewport content=\"charset=latin1\">",
        "<meta http-equiv=refresh content=\"0; charset=latin1\">",
        "<p>charset=latin1</p>",
        "<a title=\"<meta charset=latin1>\">",
    ] {
        let bytes = page(head, &[LATIN1_E_ACUTE]);
        assert!(decode(&bytes, None).ends_with('\u{FFFD}'), "{head}");
    }
}

#[test]
fn the_scan_reads_only_the_first_1024_bytes() {
    let padding = " ".repeat(PRESCAN_BYTES);
    let bytes = page(
        &format!("{padding}<meta charset=latin1>"),
        &[LATIN1_E_ACUTE],
    );
    assert!(decode(&bytes, None).ends_with('\u{FFFD}'));
}

#[test]
fn a_meta_naming_utf16_means_utf8() {
    let bytes = page("<meta charset=utf-16le>", "é".as_bytes());
    assert!(decode(&bytes, None).ends_with('é'));
}

#[test]
fn a_truncated_tag_does_not_panic() {
    for head in [
        "<",
        "<meta",
        "<meta charset",
        "<meta charset=",
        "<meta charset=\"",
        "<a b='",
        "<!-- ",
    ] {
        decode(head.as_bytes(), None);
    }
}
