use super::*;

#[test]
fn the_default_policy_is_what_bruce_has_always_judged_by() {
    assert!(Policy::default().pedantic_trailing_slash);
}

#[test]
fn every_setting_is_an_entry_with_its_value_written_out() {
    let mut policy = Policy::default();
    assert_eq!(
        policy.entries(),
        [("pedantic_trailing_slash", "true".to_string())]
    );
    policy.pedantic_trailing_slash = false;
    assert_eq!(
        policy.entries(),
        [("pedantic_trailing_slash", "false".to_string())]
    );
}
