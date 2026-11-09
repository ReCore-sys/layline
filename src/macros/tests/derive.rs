use bytes::Bytes;
use layline_codec::{NetDecode, NetEncode};

#[derive(Debug, PartialEq, layline_macros::NetEncode, layline_macros::NetDecode)]
struct Sample {
    name: String,
    count: i32,
    ratio: f32,
}

#[derive(Debug, PartialEq, layline_macros::NetEncode, layline_macros::NetDecode)]
struct Empty;

#[derive(Debug, PartialEq, layline_macros::NetEncode, layline_macros::NetDecode)]
struct Wrapper(i16, String);

#[derive(Debug, PartialEq, layline_macros::NetEncode, layline_macros::NetDecode)]
struct Conditional {
    has_value: bool,
    #[conditional(has_value)]
    value: Option<i16>,
    tail: u8,
}

#[derive(Debug, PartialEq, layline_macros::NetEncode, layline_macros::NetDecode)]
#[starting_index = 1]
enum Fieldless {
    First,
    Second,
}

#[derive(Debug, PartialEq, layline_macros::NetEncode, layline_macros::NetDecode)]
enum Message {
    Empty,
    Tuple(u8, String),
    Named { value: u16 },
    Nested(Wrapper),
}

#[test]
fn fields_are_encoded_in_declaration_order() {
    let sample = Sample {
        name: "a".to_string(),
        count: 7,
        ratio: 1.5,
    };
    let mut expected = b"\x00\x01a".to_vec();
    expected.extend_from_slice(&7_i32.to_be_bytes());
    expected.extend_from_slice(&1.5_f32.to_be_bytes());
    assert_eq!(sample.encode(), Bytes::from(expected));
}

#[test]
fn derived_structs_round_trip() {
    let sample = Sample {
        name: "zomboid".to_string(),
        count: -3,
        ratio: 0.25,
    };
    let mut bytes = sample.encode();
    assert_eq!(Sample::decode(&mut bytes), Some(sample));
    assert!(bytes.is_empty());

    let wrapper = Wrapper(9, "x".to_string());
    let mut bytes = wrapper.encode();
    assert_eq!(Wrapper::decode(&mut bytes), Some(wrapper));
}

#[test]
fn truncated_input_fails_to_decode() {
    let mut bytes = Bytes::from_static(b"\x00\x01a\x00\x00");
    assert_eq!(Sample::decode(&mut bytes), None);
}

#[test]
fn empty_structs_use_no_bytes() {
    assert!(Empty.encode().is_empty());
    assert_eq!(Empty::decode(&mut Bytes::new()), Some(Empty));
}

#[test]
fn conditional_fields_are_omitted_when_the_selector_is_false() {
    let packet = Conditional {
        has_value: false,
        value: None,
        tail: 4,
    };

    assert_eq!(packet.encode(), Bytes::from_static(b"\x00\x04"));
    let mut bytes = packet.encode();
    assert_eq!(Conditional::decode(&mut bytes), Some(packet));
    assert!(bytes.is_empty());
}

#[test]
fn conditional_fields_are_encoded_when_the_selector_is_true() {
    let packet = Conditional {
        has_value: true,
        value: Some(0x1234),
        tail: 4,
    };

    assert_eq!(packet.encode(), Bytes::from_static(b"\x01\x12\x34\x04"));
    let mut bytes = packet.encode();
    assert_eq!(Conditional::decode(&mut bytes), Some(packet));
    assert!(bytes.is_empty());
}

#[test]
fn fieldless_enum_indices_start_at_the_configured_value() {
    let mut first = Fieldless::First.encode();
    assert_eq!(first, Bytes::from_static(b"\x01"));
    assert_eq!(Fieldless::decode(&mut first), Some(Fieldless::First));

    let mut second = Fieldless::Second.encode();
    assert_eq!(second, Bytes::from_static(b"\x02"));
    assert_eq!(Fieldless::decode(&mut second), Some(Fieldless::Second));

    assert_eq!(Fieldless::decode(&mut Bytes::from_static(b"\x00")), None);
}

#[test]
fn enum_variants_encode_tags_and_payloads_in_declaration_order() {
    let values = [
        (Message::Empty, b"\x00".as_slice()),
        (Message::Tuple(7, "x".to_string()), b"\x01\x07\x00\x01x"),
        (Message::Named { value: 0x1234 }, b"\x02\x12\x34"),
        (
            Message::Nested(Wrapper(9, "x".to_string())),
            b"\x03\x00\x09\x00\x01x",
        ),
    ];

    for (value, expected) in values {
        let encoded = value.encode();
        assert_eq!(encoded, Bytes::from_static(expected));
        let mut bytes = encoded;
        assert_eq!(Message::decode(&mut bytes), Some(value));
        assert!(bytes.is_empty());
    }
}
