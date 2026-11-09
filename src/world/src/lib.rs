pub mod error;
pub mod load_spawn_locations;

use crate::error::WorldError;
use crate::load_spawn_locations::load_spawn_locations;
use ahash::AHashMap;
use glam::Vec3A;
use layline_shared::locations::Location;
use layline_shared::profession::Profession;
use std::path::PathBuf;
use layline_shared::paths::get_zomboid_install_dir;

type ProfessionSpawnLocations = AHashMap<(Location, PathBuf), AHashMap<Profession, Vec<Vec3A>>>;

pub struct GameMap {
    pub spawn_locations: ProfessionSpawnLocations,
}

impl GameMap {
    pub fn new() -> Result<Self, WorldError> {
        let base_path = get_zomboid_install_dir()?;

        Ok(GameMap {
            spawn_locations: load_spawn_locations(base_path)?,
        })
    }
}
