use crate::chunks::saved_chunk::blending::blend_direction::BlendDirection;
use byteorder::ReadBytesExt;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Blending {
    pub blending_done_full: bool,
    pub blending_done_partial: bool,
    pub blending_modified: [bool; 4],
    pub blending_depth: Vec<BlendDirection>,
}

impl Blending {
    pub(crate) fn read_blending(
        data: &mut impl Read,
        world_version: u16,
    ) -> Result<Self, std::io::Error> {
        let mut blending = Blending {
            blending_done_full: false,
            blending_done_partial: false,
            blending_modified: [false; 4],
            blending_depth: Vec::new(),
        };
        if world_version >= 209 {
            blending.blending_done_full = data.read_u8()? != 0;
        }

        if world_version >= 210 {
            blending.blending_modified = *layline_shared::read_flags(data.read_i8()?)
                .first_chunk::<4>()
                .expect("Could not convert 8 bools to 4 bools");

            blending.blending_done_partial = data.read_u8()? != 0;

            if blending.blending_modified != [true; 4] && blending.blending_done_partial {
                for _ in 0..4 {
                    let direction = match data.read_i8()? {
                        0 => BlendDirection::NORTH,
                        1 => BlendDirection::SOUTH,
                        2 => BlendDirection::WEST,
                        3 => BlendDirection::EAST,
                        _ => panic!("Invalid blending direction"),
                    };
                    blending.blending_depth.push(direction);
                }
            }
        }

        Ok(blending)
    }
}

mod blend_direction {
    #[derive(Debug, Clone)]
    pub struct BlendDirection {
        pub x: i32,
        pub y: i32,
        pub plane_x: i32,
        pub plane_y: i32,
        pub index: i32,
        pub opposite: i32,
        pub default_depth: i8,
    }

    impl BlendDirection {
        pub const NORTH: Self = Self {
            x: 0,
            y: -1,
            plane_x: 0,
            plane_y: 1,
            index: 0,
            opposite: 1,
            default_depth: 7,
        };
        pub const SOUTH: Self = Self {
            x: 0,
            y: 1,
            plane_x: 0,
            plane_y: 1,
            index: 1,
            opposite: 0,
            default_depth: 0,
        };
        pub const WEST: Self = Self {
            x: -1,
            y: 0,
            plane_x: 1,
            plane_y: 0,
            index: 2,
            opposite: 3,
            default_depth: 7,
        };
        pub const EAST: Self = Self {
            x: 1,
            y: 0,
            plane_x: 1,
            plane_y: 0,
            index: 3,
            opposite: 2,
            default_depth: 0,
        };
    }
}
