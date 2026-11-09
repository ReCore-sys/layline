use layline_macros::{NetEncode, packet};

#[derive(Debug, NetEncode)]
#[packet(name = "LoadPlayerProfile", outbound)]
pub struct LoadPlayerProfileResponse {
    pub is_exist: bool,
    pub player_index: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use layline_codec::NetEncode;

    #[test]
    fn encodes_a_missing_profile_response() {
        let response = LoadPlayerProfileResponse {
            is_exist: false,
            player_index: 2,
        };

        assert_eq!(response.encode().as_ref(), &[0, 2]);
    }
}
