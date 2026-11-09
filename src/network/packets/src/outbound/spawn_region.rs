use layline_codec::lua::lua_table::NetworkLuaTable;
use layline_macros::{NetEncode, packet};

#[derive(Debug, NetEncode)]
#[packet(name = "SpawnRegion", outbound)]
pub struct SpawnRegion {
    pub index: i32,
    pub table: NetworkLuaTable,
}