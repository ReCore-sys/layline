use crate::NetEncode;

#[derive(Clone, Debug)]
pub struct RawBytes(Vec<u8>);

impl From<Vec<u8>> for RawBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<RawBytes> for Vec<u8> {
    fn from(value: RawBytes) -> Self {
        value.0
    }
}

impl NetEncode for RawBytes {
    fn encode(&self) -> bytes::Bytes {
        self.0.clone().into()
    }
}