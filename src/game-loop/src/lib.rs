mod handlers;
mod game_state;

use crate::handlers::PacketRegistry;
use crate::game_state::GameState;
use layline_network::start_net_server;

const TPS: u32 = 30; // Ticks per second
const MAX_PACKETS_PER_TICK: usize = 64;

pub fn start_game_loop() {
    std::thread::spawn(|| {
        start_net_server();
    });
    let registry = handlers::registry();
    let mut world = GameState::default();
    loop {
        let start_time = std::time::Instant::now();
        handle_connections(&registry, &mut world);
        let elapsed = start_time.elapsed();
        let sleep_duration = std::time::Duration::from_secs_f32(1.0 / TPS as f32) - elapsed;
        if sleep_duration > std::time::Duration::ZERO {
            std::thread::sleep(sleep_duration);
        }
    }
}

fn handle_connections(registry: &PacketRegistry, world: &mut GameState) {
    for connection in layline_network::CONNECTIONS.iter() {
        for _ in 0..MAX_PACKETS_PER_TICK {
            let Some(packet) = connection.receive() else {
                break;
            };
            registry.dispatch(world, &connection, packet.id, packet.bytes);
        }
    }
}
