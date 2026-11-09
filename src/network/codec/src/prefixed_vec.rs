use crate::{NetDecode, NetEncode};
use bytes::{Bytes, BytesMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// An integer type used to encode the number of items in a [`PrefixedVec`].
///
/// The built-in implementations are `u8`, `u16`, `i16`, `u32` and `i32`.
pub trait LengthPrefix: NetEncode + NetDecode + Copy {
    /// Converts an item count to this prefix type, returning `None` if it does not fit.
    fn from_len(len: usize) -> Option<Self>;

    /// Converts this prefix value to an item count.
    fn to_len(self) -> Option<usize>;
}

macro_rules! impl_length_prefix {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl LengthPrefix for $ty {
                fn from_len(len: usize) -> Option<Self> {
                    Self::try_from(len).ok()
                }

                fn to_len(self) -> Option<usize> {
                    usize::try_from(self).ok()
                }
            }
        )+
    };
}

impl_length_prefix!(u8, u16, i16, u32, i32);

/// A vector whose item count is encoded before its items.
///
/// The prefix type defaults to `u32`; use a different type such as
/// `PrefixedVec<T, u16>` when the wire format uses a narrower count.
/// Encoding panics if the item count does not fit in the selected prefix type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixedVec<T, L = u32> {
    items: Vec<T>,
    prefix: PhantomData<L>,
}

impl<T, L> PrefixedVec<T, L> {
    /// Creates a prefixed vector from its items.
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            prefix: PhantomData,
        }
    }

    /// Returns the items as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Consumes the wrapper and returns its items.
    pub fn into_inner(self) -> Vec<T> {
        self.items
    }

    /// Appends an item.
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }
}

impl<T, L> From<Vec<T>> for PrefixedVec<T, L> {
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<T, L> From<PrefixedVec<T, L>> for Vec<T> {
    fn from(value: PrefixedVec<T, L>) -> Self {
        value.items
    }
}

impl<T, L> Default for PrefixedVec<T, L> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T, L> AsRef<[T]> for PrefixedVec<T, L> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L> Deref for PrefixedVec<T, L> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, L> DerefMut for PrefixedVec<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T: NetEncode, L: LengthPrefix> NetEncode for PrefixedVec<T, L> {
    fn encode(&self) -> Bytes {
        let len = L::from_len(self.items.len())
            .expect("vector length does not fit in its prefix type");
        let mut bytes = BytesMut::new();
        bytes.extend(len.encode());
        for item in &self.items {
            bytes.extend(item.encode());
        }
        bytes.freeze()
    }
}

impl<T: NetDecode, L: LengthPrefix> NetDecode for PrefixedVec<T, L> {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        let len = L::decode(bytes)?.to_len()?;
        let mut items = Vec::new();
        for _ in 0..len {
            items.push(T::decode(bytes)?);
        }
        Some(Self::new(items))
    }
}
