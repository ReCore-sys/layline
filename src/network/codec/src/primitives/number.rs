use crate::{NetDecode, NetEncode};
use bytes::{Buf, Bytes};

macro_rules! impl_number {
    ($($ty:ty => $read:ident),+ $(,)?) => {
        $(
            impl NetDecode for $ty {
                fn decode(bytes: &mut Bytes) -> Option<Self> {
                    bytes.$read().ok()
                }
            }

            impl NetEncode for $ty {
                fn encode(&self) -> Bytes {
                    Bytes::copy_from_slice(&self.to_be_bytes())
                }
            }
        )+
    };
}
impl_number!(
    i16 => try_get_i16,
    u16 => try_get_u16,
    i32 => try_get_i32,
    u32 => try_get_u32,
    i64 => try_get_i64,
    u64 => try_get_u64,
    f32 => try_get_f32,
    f64 => try_get_f64,
);
