use layline_macros::{NetEncode, packet};

#[derive(Debug, NetEncode)]
#[packet(name = "AccessDenied", outbound)]
pub struct Rejected {
    pub reason: String,
}
