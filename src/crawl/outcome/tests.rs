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
    assert_eq!(status(302), (Outcome::BrokenUrl, Some(Reason::BadRedirect)));
}

#[test]
fn a_page_over_the_cap_answered_so_it_is_ok() {
    assert_eq!(
        answered(200, &Body::TooLarge, false),
        (Outcome::Ok, Some(Reason::TooLarge))
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
        (Outcome::BrokenUrl, Some(Reason::Nxdomain))
    );
    for kind in [
        FailureKind::Timeout,
        FailureKind::Tls,
        FailureKind::Connect,
        FailureKind::Reset,
        FailureKind::Other,
    ] {
        assert_eq!(failed(&failure(kind)), (Outcome::Error, kind.reason()));
    }
}

#[test]
fn a_toggled_slash_that_answers_names_a_broken_4xx_and_nothing_else() {
    let slash = |code| answered(code, &Body::Unread, true);
    assert_eq!(
        slash(404),
        (Outcome::BrokenUrl, Some(Reason::WrongTrailingSlash))
    );
    for other in [200, 302, 403, 429, 500] {
        assert_eq!(slash(other), status(other), "{other}");
    }
}

#[test]
fn the_default_status_map_of_the_policy_is_the_class_given_here() {
    // A run records the map whole (D78), and the macro reads it over the
    // outcome stored: the default must change no code's outcome.
    for (code, answer) in &crate::config::Policy::default().status {
        let (outcome, reason) = status(*code);
        assert_eq!(
            (outcome.as_str(), reason),
            (answer.as_str(), None),
            "{code}"
        );
    }
}

#[test]
fn a_cause_names_the_reason_it_writes() {
    use crate::crawl::hops::Stop;
    assert_eq!(FailureKind::NxDomain.reason(), Some(Reason::Nxdomain));
    assert_eq!(FailureKind::Other.reason(), Some(Reason::Other));
    // The crawl writes nothing for a request it stopped (D16).
    assert_eq!(FailureKind::Stopped.reason(), None);
    assert_eq!(Stop::Loop.reason(), Reason::RedirectLoop);
    assert_eq!(Stop::TooMany.reason(), Reason::TooManyRedirects);
}
