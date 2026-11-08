use crate::{NetDecode, NetEncode};
use bytes::{Buf, Bytes};

impl NetEncode for u8 {
    fn encode(&self) -> Bytes {
        Bytes::from(vec![*self])
    }
}

impl NetDecode for u8 {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        Some(bytes.get_u8())
    }
}
