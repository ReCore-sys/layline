use crate::game_state::GameState;
use layline_macros::handler;
use layline_network::Connection;
use layline_packets::inbound::login_queue_request::LoginQueueRequest;
use layline_packets::outbound::login_queue_request::LoginQueueResponse;
use tracing::warn;

#[handler]
pub fn login_queue_request(
    _: &mut GameState,
    connection: &Connection,
    _: &mut LoginQueueRequest,
) {
    if let Err(error) = connection.send(LoginQueueResponse::ConnectionImmediate) {
        warn!("Failed to send LoginQueueRequest immediate response: {error}");
    }
}
