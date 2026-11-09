use layline_macros::{NetDecode, packet};

#[derive(Debug, NetDecode)]
#[packet(name = "LoginQueueRequest", inbound)]
pub struct LoginQueueRequest {}
