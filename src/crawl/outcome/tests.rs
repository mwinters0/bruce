use super::*;

fn status(code: u16) -> Verdict {
    answered(code, &Body::Unread)
}

#[test]
fn a_status_is_classified_by_d16() {
    assert_eq!(status(200), (Outcome::Ok, None));
    assert_eq!(status(204), (Outcome::Ok, None));
    assert_eq!(status(404), (Outcome::BrokenUrl, None));
    assert_eq!(status(410), (Outcome::BrokenUrl, None));
    for refused in [401, 403, 429] {
        assert_eq!(status(refused), (Outcome::Error, None), "{refused}");
    }
    assert_eq!(status(500), (Outcome::Error, None));
    assert_eq!(status(503), (Outcome::Error, None));
    assert_eq!(status(302), (Outcome::BrokenUrl, Some("bad_redirect")));
}

#[test]
fn a_page_over_the_cap_answered_so_it_is_ok() {
    assert_eq!(
        answered(200, &Body::TooLarge),
        (Outcome::Ok, Some("too_large"))
    );
}

#[test]
fn only_a_missing_name_is_broken() {
    let failure = |kind| Failure {
        kind,
        detail: String::new(),
    };
    assert_eq!(
        failed(&failure(FailureKind::NxDomain)),
        (Outcome::BrokenUrl, Some("nxdomain"))
    );
    for kind in [
        FailureKind::Timeout,
        FailureKind::Tls,
        FailureKind::Connect,
        FailureKind::Reset,
        FailureKind::Other,
    ] {
        assert_eq!(
            failed(&failure(kind)),
            (Outcome::Error, Some(kind.as_str()))
        );
    }
}
