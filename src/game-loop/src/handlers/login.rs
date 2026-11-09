use crate::game_state::GameState;
use layline_codec::NetEncode;
use layline_codec::lua::lua_table::NetworkLuaTable;
use layline_macros::handler;
use layline_network::Connection;
use layline_packets::inbound::login::Login;
use layline_packets::outbound::connection_details::ConnectionDetails;
use layline_packets::outbound::request_data::{RequestData, RequestDataId};
use layline_packets::outbound::spawn_region::SpawnRegion;
use layline_world::SpawnPointsFile;
use tracing::{error, info, warn};

#[handler]
pub fn login(world: &mut GameState, connection: &Connection, packet: &mut Login) {
    info!("Login packet received from {}", packet.username);

    let lua = mlua::Lua::new();
    for (region_offset, spawn_region) in world.map.spawn_locations.iter().enumerate() {
        info!(
            "Sending spawn region {region_offset} ({}) to {}",
            spawn_region.name, packet.username
        );

        let table = (|| -> mlua::Result<NetworkLuaTable> {
            let region = lua.create_table()?;
            region.set("name", spawn_region.name.as_str())?;
            match &spawn_region.file {
                SpawnPointsFile::File(file) => region.set("file", file.as_str())?,
                SpawnPointsFile::ServerFile(file) => region.set("serverfile", file.as_str())?,
            }

            let points = lua.create_table()?;
            for (profession, spawn_points) in &spawn_region.profession_points {
                let profession_points = lua.create_table()?;
                for (point_offset, spawn_point) in spawn_points.iter().enumerate() {
                    let point = lua.create_table()?;
                    point.set("posX", spawn_point.x)?;
                    point.set("posY", spawn_point.y)?;
                    point.set("posZ", spawn_point.z)?;

                    let point_index =
                        i64::try_from(point_offset + 1).map_err(mlua::Error::external)?;
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
                    "Failed to build spawn region {}: {error}",
                    spawn_region.name
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

    let mut details = ConnectionDetails {
        max_players: world.config.max_players,
        world_dictionary: world.world_dictionary.clone(),
        ..Default::default()
    };
    if !world.config.map.is_empty() {
        details.game_map = world.config.map.clone();
    }

    let payload = details.encode();
    if payload.len() < 1024 {
        if let Err(error) = connection.send(RequestData::full_data(
            RequestDataId::ConnectionDetails,
            payload,
        )) {
            warn!("Failed to queue ConnectionDetails: {error}");
        }
        return;
    }

    const PART_SIZE: usize = 1024;
    const MAX_UNACKNOWLEDGED_SIZE: usize = 204_800;
    if payload.len() > MAX_UNACKNOWLEDGED_SIZE {
        warn!(
            "ConnectionDetails payload is {} bytes; transfers larger than one 200 KiB ACK window are not implemented",
            payload.len()
        );
        return;
    }

    let total_size = payload.len();
    let part_count = total_size.div_ceil(PART_SIZE);
    info!("Sending ConnectionDetails in {part_count} parts ({total_size} bytes)");
    for (part_index, part) in payload.chunks(PART_SIZE).enumerate() {
        let offset = part_index * PART_SIZE;
        let packet = RequestData::part_data(
            RequestDataId::ConnectionDetails,
            total_size,
            offset,
            part,
        );
        if let Err(error) = connection.send(packet) {
            warn!(
                "Failed to queue ConnectionDetails part {} of {part_count}: {error}",
                part_index + 1
            );
            return;
        }
    }
}
