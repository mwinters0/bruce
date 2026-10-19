use super::*;

fn table() -> Table {
    Table {
        columns: vec!["page".into(), "n".into(), "note".into()],
        rows: vec![
            vec![
                Cell::Text("https://e.test/a".into()),
                Cell::Int(12),
                Cell::Null,
            ],
            vec![Cell::Text("b".into()), Cell::Int(3), Cell::Bool(true)],
        ],
    }
}

#[test]
fn text_pads_columns_right_aligns_numbers_and_marks_a_missing_value() {
    assert_eq!(
        table().render(Format::Text),
        "page               n  note\n\
         https://e.test/a  12  -\n\
         b                  3  true"
    );
}

#[test]
fn text_with_no_rows_says_so() {
    let t = Table {
        columns: vec!["page".into()],
        rows: vec![],
    };
    assert_eq!(t.render(Format::Text), "page\n(none)");
}

#[test]
fn json_is_an_array_of_objects_with_typed_values() {
    assert_eq!(
        table().render(Format::Json),
        "[{\"page\":\"https://e.test/a\",\"n\":12,\"note\":null},\n \
         {\"page\":\"b\",\"n\":3,\"note\":true}]"
    );
    let none = Table {
        columns: vec!["page".into()],
        rows: vec![],
    };
    assert_eq!(none.render(Format::Json), "[]");
}

#[test]
fn json_escapes_what_a_string_may_not_hold() {
    assert_eq!(quoted("a\"b\\c\nd\u{1}é"), "\"a\\\"b\\\\c\\nd\\u0001é\"");
}

#[test]
fn a_database_value_becomes_the_nearest_cell() {
    assert_eq!(Cell::from(Value::UBigInt(7)), Cell::Int(7));
    assert_eq!(Cell::from(Value::USmallInt(404)), Cell::Int(404));
    assert_eq!(Cell::from(Value::Null), Cell::Null);
    assert_eq!(
        Cell::from(Value::UBigInt(u64::MAX)),
        Cell::Text(u64::MAX.to_string())
    );
}
