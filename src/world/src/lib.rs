pub mod error;
pub mod load_spawn_locations;

use crate::error::WorldError;
use crate::load_spawn_locations::load_spawn_locations;
use ahash::AHashMap;
use glam::Vec3A;
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
}

impl GameMap {
    pub fn new(save_name: &str) -> Result<Self, WorldError> {
        let installation_path = get_zomboid_install_dir()?;
        let user_dir = get_user_dir()?;

        Ok(GameMap {
            spawn_locations: load_spawn_locations(installation_path, user_dir, save_name)?,
        })
    }
}
