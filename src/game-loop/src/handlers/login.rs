use crate::game_state::GameState;
use layline_macros::handler;
use layline_network::Connection;
use layline_packets::inbound::login::Login;
use layline_packets::outbound::rejected::Rejected;
use tracing::info;

#[handler]
pub fn login(_world: &mut GameState, connection: &Connection, packet: &mut Login) {
    info!("Login packet received from {}", packet.username);
    let rejected = Rejected {
        reason: "ServerFull".to_string(),
    };
    connection.send(rejected).unwrap();
}
