use layline_macros::{NetDecode, packet};

#[derive(Debug, NetDecode)]
#[packet(name = "Ping", inbound)]
pub struct InboundPing {
    pub server_ip: String,
}
