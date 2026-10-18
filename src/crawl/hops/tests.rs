use crate::fetch::Body;

use super::*;

fn redirect(status: u16, location: Option<&str>) -> Response {
    Response {
        status,
        location: location.map(str::to_owned),
        content_type: None,
        body: Body::Unread,
    }
}

fn refresh(url: &str, immediate: bool) -> Link {
    Link {
        raw: url.to_owned(),
        url: Url::parse(url).ok(),
        kind: Kind::Refresh,
        tag: None,
        immediate,
    }
}

#[test]
fn a_location_is_resolved_against_the_url_asked_and_loses_its_fragment() {
    let from = Url::parse("https://example.org/a/b").unwrap();
    let to = |status, loc: Option<&str>| location(&from, &redirect(status, loc));
    assert_eq!(
        to(301, Some("../c#x")).as_ref().map(Url::as_str),
        Some("https://example.org/c")
    );
    assert_eq!(
        to(308, Some("https://other.org/"))
            .as_ref()
            .map(Url::as_str),
        Some("https://other.org/")
    );
    assert_eq!(to(302, None), None, "a 3xx with no Location leads nowhere");
    assert_eq!(to(302, Some("mailto:a@b.c")), None);
    assert_eq!(to(200, Some("/c")), None, "only a 3xx is a hop");
}

#[test]
fn a_chain_records_each_hop_with_the_response_it_gave() {
    let mut chain = Chain::new("https://e.org/a");
    chain.follow(301, false, "https://e.org/b").unwrap();
    chain.follow(200, true, "https://e.org/c").unwrap();
    assert_eq!(
        chain.into_hops(),
        [
            Hop {
                url: "https://e.org/a".into(),
                status: 301,
                refresh: false
            },
            Hop {
                url: "https://e.org/b".into(),
                status: 200,
                refresh: true
            },
        ]
    );
}

#[test]
fn a_chain_stops_at_a_loop_and_past_ten_hops() {
    let mut chain = Chain::new("https://e.org/a");
    chain.follow(302, false, "https://e.org/b").unwrap();
    assert_eq!(chain.follow(302, false, "https://e.org/a"), Err(Stop::Loop));

    let mut chain = Chain::new("https://e.org/0");
    for i in 1..=MAX_HOPS {
        chain
            .follow(301, false, &format!("https://e.org/{i}"))
            .unwrap();
    }
    assert_eq!(
        chain.follow(301, false, "https://e.org/more"),
        Err(Stop::TooMany)
    );
    assert_eq!(chain.into_hops().len(), MAX_HOPS);
}

#[test]
fn only_the_first_refresh_is_followed_and_only_at_delay_zero() {
    let alias = [refresh("https://e.org/real#frag", true)];
    assert_eq!(
        refresh_target(&alias).as_ref().map(Url::as_str),
        Some("https://e.org/real")
    );
    assert_eq!(refresh_target(&[refresh("https://e.org/x", false)]), None);
    assert_eq!(
        refresh_target(&[
            refresh("https://e.org/x", false),
            refresh("https://e.org/y", true)
        ]),
        None,
        "a browser obeys the first"
    );
    assert_eq!(refresh_target(&[]), None);
}
