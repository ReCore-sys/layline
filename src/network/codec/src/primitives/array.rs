use crate::{NetDecode, NetEncode};
use bytes::{Bytes, BytesMut};

impl<T: NetEncode, const N: usize> NetEncode for [T; N] {
    fn encode(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        for value in self {
            bytes.extend(value.encode());
        }
        bytes.freeze()
    }
}

impl<T: NetDecode, const N: usize> NetDecode for [T; N] {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        let mut values = Vec::with_capacity(N);
        for _ in 0..N {
            values.push(T::decode(bytes)?);
        }
        values.try_into().ok()
    }
}
