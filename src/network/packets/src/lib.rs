pub mod inbound;
pub mod outbound;
pub mod registry;

#[cfg(test)]
mod tests {
    use super::*;
    use inbound::{
        load_player_profile::LoadPlayerProfile,
        login::Login,
        login_queue_request::LoginQueueRequest,
        ping::InboundPing,
    };
    use layline_codec::{Packet, Reliability};
    use outbound::{
        load_player_profile::LoadPlayerProfileResponse,
        login_queue_request::LoginQueueResponse,
        ping::OutboundPing,
        rejected::Rejected,
        request_data::RequestData,
    };

    #[test]
    fn packet_transport_settings_match_protocol() {
        assert_eq!(Login::PACKET_ID, 0x0001);
        assert_eq!(Login::RELIABILITY, Reliability::ReliableOrdered);
        assert_eq!(Login::ORDERING_CHANNEL, 0);

        assert_eq!(LoadPlayerProfile::PACKET_ID, 0x0004);
        assert_eq!(
            LoadPlayerProfile::RELIABILITY,
            Reliability::ReliableOrdered
        );
        assert_eq!(LoadPlayerProfile::ORDERING_CHANNEL, 0);
        assert_eq!(LoadPlayerProfileResponse::PACKET_ID, 0x0004);
        assert_eq!(
            LoadPlayerProfileResponse::RELIABILITY,
            Reliability::ReliableOrdered
        );
        assert_eq!(LoadPlayerProfileResponse::ORDERING_CHANNEL, 0);

        assert_eq!(LoginQueueRequest::PACKET_ID, 0x0002);
        assert_eq!(
            LoginQueueRequest::RELIABILITY,
            Reliability::ReliableOrdered
        );
        assert_eq!(LoginQueueRequest::ORDERING_CHANNEL, 0);
        assert_eq!(LoginQueueResponse::PACKET_ID, 0x0002);
        assert_eq!(
            LoginQueueResponse::RELIABILITY,
            Reliability::ReliableOrdered
        );
        assert_eq!(LoginQueueResponse::ORDERING_CHANNEL, 0);

        assert_eq!(InboundPing::PACKET_ID, 0x0072);
        assert_eq!(InboundPing::RELIABILITY, Reliability::Unreliable);
        assert_eq!(InboundPing::ORDERING_CHANNEL, 0);

        assert_eq!(OutboundPing::PACKET_ID, 0x0072);
        assert_eq!(OutboundPing::RELIABILITY, Reliability::Unreliable);
        assert_eq!(OutboundPing::ORDERING_CHANNEL, 0);

        assert_eq!(Rejected::PACKET_ID, 0x0064);
        assert_eq!(Rejected::RELIABILITY, Reliability::Reliable);
        assert_eq!(Rejected::ORDERING_CHANNEL, 0);

        assert_eq!(RequestData::PACKET_ID, 0x0029);
        assert_eq!(RequestData::RELIABILITY, Reliability::Reliable);
        assert_eq!(RequestData::ORDERING_CHANNEL, 4);
    }
}
