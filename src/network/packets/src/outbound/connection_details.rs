use bytes::{Bytes, BytesMut};
use layline_codec::raw_bytes::RawBytes;
use layline_codec::{NetEncode, PrefixedVec};
use layline_macros::NetEncode;

#[derive(Debug, Clone, NetEncode)]
pub struct CoopSteamHostDetails {
    pub steam_id: i64,
    pub server_name: String,
}

#[derive(Debug, Clone, NetEncode)]
pub struct WorkshopItem {
    pub id: i64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, NetEncode)]
pub struct ModInfo {
    pub id: String,
    pub workshop_id: String,
    pub name: String,
}

#[derive(Debug, Clone, NetEncode)]
pub struct ServerOption {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, NetEncode)]
pub struct RoleColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Default for RoleColor {
    fn default() -> Self {
        Self {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        }
    }
}

#[derive(Debug, Clone, NetEncode)]
pub struct ConnectionRole {
    pub name: String,
    pub description: String,
    pub color: RoleColor,
    pub position: i32,
    pub capabilities: PrefixedVec<u8, u8>,
    pub is_read_only: bool,
}

impl Default for ConnectionRole {
    fn default() -> Self {
        Self {
            name: "User".to_string(),
            description: String::new(),
            color: RoleColor::default(),
            position: -1,
            capabilities: PrefixedVec::default(),
            is_read_only: false,
        }
    }
}

#[derive(Debug, Clone, NetEncode)]
pub struct SandboxOptions {
    pub magic: RawBytes,
    pub world_version: i32,
    pub format_version: i32,
    pub options: PrefixedVec<ServerOption, i32>,
    pub preset: String,
}

impl Default for SandboxOptions {
    fn default() -> Self {
        Self {
            magic: RawBytes::from(b"SAND".to_vec()),
            world_version: 249,
            format_version: 6,
            options: PrefixedVec::default(),
            preset: "Apocalypse".to_string(),
        }
    }
}

#[derive(Debug, Clone, NetEncode)]
pub struct GameTime {
    pub multiplier: f32,
    pub nights_survived: i32,
    pub target_zombies: i32,
    pub last_time_of_day: f32,
    pub time_of_day: f32,
    pub day: i32,
    pub month: i32,
    pub year: i32,
    pub reserved_float_1: f32,
    pub reserved_float_2: f32,
    pub reserved_int: i32,
    pub has_mod_data: bool,
    #[conditional(has_mod_data)]
    pub mod_data: Option<Bytes>,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            multiplier: 1.0,
            nights_survived: 0,
            target_zombies: 0,
            last_time_of_day: 8.0,
            time_of_day: 8.0,
            day: 1,
            month: 7,
            year: 1993,
            reserved_float_1: 0.0,
            reserved_float_2: 0.0,
            reserved_int: 0,
            has_mod_data: false,
            mod_data: None,
        }
    }
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct ErosionSeeds {
    pub main: [i32; 3],
    pub moisture: [i32; 3],
    pub minerals: [i32; 3],
    pub kudzu: [i32; 3],
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct ErosionTime {
    pub tick_unit: i32,
    pub ticks: i32,
    pub elapsed_ticks: i32,
    pub epoch: i32,
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct ErosionSeason {
    pub latitude: i32,
    pub temp_max: i32,
    pub temp_min: i32,
    pub temp_diff: i32,
    pub season_lag: i32,
    pub noon: f32,
    pub seeds: [i32; 3],
    pub monthly_values: [f32; 12],
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct ErosionConfig {
    pub seeds: ErosionSeeds,
    pub time: ErosionTime,
    pub season: ErosionSeason,
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct GlobalObjectSystem {
    pub name: String,
    pub has_initial_state: bool,
    #[conditional(has_initial_state)]
    pub initial_state: Option<Bytes>,
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct GlobalObjects {
    pub systems: PrefixedVec<GlobalObjectSystem, u8>,
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct LengthPrefixedBytes {
    length: i32,
    bytes: Bytes,
}

impl From<Bytes> for LengthPrefixedBytes {
    fn from(bytes: Bytes) -> Self {
        let length = i32::try_from(bytes.len()).expect("byte block exceeds the i32 length limit");
        Self { length, bytes }
    }
}

#[derive(Debug, Clone, Default, NetEncode)]
pub struct WorldDictionary {
    pub data: LengthPrefixedBytes,
}

impl WorldDictionary {
    pub fn from_raw_data(raw_data: Bytes) -> Self {
        let sentinel = "DICTIONARY_PACKET_END".encode();
        let mut data = BytesMut::with_capacity(raw_data.len() + sentinel.len());
        data.extend(raw_data);
        data.extend(sentinel);

        Self {
            data: LengthPrefixedBytes::from(data.freeze()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionDetails {
    pub is_coop_host: bool,
    pub max_players: i32,
    pub coop_steam_host_details: Option<CoopSteamHostDetails>,
    pub player_index: u8,
    pub role: ConnectionRole,
    pub game_map: String,
    /// Controls whether the conditional workshop list is included in the wire record.
    pub steam_mode: bool,
    pub workshop_items: PrefixedVec<WorkshopItem, i16>,
    pub mods: PrefixedVec<ModInfo, i32>,
    pub start_location: [i32; 3],
    pub server_options: PrefixedVec<ServerOption, i32>,
    pub sandbox_options: SandboxOptions,
    pub game_time: GameTime,
    pub erosion: ErosionConfig,
    pub global_objects: GlobalObjects,
    pub reset_id: i32,
    pub poisonous_berry: String,
    pub poisonous_mushroom: String,
    pub world_dictionary: WorldDictionary,
}

#[derive(NetEncode)]
struct ConnectionDetailsHeader {
    is_coop_host: bool,
    max_players: i32,
    has_coop_steam_host_details: bool,
    #[conditional(has_coop_steam_host_details)]
    coop_steam_host_details: Option<CoopSteamHostDetails>,
    player_index: u8,
    role: ConnectionRole,
    game_map: String,
}

#[derive(NetEncode)]
struct ConnectionDetailsTail {
    mods: PrefixedVec<ModInfo, i32>,
    start_location: [i32; 3],
    server_options: PrefixedVec<ServerOption, i32>,
    sandbox_options: SandboxOptions,
    game_time: GameTime,
    erosion: ErosionConfig,
    global_objects: GlobalObjects,
    reset_id: i32,
    poisonous_berry: String,
    poisonous_mushroom: String,
    world_dictionary: WorldDictionary,
}

impl Default for ConnectionDetails {
    fn default() -> Self {
        Self {
            is_coop_host: false,
            max_players: 16,
            coop_steam_host_details: None,
            player_index: 0,
            role: ConnectionRole::default(),
            game_map: "Muldraugh, KY".to_string(),
            steam_mode: false,
            workshop_items: PrefixedVec::default(),
            mods: PrefixedVec::default(),
            start_location: [10_745, 9_412, 0],
            server_options: PrefixedVec::default(),
            sandbox_options: SandboxOptions::default(),
            game_time: GameTime::default(),
            erosion: ErosionConfig::default(),
            global_objects: GlobalObjects::default(),
            reset_id: 0,
            poisonous_berry: String::new(),
            poisonous_mushroom: String::new(),
            world_dictionary: WorldDictionary::default(),
        }
    }
}

impl NetEncode for ConnectionDetails {
    fn encode(&self) -> Bytes {
        let header = ConnectionDetailsHeader {
            is_coop_host: self.is_coop_host,
            max_players: self.max_players,
            has_coop_steam_host_details: self.coop_steam_host_details.is_some(),
            coop_steam_host_details: self.coop_steam_host_details.clone(),
            player_index: self.player_index,
            role: self.role.clone(),
            game_map: self.game_map.clone(),
        }
        .encode();
        let workshop_items = if self.steam_mode {
            self.workshop_items.encode()
        } else {
            Bytes::new()
        };
        let tail = ConnectionDetailsTail {
            mods: self.mods.clone(),
            start_location: self.start_location,
            server_options: self.server_options.clone(),
            sandbox_options: self.sandbox_options.clone(),
            game_time: self.game_time.clone(),
            erosion: self.erosion.clone(),
            global_objects: self.global_objects.clone(),
            reset_id: self.reset_id,
            poisonous_berry: self.poisonous_berry.clone(),
            poisonous_mushroom: self.poisonous_mushroom.clone(),
            world_dictionary: self.world_dictionary.clone(),
        }
        .encode();

        let mut bytes = BytesMut::with_capacity(header.len() + workshop_items.len() + tail.len());
        bytes.extend(header);
        bytes.extend(workshop_items);
        bytes.extend(tail);
        bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_raw_world_dictionary_data_with_sentinel_and_length() {
        let raw_data = Bytes::from_static(&[0x01, 0x02, 0x03]);
        let encoded = WorldDictionary::from_raw_data(raw_data).encode();
        let declared_length = i32::from_be_bytes(encoded[..4].try_into().unwrap()) as usize;
        let sentinel = "DICTIONARY_PACKET_END".encode();

        assert_eq!(declared_length, encoded.len() - 4);
        assert_eq!(&encoded[4..7], &[0x01, 0x02, 0x03]);
        assert_eq!(&encoded[7..], sentinel.as_ref());
    }

    #[test]
    fn default_connection_details_is_compact_and_has_fixed_subblocks() {
        let details = ConnectionDetails::default().encode();
        assert!(details.len() < 1024);
        assert_eq!(SandboxOptions::default().encode().as_ref()[..4], *b"SAND");
        assert_eq!(ErosionConfig::default().encode().len(), 148);
    }

    #[test]
    fn workshop_section_is_only_written_in_steam_mode() {
        let details = ConnectionDetails {
            steam_mode: true,
            ..Default::default()
        };
        let encoded = details.encode();
        let without_workshop = ConnectionDetails::default().encode();

        assert_eq!(encoded.len(), without_workshop.len() + 2);
        assert_eq!(
            encoded[encoded.len() - 4..],
            without_workshop[without_workshop.len() - 4..]
        );
    }
}
