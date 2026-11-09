use std::path::{Path, PathBuf};

use ahash::AHashMap;
use glam::Vec3A;
use layline_lua::load_layline_lua;
use layline_shared::profession::Profession;
use mlua::{Function, Lua, Table, Value};
use tracing::{debug, info, warn};

use crate::error::WorldError;
use crate::{SpawnPointsFile, SpawnRegion};

pub(crate) fn load_spawn_locations(
    installation_path: PathBuf,
    user_dir: PathBuf,
    save_name: &str,
) -> Result<Vec<SpawnRegion>, WorldError> {
    let server_dir = user_dir.join("Server");
    let regions_path = server_dir.join(format!("{save_name}_spawnregions.lua"));
    let mut lua = Lua::new();
    load_layline_lua(&mut lua)?;

    lua.globals().set("SpawnRegions", Value::Nil)?;
    let regions_code = std::fs::read_to_string(&regions_path)?;
    lua.load(&regions_code).exec()?;
    let regions_fn: Function = lua.globals().get("SpawnRegions")?;
    let regions_table: Table = regions_fn.call(())?;

    let mut spawn_regions = Vec::new();
    let mut counter = 0;
    for region_result in regions_table.sequence_values::<Table>() {
        let region_table = region_result?;
        let name: String = region_table.get("name")?;
        let file: Option<String> = region_table.get("file")?;
        let serverfile: Option<String> = region_table.get("serverfile")?;

        let source = if let Some(file) = file {
            SpawnPointsFile::File(file)
        } else if let Some(serverfile) = serverfile {
            SpawnPointsFile::ServerFile(serverfile)
        } else {
            warn!("Skipping spawn region {name:?}: no file or serverfile specified");
            continue;
        };

        let points_path = match &source {
            SpawnPointsFile::File(file) => resolve_path(&installation_path, file),
            SpawnPointsFile::ServerFile(file) => resolve_path(&server_dir, file),
        };
        if !points_path.is_file() {
            warn!(
                "Skipping spawn region {name:?}: spawn-points file {} does not exist",
                points_path.display()
            );
            continue;
        }

        lua.globals().set("SpawnPoints", Value::Nil)?;
        let points_code = std::fs::read_to_string(&points_path)?;
        lua.load(&points_code).exec()?;
        let points_fn: Function = lua.globals().get("SpawnPoints")?;
        let points_table: Table = points_fn.call(())?;
        let profession_points = parse_spawn_locations(points_table, &mut counter)?;

        debug!(
            "Loaded spawn region {name:?} from {}",
            points_path.display()
        );
        spawn_regions.push(SpawnRegion {
            name,
            file: source,
            profession_points,
        });
    }

    info!(
        "Loaded {} spawn locations across {} regions",
        counter,
        spawn_regions.len()
    );
    Ok(spawn_regions)
}

fn resolve_path(base: &Path, file: &str) -> PathBuf {
    base.join(file.replace('\\', "/"))
}

fn parse_spawn_locations(
    lua_table: Table,
    counter: &mut u32,
) -> Result<AHashMap<Profession, Vec<Vec3A>>, WorldError> {
    let mut spawn_locations = AHashMap::new();
    for pair in lua_table.pairs::<String, Table>() {
        let (profession_str, location_table) = pair?;
        if let Some(profession) = Profession::from_id(&profession_str) {
            let mut locations = Vec::new();
            for pair in location_table.sequence_values::<Table>() {
                let coords_table = pair?;
                let pos_x: f32 = coords_table.get("posX")?;
                let pos_y: f32 = coords_table.get("posY")?;
                let pos_z: Option<f32> = coords_table.get("posZ")?;
                let world_x: Option<f32> = coords_table.get("worldX")?;
                let world_y: Option<f32> = coords_table.get("worldY")?;

                let (x, y) = match (world_x, world_y) {
                    (Some(world_x), Some(world_y)) => (
                        (world_x as i32 * 300 + pos_x as i32) as f32,
                        (world_y as i32 * 300 + pos_y as i32) as f32,
                    ),
                    (None, None) => (pos_x, pos_y),
                    _ => {
                        warn!(
                            "Skipping spawn point for {profession_str}: worldX and worldY must be specified together"
                        );
                        continue;
                    }
                };
                locations.push(Vec3A::new(x, y, pos_z.unwrap_or(0.0)));
                *counter += 1;
            }
            spawn_locations.insert(profession, locations);
        } else {
            info!("Unknown profession in spawn locations: {profession_str}");
        }
    }
    Ok(spawn_locations)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::SpawnPointsFile;

    #[test]
    fn loads_only_regions_listed_in_the_save_file() {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "layline-spawn-regions-{}-{unique_id}",
            std::process::id()
        ));
        let installation_dir = root.join("install");
        let server_dir = root.join("user/Server");
        let configured_map_file = installation_dir.join("media/maps/Selected Map/spawnpoints.lua");
        let unconfigured_map_file =
            installation_dir.join("media/maps/Unselected Map/spawnpoints.lua");
        let custom_points_file = server_dir.join("save1_spawnpoints.lua");
        std::fs::create_dir_all(configured_map_file.parent().unwrap()).unwrap();
        std::fs::create_dir_all(unconfigured_map_file.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&server_dir).unwrap();
        std::fs::write(
            server_dir.join("save1_spawnregions.lua"),
            r#"
                function SpawnRegions()
                    return {
                        { name = "Selected Map", file = "media/maps/Selected Map/spawnpoints.lua" },
                        { name = "Custom Region", serverfile = "save1_spawnpoints.lua" },
                    }
                end
            "#,
        )
        .unwrap();
        std::fs::write(
            &configured_map_file,
            "function SpawnPoints() return { unemployed = { { posX = 12, posY = 34 } } } end",
        )
        .unwrap();
        std::fs::write(
            &unconfigured_map_file,
            "function SpawnPoints() return { unemployed = { { posX = 99, posY = 99 } } } end",
        )
        .unwrap();
        std::fs::write(
            &custom_points_file,
            "function SpawnPoints() return { unemployed = { { worldX = 40, worldY = 22, posX = 67, posY = 101 } } } end",
        )
        .unwrap();

        let regions = load_spawn_locations(installation_dir, root.join("user"), "save1").unwrap();

        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].name, "Selected Map");
        assert_eq!(
            regions[0].profession_points[&Profession::Unemployed][0],
            Vec3A::new(12.0, 34.0, 0.0)
        );
        assert!(matches!(
            &regions[1].file,
            SpawnPointsFile::ServerFile(file) if file == "save1_spawnpoints.lua"
        ));
        assert_eq!(
            regions[1].profession_points[&Profession::Unemployed][0],
            Vec3A::new(12067.0, 6701.0, 0.0)
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}
