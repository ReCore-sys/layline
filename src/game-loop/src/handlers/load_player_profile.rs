use crate::game_state::GameState;
use layline_macros::handler;
use layline_network::Connection;
use layline_packets::inbound::load_player_profile::LoadPlayerProfile;
use layline_packets::outbound::load_player_profile::LoadPlayerProfileResponse;
use tracing::warn;

#[handler]
pub fn load_player_profile(
    _state: &mut GameState,
    connection: &Connection,
    packet: &mut LoadPlayerProfile,
) {
    let response = LoadPlayerProfileResponse {
        is_exist: false,
        player_index: packet.player_index,
    };

    if let Err(error) = connection.send(response) {
        warn!(
            "Failed to send empty LoadPlayerProfile response for player {}: {error}",
            packet.player_index
        );
    }
}
