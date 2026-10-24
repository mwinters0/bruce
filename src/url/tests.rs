use super::*;

fn canon(s: &str) -> String {
    canonical(&Url::parse(s).unwrap(), false).0
}

#[test]
fn url_gives_the_host_port_and_path_their_form() {
    assert_eq!(
        canon("HTTPS://Docs.Example.ORG:443"),
        "https://docs.example.org/"
    );
    assert_eq!(canon("http://a.example:80/x"), "http://a.example/x");
    assert_eq!(canon("http://a.example:8080/x"), "http://a.example:8080/x");
    assert_eq!(
        canon("https://a.example/a/./b/../c"),
        "https://a.example/a/c"
    );
    assert_eq!(
        canon("https://BÜCHER.example/"),
        "https://xn--bcher-kva.example/"
    );
}

#[test]
fn a_trailing_slash_is_kept_apart() {
    assert_ne!(
        canon("https://a.example/foo"),
        canon("https://a.example/foo/")
    );
}

#[test]
fn an_unreserved_escape_is_decoded() {
    assert_eq!(
        canon("https://a.example/%7Euser/%41-%2e_%7e"),
        "https://a.example/~user/A-._~"
    );
    assert_eq!(
        canon("https://a.example/?q=%61%62"),
        "https://a.example/?q=ab"
    );
}

#[test]
fn any_other_escape_stays_with_its_hex_uppercased() {
    // A `%2F` stays an escape, so the path does not gain a segment.
    assert_eq!(canon("https://a.example/a%2fb"), "https://a.example/a%2Fb");
    assert_eq!(
        canon("https://a.example/a%20b?x=%26%3d"),
        "https://a.example/a%20b?x=%26%3D"
    );
    assert_eq!(
        canon("https://a.example/caf%c3%a9"),
        "https://a.example/caf%C3%A9"
    );
    assert_eq!(
        canon("https://a.example/caf\u{e9}"),
        "https://a.example/caf%C3%A9"
    );
}

#[test]
fn a_malformed_escape_is_left_as_written() {
    assert_eq!(
        canon("https://a.example/100%/%zz/%4"),
        "https://a.example/100%/%zz/%4"
    );
}

#[test]
fn a_decoded_dot_never_makes_a_segment_the_url_crate_kept() {
    // `url` already resolves `%2e%2e`, so decoding cannot move the path.
    assert_eq!(canon("https://a.example/a/%2e%2e/b"), "https://a.example/b");
    assert_eq!(
        canon("https://a.example/a/%2e%2e%2e/b"),
        "https://a.example/a/.../b"
    );
}

#[test]
fn the_fragment_is_not_part_of_it() {
    assert_eq!(canon("https://a.example/p#frag"), "https://a.example/p");
    assert_eq!(
        canon("https://a.example/p?q=1#frag"),
        "https://a.example/p?q=1"
    );
}

#[test]
fn the_query_is_kept_unless_the_root_strips_it() {
    let u = Url::parse("https://a.example/p?b=2&a=1").unwrap();
    assert_eq!(canonical(&u, false).as_str(), "https://a.example/p?b=2&a=1");
    assert_eq!(canonical(&u, true).as_str(), "https://a.example/p");
    // An empty query is a query.
    let u = Url::parse("https://a.example/p?").unwrap();
    assert_eq!(canonical(&u, false).as_str(), "https://a.example/p?");
    assert_eq!(canonical(&u, true).as_str(), "https://a.example/p");
}

#[test]
fn spellings_of_one_url_share_an_id_and_distinct_urls_do_not() {
    let id = |s: &str| canonical(&Url::parse(s).unwrap(), false).id();
    assert_eq!(
        id("https://A.example:443/%7Ex#a"),
        id("https://a.example/~x#b")
    );
    assert_ne!(id("https://a.example/a%2Fb"), id("https://a.example/a/b"));
    assert_ne!(id("https://a.example/x"), id("https://a.example/x/"));
    assert_ne!(id("https://a.example/x"), id("http://a.example/x"));
}

#[test]
fn the_id_is_xxh3_of_the_form_and_stable() {
    let c = canonical(&Url::parse("https://a.example/x").unwrap(), false);
    assert_eq!(c.id(), xxh3_64(b"https://a.example/x"));
    // Fixed so a change of hash or of form cannot pass silently: the IDs
    // are in every stored table (D26).
    assert_eq!(c.id(), 1_187_806_132_242_855_503);
}

#[test]
fn a_final_slash_is_toggled_and_the_query_kept() {
    let toggled = |s: &str| toggle_slash(&Url::parse(s).unwrap()).map(String::from);
    assert_eq!(
        toggled("https://a.example/foo"),
        Some("https://a.example/foo/".to_owned())
    );
    assert_eq!(
        toggled("https://a.example/foo/?q=1"),
        Some("https://a.example/foo?q=1".to_owned())
    );
    assert_eq!(
        toggled("https://a.example/foo?q=1"),
        Some("https://a.example/foo/?q=1".to_owned())
    );
    assert_eq!(toggled("https://a.example/"), None);
    assert_eq!(toggled("https://a.example"), None);
}
