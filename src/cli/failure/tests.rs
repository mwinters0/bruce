use super::*;
use crate::store::{Error, Mismatch};

fn mismatch() -> Error {
    Error::SchemaVersion(Mismatch {
        found: "6".into(),
        expected: 7,
    })
}

#[test]
fn another_schema_version_is_status_3_and_any_other_failure_the_commands_own() {
    let schema = Failure::at("x.duckdb", &mismatch());
    assert_eq!(schema.status(1), SCHEMA);
    assert_eq!(schema.status(2), SCHEMA);
    assert_eq!(
        schema.to_string(),
        "x.duckdb: database has schema version 6, but this bruce reads version 7"
    );
    let other = Failure::at("x.duckdb", &Error::NotBruce);
    assert_eq!((other.status(1), other.status(2)), (1, 2));
    assert_eq!(Failure::from("no such file").status(2), 2);
}

#[test]
fn a_report_over_a_baseline_of_another_schema_is_status_3() {
    let db = Path::new("x.duckdb");
    let schema = Failure::report(db, &report::Error::Baseline(mismatch()));
    assert_eq!(schema.status(1), SCHEMA);
    let other = Failure::report(db, &report::Error::Baseline(Error::NotBruce));
    assert_eq!(other.status(1), 1);
    assert_eq!(
        Failure::report(db, &report::Error::NoFinishedRun).status(1),
        1
    );
}
