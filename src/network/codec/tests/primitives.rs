use bytes::Bytes;
use layline_codec::{NetDecode, NetEncode, PrefixedVec};

macro_rules! number_case {
    ($ty:ty, $value:expr, $bytes:expr) => {{
        let value: $ty = $value;
        let expected = Bytes::from_static($bytes);
        assert_eq!(value.encode(), expected);
        let mut exact = expected.clone();
        assert_eq!(<$ty>::decode(&mut exact), Some(value));
        assert!(exact.is_empty());
        let mut truncated = expected.slice(..expected.len() - 1);
        let original = truncated.clone();
        assert_eq!(<$ty>::decode(&mut truncated), None);
        assert_eq!(truncated, original);
        let mut extra = expected.to_vec();
        extra.extend_from_slice(&[0x5a, 0xa5]);
        let mut bytes = Bytes::from(extra);
        assert_eq!(<$ty>::decode(&mut bytes), Some(value));
        assert_eq!(bytes, Bytes::from_static(&[0x5a, 0xa5]));
    }};
}

#[test]
fn bytes_decode_one_field_at_a_time() {
    number_case!(i8, -2, &[0xfe]);
    number_case!(u8, 0xab, &[0xab]);
}

#[test]
fn numbers_use_big_endian_encoding() {
    number_case!(i16, -2, &[0xff, 0xfe]);
    number_case!(u16, 0xabcd, &[0xab, 0xcd]);
    number_case!(i32, -2, &[0xff, 0xff, 0xff, 0xfe]);
    number_case!(u32, 0x12345678, &[0x12, 0x34, 0x56, 0x78]);
    number_case!(i64, -2, &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe]);
    number_case!(
        u64,
        0x0123456789abcdef,
        &[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
    );
    number_case!(f32, 1.0, &[0x3f, 0x80, 0x00, 0x00]);
    number_case!(f64, 1.0, &[0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn floating_point_bit_patterns_are_preserved() {
    let value = f32::from_bits(0x7fc01234);
    assert_eq!(
        f32::decode(&mut value.encode()).unwrap().to_bits(),
        value.to_bits()
    );
    let value = f64::from_bits(0x8000000000000000);
    assert_eq!(
        f64::decode(&mut value.encode()).unwrap().to_bits(),
        value.to_bits()
    );
}

#[test]
fn booleans_encode_as_zero_or_one_and_decode_nonzero_as_true() {
    assert_eq!(false.encode(), Bytes::from_static(&[0]));
    assert_eq!(true.encode(), Bytes::from_static(&[1]));
    assert_eq!(bool::decode(&mut Bytes::from_static(&[0])), Some(false));
    assert_eq!(bool::decode(&mut Bytes::from_static(&[0xff])), Some(true));
    assert_eq!(bool::decode(&mut Bytes::new()), None);
    let mut bytes = Bytes::from_static(&[1, 0]);
    assert_eq!(bool::decode(&mut bytes), Some(true));
    assert_eq!(bytes, Bytes::from_static(&[0]));
}

#[test]
fn strings_use_a_signed_byte_length_and_utf8() {
    for (value, encoded) in [
        ("", &b"\x00\x00"[..]),
        ("A", &b"\x00\x01A"[..]),
        ("é", &b"\x00\x02\xc3\xa9"[..]),
        ("127.0.0.1", &b"\x00\x09127.0.0.1"[..]),
    ] {
        assert_eq!(value.encode().as_ref(), encoded);
        assert_eq!(value.to_string().encode().as_ref(), encoded);
        let mut bytes = encoded.to_vec();
        bytes.extend_from_slice(&[0x5a, 0xa5]);
        let mut bytes = Bytes::from(bytes);
        assert_eq!(String::decode(&mut bytes).as_deref(), Some(value));
        assert_eq!(bytes, Bytes::from_static(&[0x5a, 0xa5]));
    }

    let max = "a".repeat(i16::MAX as usize);
    assert_eq!(String::decode(&mut max.encode()), Some(max));
}

#[test]
fn strings_reject_bad_lengths_and_replace_invalid_utf8() {
    for bytes in [&b""[..], &b"\x00"[..], &b"\xff\xff"[..], &b"\x00\x02a"[..]] {
        let mut input = Bytes::copy_from_slice(bytes);
        assert_eq!(String::decode(&mut input), None);
        assert_eq!(input.as_ref(), bytes);
    }
    assert_eq!(
        String::decode(&mut Bytes::from_static(b"\x00\x01\xff")).as_deref(),
        Some("\u{fffd}"),
    );
}

#[test]
fn prefixed_vectors_use_the_selected_count_width() {
    let default_prefix = PrefixedVec::<u8>::new(vec![0x12, 0x34]);
    assert_eq!(
        default_prefix.encode(),
        Bytes::from_static(b"\x00\x00\x00\x02\x12\x34")
    );

    let strings = PrefixedVec::<String, u32>::new(vec!["a".into(), "bc".into()]);
    let expected = Bytes::from_static(b"\x00\x00\x00\x02\x00\x01a\x00\x02bc");
    assert_eq!(strings.encode(), expected);
    let mut bytes = expected.clone();
    assert_eq!(PrefixedVec::<String, u32>::decode(&mut bytes), Some(strings));
    assert!(bytes.is_empty());

    let values = PrefixedVec::<u8, u16>::new(vec![0xff, 0x01]);
    let expected = Bytes::from_static(b"\x00\x02\xff\x01");
    assert_eq!(values.encode(), expected);
    let mut bytes = expected;
    assert_eq!(PrefixedVec::<u8, u16>::decode(&mut bytes), Some(values));
}

#[test]
fn prefixed_vectors_reject_truncated_items() {
    let mut bytes = Bytes::from_static(b"\x00\x02\x01");
    assert_eq!(PrefixedVec::<u8, u16>::decode(&mut bytes), None);
}

#[test]
#[should_panic(expected = "vector length does not fit in its prefix type")]
fn prefixed_vector_encoding_rejects_an_oversized_count() {
    PrefixedVec::<u8, u8>::new(vec![0; usize::from(u8::MAX) + 1]).encode();
}

#[test]
fn fields_can_be_decoded_sequentially() {
    let mut bytes = Bytes::from_static(b"\x00\x02hi\x12\x34\xff\x02\x7f");
    assert_eq!(String::decode(&mut bytes).as_deref(), Some("hi"));
    assert_eq!(u16::decode(&mut bytes), Some(0x1234));
    assert_eq!(i8::decode(&mut bytes), Some(-1));
    assert_eq!(bool::decode(&mut bytes), Some(true));
    assert_eq!(u8::decode(&mut bytes), Some(0x7f));
    assert!(bytes.is_empty());
}

#[test]
#[should_panic(expected = "string exceeds the maximum UTF-8 byte length")]
fn string_encoding_rejects_oversized_values() {
    "a".repeat(i16::MAX as usize + 1).encode();
}
