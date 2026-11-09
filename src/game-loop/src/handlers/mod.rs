use crate::game_state::GameState;
use layline_network::Connection;
use layline_packets::registry::Registry;

pub mod login;
pub mod load_player_profile;
pub mod ping;

pub type PacketRegistry = Registry<GameState, Connection>;

include!(concat!(env!("OUT_DIR"), "/handlers.rs"));

pub fn registry() -> PacketRegistry {
    let mut registry = PacketRegistry::new();
    register_handlers(&mut registry);
    registry
}
