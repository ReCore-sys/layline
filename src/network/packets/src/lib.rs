pub mod inbound;
pub mod outbound;
pub mod registry;

#[cfg(test)]
mod tests {
    use super::*;
    use inbound::{login::Login, ping::InboundPing};
    use layline_codec::{Packet, Reliability};
    use outbound::{ping::OutboundPing, rejected::Rejected, request_data::RequestData};

    #[test]
    fn packet_transport_settings_match_protocol() {
        assert_eq!(Login::PACKET_ID, 0x0001);
        assert_eq!(Login::RELIABILITY, Reliability::ReliableOrdered);
        assert_eq!(Login::ORDERING_CHANNEL, 0);

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
