use std::path::PathBuf;
use ahash::AHashMap;
use glam::Vec3A;
use mlua::Lua;
use tracing::info;
use layline_lua::load_layline_lua;
use layline_shared::locations::Location;
use layline_shared::profession::Profession;
use crate::error::WorldError;

pub(crate) fn load_spawn_locations(installation_path: PathBuf) -> Result<AHashMap<Location, AHashMap<Profession, Vec<Vec3A>>>, WorldError> {
    let mut spawn_locations = AHashMap::new();

    let mut lua_rt = Lua::new();

    load_layline_lua(&mut lua_rt)?;

    let mut counter = 0;

    let maps_dir = installation_path.join("media").join("maps");
    if !maps_dir.exists() {
        return Err(WorldError::InvalidGameLocation(installation_path));
    }
    for entry in std::fs::read_dir(maps_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        if folder_name == "challengemaps" {
            continue
        }
        if let Some(location) = Location::from_folder_name(&folder_name) {
            let spawn_locations_lua_path = entry.path().join("spawnpoints.lua");
            if spawn_locations_lua_path.exists() {
                let lua_code = std::fs::read_to_string(spawn_locations_lua_path)?;
                // Run the code to put the function in the lua globals
                lua_rt.load(&lua_code).exec()?;
                let spawnpoints_fn: mlua::Function = lua_rt.globals().get("SpawnPoints")?;
                let lua_table: mlua::Table = spawnpoints_fn.call(())?;
                let as_map = parse_spawn_locations(lua_table, &mut counter)?;
                spawn_locations.insert(location, as_map);
            } else {
                info!("No spawnpoints.lua found for {}", location);
            }
        } else {
            info!("Unknown location folder: {}", folder_name);
        }
    }

    info!("Loaded {} spawn locations", counter);


    Ok(spawn_locations)
}

fn parse_spawn_locations(lua_table: mlua::Table, counter: &mut u32) -> Result<AHashMap<Profession, Vec<Vec3A>>, WorldError> {
    let mut spawn_locations = AHashMap::new();
    for pair in lua_table.pairs::<String, mlua::Table>() {
        let (profession_str, location_table) = pair?;
        if let Some(profession) = Profession::from_id(&profession_str) {
            let mut locations = Vec::new();
            for pair in location_table.pairs::<String, mlua::Table>() {
                let (_, coords_table) = pair?;
                let x: f32 = coords_table.get("posX")?;
                let y: f32 = coords_table.get("posY")?;
                let z: f32 = coords_table.get("posZ")?;
                locations.push(Vec3A::new(x, y, z));
                *counter += 1;
            }
            spawn_locations.insert(profession, locations);
        } else {
            info!("Unknown profession in spawn locations: {}", profession_str);
        }
    }
    Ok(spawn_locations)
}