pub mod attachments;
pub mod blending;
pub mod square_coord;
pub mod blood_splat;

use std::cmp::min;
use crate::chunks::iso_chunk::blending::Blending;
use crate::chunks::iso_chunk_level::IsoChunkLevel;
use crate::error::WorldError;
use attachments::Attachments;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;
use layline_config::Config;
use crate::chunks::iso_chunk::blood_splat::IsoFloorBloodSplat;

#[derive(Debug, Clone, Default)]
pub struct IsoChunk {
    pub x: i32,
    pub y: i32,
    pub is_debug: bool,
    pub world_version: u16,
    pub blending: Blending,
    pub attachments: Attachments,
    pub level_min: i32,
    pub level_max: i32,
    pub levels: Vec<IsoChunkLevel>,
    pub blood_splats: Vec<IsoFloorBloodSplat>,
    pub next_splat_index: u16
}

impl IsoChunk {
    pub fn load_from_disk(mut data: impl Read + Seek, x: i32, y: i32, config: &Config) -> Result<Self, WorldError> {
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
            return Err(WorldError::MismatchedChecksum(
                x,
                y,
                expected_crc,
                crc_checksum as i64,
            ));
        }

        // Reset the cursor to the position after reading the rest of the data for the CRC check
        data.seek(std::io::SeekFrom::Start(current_pos))?;

        let blending = Blending::read_blending(&mut data, world_version)?;

        let attachments = Attachments::read_attachments(&mut data, world_version)?;

        let (level_min, level_max) = if world_version <= 206 {
            let level_min = data.read_i32::<BigEndian>()?;
            let level_max = data.read_i32::<BigEndian>()?;
            (level_min, level_max)
        } else {
            (0, 7)
        };

        // This is actually way more complicated in the java stuff but this'll do for now
        let levels = (level_min..=level_max)
            .map(|l| IsoChunkLevel::new((x, y), l))
            .collect();

        let blood_splat_lifetime = config.blood_splat_lifespan_days;
        // This should be dynamic but that's not set up yet
        let world_age_hours = 12;

        let blood_splat_counter = data.read_i32::<BigEndian>()?;
        let mut blood_splats = Vec::with_capacity(blood_splat_counter as usize);
        let mut next_splat_index = 0;
        for _ in 0..blood_splat_counter {
            let mut splat = IsoFloorBloodSplat::read_blood_splat(&mut data)?;
            splat.world_age = (world_age_hours as f32).min(splat.world_age);
            if blood_splat_lifetime <= 0 || !(world_age_hours as f32 - splat.world_age >= (blood_splat_lifetime * 24) as f32) && splat.splat_type < 8{
                next_splat_index = splat.index % 10;
            }
            blood_splats.push(splat);

        }

        Ok(IsoChunk {
            x,
            y,
            is_debug: debug,
            world_version,
            blending,
            attachments,
            level_max,
            level_min,
            levels,
            blood_splats,
            next_splat_index
        })
    }
}

pub fn get_saved_chunks(save_dir: PathBuf, config: &Config) -> Result<Vec<IsoChunk>, WorldError> {
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
                        .unwrap_or_else(|_| {
                            panic!(
                                "Failed to parse x coordinate as i32: {}",
                                path_buf.display()
                            )
                        });
                    let y = split_path
                        .next()
                        .expect("Failed to get y coordinate from path")
                        .as_os_str()
                        .to_str()
                        .expect("Failed to convert y coordinate to string")
                        .parse::<i32>()
                        .expect("Failed to parse y coordinate as i32");

                    let file = BufReader::new(std::fs::File::open(&path_buf)?);

                    let saved_chunk = IsoChunk::load_from_disk(file, x, y, config)?;
                    saved_chunks.push(saved_chunk);
                }
            }
        }
    }
    Ok(saved_chunks)
}
