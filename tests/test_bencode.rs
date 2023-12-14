use bittorrent_starter_rust::bencode_format::*;
use serde_json::{json, Value};

#[test]
fn test_derive_error() {
    // PartialEq + Eq
    assert_eq!(ParseError::new("foo"), ParseError::new("foo"));
    assert_ne!(ParseError::new("foo"), ParseError::new("bar"));

    // Debug
    assert_eq!(
        format!("{:?}", ParseError::new("foo")),
        "ParseError(\"foo\")"
    );

    // Display + Error
    assert_eq!(
        format!("{}", ParseError::new("foo")),
        "Bencode parse error: foo"
    );
}

#[test]
fn test_derive_text() {
    // PartialEq + Eq
    assert_eq!(BencodeText::new(b"hello"), BencodeText::new(b"hello"));
    assert_ne!(BencodeText::new(b"hello"), BencodeText::new(b"world"));

    // Debug
    assert_eq!(
        format!("{:?}", BencodeText::new(b"hello")),
        "BencodeText([104, 101, 108, 108, 111])"
    );

    // Ord
    assert!(BencodeText::new(b"hello") < BencodeText::new(b"world"));
}

#[test]
fn test_derive_value() {
    // PartialEq + Eq
    assert_eq!(BencodeValue::Integer(42), BencodeValue::Integer(42));
    assert_ne!(BencodeValue::Integer(42), BencodeValue::Integer(72));

    // Debug
    assert_eq!(format!("{:?}", BencodeValue::Integer(42)), "Integer(42)");
}

fn check(input: &str, expected: Value, rem_len: usize) {
    let (rem, parsed) = BencodeValue::parse(input.as_bytes()).unwrap();
    let value: Value = parsed.into();
    assert_eq!(value, expected);
    assert_eq!(rem.len(), rem_len);
}

fn check_err(input: &str, expected: &str) {
    let err = BencodeValue::parse(input.as_bytes()).unwrap_err();
    assert_eq!(err, ParseError::new(expected));
}

#[test]
fn test_parse_invalid_content() {
    check_err("", "Input is empty: []");
    check_err("h", "Invalid Bencode content: [104]");
}

#[test]
fn test_parse_string_valid() {
    check("0:", json!(""), 0); // Simple case
    check("5:hello", json!("hello"), 0); // Simple case
    check("5:hello world", json!("hello"), 6); // With extra
    check("12:hello world !", json!("hello world "), 1); // Multiple bytes len
}

#[test]
fn test_parse_string_invalid() {
    check_err("4e", "String num is not a number=101: [52, 101]");
    check_err("42", "String num does not contains end tag: [52, 50]");
    check_err("42:", "String payload is too short: []");
    check_err("2:h", "String payload is too short: [104]");
}

#[test]
fn test_parse_int_valid() {
    check("i0e", json!(0), 0);
    check("i42e", json!(42), 0);
    check("i89461eHello", json!(89461), 5);

    check("i-0e", json!(0), 0);
    check("i-9e", json!(-9), 0);
    check("i-89461eHello", json!(-89461), 5);
}

#[test]
fn test_parse_int_invalid() {
    check_err("ie", "Number cannot be empty: [101]");
    check_err("i-e", "Number cannot be empty: [101]");
    check_err("ixe", "String num is not a number=120: [120, 101]");
    check_err("i42", "String num does not contains end tag: [52, 50]");
}

#[test]
fn test_parse_list_valid() {
    check("le", json!([]), 0);
    check("l1:ae", json!(["a"]), 0);
    check("l1:ai42ee", json!(["a", 42]), 0);
    check("l4:spam4:eggse", json!(["spam", "eggs"]), 0);
    check(
        "ll3:foo3:barelei56el5:helloee",
        json!([["foo", "bar"], [], 56, ["hello"]]),
        0,
    );
}

#[test]
fn test_parse_list_invalid() {
    check_err("l", "List miss end tag: []");
    check_err("l3:foo2:bare", "Invalid Bencode content: [114, 101]");
}

#[test]
fn test_parse_dict_valid() {
    check("de", json!({}), 0);
    check(
        "d3:cow3:moo4:spam4:eggse",
        json!({"cow": "moo", "spam": "eggs"}),
        0,
    );
    check(
        "d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee",
        json!({ "publisher": "bob", "publisher-webpage": "www.example.com", "publisher.location": "home" } ),
        0,
    );
    check("d3:foolee", json!({"foo": []}), 0);
    check("d3:fooli42ee3:bari-8ee", json!({"foo": [42], "bar": -8}), 0);
}

#[test]
fn test_parse_dict_invalid() {
    check_err("d", "Dict miss end tag: []");
    check_err("d3:foode", "Dict miss end tag: []");
    check_err("dlee", "String num is not a number=108: [108, 101, 101]");
    check_err("d3:fooe", "Invalid Bencode content: [101]");
}
