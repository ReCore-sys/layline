pub mod blending;
pub mod attachments;
pub mod square_coord;

use crate::error::WorldError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;
use attachments::Attachments;
use crate::chunks::saved_chunk::blending::Blending;

#[derive(Debug, Clone)]
pub struct SavedChunk {
    pub x: i32,
    pub y: i32,
    pub is_debug: bool,
    pub world_version: u16,
    pub blending: Blending,
    pub attachments: Attachments,
}

impl SavedChunk {
    pub fn load_from_disk(mut data: impl Read + Seek, x: i32, y: i32) -> Result<Self, WorldError> {

        let debug = data.read_i8()? != 0;
        let world_version = data.read_i32::<BigEndian>()? as u16;

        let data_length = data.read_i32::<BigEndian>()?;
        let expected_crc = data.read_i64::<BigEndian>()?;

        let current_pos = data.stream_position()?;
        // Should always be 17 since we've read (1 + 4 + 4 + 8) bytes but doesn't hurt to check
        assert_eq!(current_pos, 17);

        // Read the remaining data to calculate the CRC checksum
        let remaining_data_length = data_length as u64 - current_pos;
        let mut remaining_data = vec![0u8; remaining_data_length as usize];
        data.read_exact(&mut remaining_data)?;
        assert_eq!(remaining_data.len() as u64, remaining_data_length);

        let crc_checksum = crc32fast::hash(&remaining_data);
        if crc_checksum as i64 != expected_crc {
            return Err(WorldError::MismatchedChecksum(x, y, expected_crc, crc_checksum as i64));
        }

        // Reset the cursor to the position after reading the rest of the data for the CRC check
        data.seek(std::io::SeekFrom::Start(current_pos))?;

        let blending = Blending::read_blending(&mut data, world_version)?;

        let attachments = Attachments::read_attachments(&mut data, world_version)?;

        Ok(SavedChunk {
            x,
            y,
            is_debug: debug,
            world_version,
            blending,
            attachments
        })
    }
}

pub fn get_saved_chunks(save_dir: PathBuf) -> Result<Vec<SavedChunk>, WorldError> {
    let mut saved_chunks = Vec::new();
    let map_dir = save_dir.join("map");
    if !map_dir.exists() {
        return Err(WorldError::MissingMapDirectory(
            map_dir.to_string_lossy().to_string(),
        ));
    }
    for y_dir in std::fs::read_dir(map_dir)? {
        if let Ok(y_dir) = y_dir
            && y_dir.file_type()?.is_dir()
        {
            for x_file in std::fs::read_dir(y_dir.path())? {
                if let Ok(x_file) = x_file
                    && x_file.file_type()?.is_file()
                {
                    let path_buf = x_file.path();
                    let mut split_path = path_buf.components().rev();
                    let x = split_path
                        .next()
                        .expect("Failed to get x coordinate from path")
                        .as_os_str()
                        .to_str()
                        .expect("Failed to convert x coordinate to string")
                        .strip_suffix(".bin")
                        .expect("Failed to strip .bin from x coordinate")
                        .parse::<i32>()
                        .unwrap_or_else(|_| panic!("Failed to parse x coordinate as i32: {}",
                                                   path_buf.display()));
                    let y = split_path
                        .next()
                        .expect("Failed to get y coordinate from path")
                        .as_os_str()
                        .to_str()
                        .expect("Failed to convert y coordinate to string")
                        .parse::<i32>()
                        .expect("Failed to parse y coordinate as i32");

                    let file = BufReader::new(std::fs::File::open(&path_buf)?);

                    let saved_chunk = SavedChunk::load_from_disk(file, x, y)?;
                    saved_chunks.push(saved_chunk);
                }
            }
        }
    }
    Ok(saved_chunks)
}
