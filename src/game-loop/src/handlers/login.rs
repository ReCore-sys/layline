use crate::game_state::GameState;
use layline_codec::lua::lua_table::NetworkLuaTable;
use layline_macros::handler;
use layline_network::Connection;
use layline_packets::inbound::login::Login;
use layline_packets::outbound::spawn_region::SpawnRegion;
use tracing::{error, info, warn};

#[handler]
pub fn login(world: &mut GameState, connection: &Connection, packet: &mut Login) {
    info!("Login packet received from {}", packet.username);

    let lua = mlua::Lua::new();
    let mut regions = world
        .map
        .spawn_locations
        .iter()
        .collect::<Vec<_>>();
    regions.sort_by(|(left, _), (right, _)| left.1.cmp(&right.1));

    for (region_offset, ((_, path), profession_points)) in regions.into_iter().enumerate() {
        let Some(region_name) = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
        else {
            error!("Could not determine spawn region name from {}", path.display());
            return;
        };
        let file = format!("media/maps/{region_name}/spawnpoints.lua");

        info!("Sending spawn region {region_offset} ({region_name}) to {}", packet.username);

        let table = (|| -> mlua::Result<NetworkLuaTable> {
            let region = lua.create_table()?;
            region.set("name", region_name)?;
            region.set("file", file)?;

            let points = lua.create_table()?;
            for (profession, spawn_points) in profession_points {
                let profession_points = lua.create_table()?;
                for (point_offset, spawn_point) in spawn_points.iter().enumerate() {
                    let point = lua.create_table()?;
                    point.set("posX", spawn_point.x)?;
                    point.set("posY", spawn_point.y)?;
                    point.set("posZ", spawn_point.z)?;

                    let point_index = i64::try_from(point_offset + 1)
                        .map_err(mlua::Error::external)?;
                    profession_points.set(point_index, point)?;
                }
                points.set(profession.to_id(), profession_points)?;
            }
            region.set("points", points)?;
            NetworkLuaTable::from_lua_table(&region)
        })();

        let table = match table {
            Ok(table) => table,
            Err(error) => {
                error!(
                    "Failed to build spawn region from {}: {error}",
                    path.display()
                );
                return;
            }
        };

        let index = match i32::try_from(region_offset + 1) {
            Ok(index) => index,
            Err(error) => {
                error!("Too many spawn regions to send: {error}");
                return;
            }
        };

        if let Err(error) = connection.send(SpawnRegion { index, table }) {
            warn!("Failed to queue SpawnRegion {index}: {error}");
            return;
        }
    }
}
