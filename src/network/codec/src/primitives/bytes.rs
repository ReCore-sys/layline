use crate::{NetDecode, NetEncode};
use bytes::Bytes;

impl NetEncode for Bytes {
    fn encode(&self) -> Bytes {
        self.clone()
    }
}

impl NetDecode for Bytes {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        Some(bytes.split_to(bytes.len()))
    }
}
