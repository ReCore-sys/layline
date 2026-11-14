pub mod error;
pub mod load_spawn_locations;
pub mod chunks;

use crate::error::WorldError;
use crate::load_spawn_locations::load_spawn_locations;
use ahash::AHashMap;
use glam::Vec3A;
use tracing::info;
use layline_shared::paths::{get_user_dir, get_zomboid_install_dir};
use layline_shared::profession::Profession;

pub enum SpawnPointsFile {
    File(String),
    ServerFile(String),
}

pub struct SpawnRegion {
    pub name: String,
    pub file: SpawnPointsFile,
    pub profession_points: AHashMap<Profession, Vec<Vec3A>>,
}

pub struct GameMap {
    pub spawn_locations: Vec<SpawnRegion>,
    pub saved_chunks: AHashMap<(i32, i32), chunks::saved_chunk::SavedChunk>,
}

impl GameMap {
    pub fn new(save_name: &str) -> Result<Self, WorldError> {
        let installation_path = get_zomboid_install_dir()?;
        let user_dir = get_user_dir()?;
        let save_dir = user_dir.join("Saves").join("Multiplayer").join(save_name);

        let saved_chunks = chunks::saved_chunk::get_saved_chunks(save_dir)?;

        info!("Loaded {} saved chunks for save '{}'", saved_chunks.len(), save_name);
        
        if let Some(first_chunk) = saved_chunks.first() {
            info!(
                "First saved chunk: {first_chunk:?}",
            );
        }

        Ok(GameMap {
            spawn_locations: load_spawn_locations(installation_path, user_dir, save_name)?,
            saved_chunks: saved_chunks
                .into_iter()
                .map(|chunk| ((chunk.x, chunk.y), chunk))
                .collect(),
        })
    }
}
