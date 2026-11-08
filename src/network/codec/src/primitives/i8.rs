use crate::{NetDecode, NetEncode};
use bytes::{Buf, Bytes};

impl NetEncode for i8 {
    fn encode(&self) -> Bytes {
        Bytes::from(vec![*self as u8])
    }
}

impl NetDecode for i8 {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        Some(bytes.get_i8())
    }
}
