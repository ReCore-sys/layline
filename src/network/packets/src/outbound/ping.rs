use layline_macros::{NetEncode, packet};

#[derive(Debug, NetEncode)]
#[packet(name = "Ping", outbound)]
pub struct OutboundPing {
    pub server_ip: String,
    pub connections: i32,
    pub max_connections: i32,
}
