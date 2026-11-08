use crate::{NetDecode, NetEncode};
use bytes::{Buf, BufMut, Bytes, BytesMut};

impl NetEncode for str {
    fn encode(&self) -> Bytes {
        let length =
            i16::try_from(self.len()).expect("string exceeds the maximum UTF-8 byte length");
        let mut bytes = BytesMut::with_capacity(2 + self.len());
        bytes.put_i16(length);
        bytes.extend_from_slice(self.as_bytes());
        bytes.freeze()
    }
}

impl NetEncode for String {
    fn encode(&self) -> Bytes {
        self.as_str().encode()
    }
}

impl NetDecode for String {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        let length = i16::from_be_bytes(bytes.get(..2)?.try_into().ok()?);
        let length = usize::try_from(length).ok()?;

        if bytes.len() < 2 + length {
            return None;
        }

        bytes.advance(2);
        let string_bytes = bytes.split_to(length);
        Some(String::from_utf8_lossy(&string_bytes).into_owned())
    }
}
