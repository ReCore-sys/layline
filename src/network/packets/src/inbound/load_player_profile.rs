use layline_macros::{NetDecode, packet};

#[derive(Debug, NetDecode)]
#[packet(name = "LoadPlayerProfile", inbound)]
pub struct LoadPlayerProfile {
    pub player_index: u8,
}
