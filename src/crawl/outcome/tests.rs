use super::*;

fn status(code: u16) -> Verdict {
    answered(code, &Body::Unread, false)
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
        answered(200, &Body::TooLarge, false),
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

#[test]
fn a_toggled_slash_that_answers_names_a_broken_4xx_and_nothing_else() {
    let slash = |code| answered(code, &Body::Unread, true);
    assert_eq!(slash(404), (Outcome::BrokenUrl, Some(WRONG_TRAILING_SLASH)));
    for other in [200, 302, 403, 429, 500] {
        assert_eq!(slash(other), status(other), "{other}");
    }
}
