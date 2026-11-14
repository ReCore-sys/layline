use derive_deref::{Deref, DerefMut};
use glam::IVec3;

// Just using a IVec3 since it has some nice extra methods and is easy to convert to/from other types. This is a wrapper struct to make it more clear what this is used for.
#[derive(Deref, DerefMut, Debug, Clone)]
pub struct SquareCoord(IVec3);

impl SquareCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }
}