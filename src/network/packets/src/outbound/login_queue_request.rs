use layline_macros::{NetDecode, NetEncode, packet};

#[derive(Debug, NetEncode, NetDecode)]
#[packet(name = "LoginQueueRequest", outbound)]
pub enum LoginQueueResponse {
    None,
    ConnectionImmediate,
    PlaceInQueue {
        place_in_queue: u8,
        max_players: i16,
        count_players: i16,
        count_zombies: i32,
        zombies_killed_by_fire_today: i32,
        zombies_killed_today: i32,
        zombified_players_today: i32,
        players_killed_by_fire_today: i32,
        players_killed_by_zombie_today: i32,
        players_killed_by_player_today: i32,
        burned_corpses_today: i32,
        server_time_minutes: u8,
        server_time_hour: u8,
        server_time_day: u8,
        server_time_month: u8,
        server_time_year: i16,
        server_climate_base_temperature: f32,
        server_climate_humidity: f32,
        server_climate_rain_intensity: f32,
        server_climate_windspeed_kph: f32,
        server_climate_wind_angle_degrees: f32,
        server_climate_fog_intensity: f32,
        server_climate_season_id: u8,
        server_climate_moon: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use layline_codec::NetEncode;

    #[test]
    fn encodes_connection_immediate_message_type() {
        assert_eq!(
            LoginQueueResponse::ConnectionImmediate.encode().as_ref(),
            &[1]
        );
    }
}
