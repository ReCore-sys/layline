pub mod error;
pub mod load_spawn_locations;

use crate::error::WorldError;
use ahash::AHashMap;
use glam::Vec3A;
use layline_shared::locations::Location;
use layline_shared::profession::Profession;
use crate::load_spawn_locations::load_spawn_locations;

pub struct GameMap {
    pub spawn_locations: AHashMap<Location, AHashMap<Profession, Vec<Vec3A>>>,
}

impl GameMap {
    pub fn new() -> Result<Self, WorldError> {
        let path = if let Some(path) = std::env::var_os("ZOMBOID_SERVER_INSTALL_PATH") {
            path.into()
        } else if let Ok(path) = std::env::current_dir() {
            path
        } else {
            return Err(WorldError::InvalidGameLocation(
                std::env::current_dir().unwrap_or_default(),
            ));
        };
        if ![path.join("media"), path.join("java"), path.join("jre64")]
            .iter()
            .all(|p| p.exists())
        {
            return Err(WorldError::InvalidGameLocation(path));
        };

        Ok(GameMap {
            spawn_locations: load_spawn_locations(path)?,
        })
    }
}
