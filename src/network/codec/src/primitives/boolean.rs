use crate::{NetDecode, NetEncode};
use bytes::{Buf, Bytes};

impl NetEncode for bool {
    fn encode(&self) -> Bytes {
        Bytes::from(vec![u8::from(*self)])
    }
}

impl NetDecode for bool {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        Some(bytes.get_u8() != 0)
    }
}
