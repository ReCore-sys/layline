use crate::game_state::GameState;
use hecs::With;
use layline_components::markers::PlayerMarker;
use layline_macros::handler;
use layline_network::Connection;
use layline_packets::inbound::ping::InboundPing;

#[handler]
pub fn ping(state: &mut GameState, connection: &Connection, packet: &mut InboundPing) {
    let outbound_ping = layline_packets::outbound::ping::OutboundPing {
        server_ip: std::mem::take(&mut packet.server_ip),
        max_connections: 99,
        connections: state
            .world
            .query::<With<&PlayerMarker, &bool>>()
            .iter()
            .len() as i32,
    };
    connection.send(outbound_ping).unwrap();
}
