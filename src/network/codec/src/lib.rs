pub mod primitives;
pub mod lua;
pub mod prefixed_vec;

pub use bytes;
pub use raknet_v6::Reliability;
pub use prefixed_vec::{LengthPrefix, PrefixedVec};

use bytes::Bytes;

pub trait NetEncode {
    /// Encodes a value in the application's wire format.
    ///
    /// String encoding panics if its UTF-8 byte length exceeds 32767.
    fn encode(&self) -> Bytes;
}

pub trait NetDecode: Sized {
    /// Decodes a value from the supplied bytes.
    ///
    /// Decoding consumes the supplied buffer without returning any remainder.
    fn decode(bytes: &mut Bytes) -> Option<Self>;
}

/// A trait for types that can be serialized and deserialized as network packets.
pub trait Packet {
    const PACKET_ID: i16;
    const DIRECTION: PacketDirection;
    const PACKET_NAME: &'static str;
    const ORDERING_CHANNEL: u8;

    const RELIABILITY: Reliability;
}

pub enum PacketDirection {
    Inbound,
    Outbound,
}
