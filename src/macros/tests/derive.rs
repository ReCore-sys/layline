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
