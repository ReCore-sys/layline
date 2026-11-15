use std::io::Read;
use byteorder::ReadBytesExt;
use layline_shared::read_flags;
use crate::chunks::iso_chunk::square_coord::SquareCoord;

#[derive(Debug, Clone, Default)]
pub struct Attachments {
    pub attachments_done_full: bool,
    pub attachments_state: [bool; 4],
    pub attachments_partial: Option<Vec<SquareCoord>>
}

impl Attachments {
    pub(crate) fn read_attachments(
        data: &mut impl Read,
        world_version: u16,
    ) -> Result<Self, std::io::Error> {
        let mut attachments = Attachments {
            attachments_done_full: true,
            attachments_state: [true; 4],
            attachments_partial: None,
        };

        if world_version >= 214 {
            attachments.attachments_done_full = data.read_u8()? != 0;
            attachments.attachments_state = *read_flags(data.read_i8()?)
                .first_chunk::<4>()
                .expect("Could not convert 8 bools to 4 bools");
        }

        if world_version >= 221 {
            let count = data.read_i16::<byteorder::BigEndian>()?;
            if count > 0 {
                let mut partial_attachments = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let x = data.read_i32::<byteorder::BigEndian>()?;
                    let y = data.read_i32::<byteorder::BigEndian>()?;
                    let z = data.read_i32::<byteorder::BigEndian>()?;
                    partial_attachments.push(SquareCoord::new(x, y, z ));
                }
                attachments.attachments_partial = Some(partial_attachments);
            }
        }


        Ok(attachments)
    }
}