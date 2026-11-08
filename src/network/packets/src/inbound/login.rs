use layline_macros::{NetDecode, packet};

#[derive(Debug, NetDecode)]
#[packet(name = "Login", inbound)]
pub struct Login {
    pub username: String,
    pub password: String,
    pub client_version: String,
    pub auth_type: i32,
}
