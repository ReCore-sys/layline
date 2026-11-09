mod game_state;
mod handlers;

use tracing::{error, info};
use crate::game_state::GameState;
use crate::handlers::PacketRegistry;
use layline_network::start_net_server;
use layline_shared::paths::get_user_dir;
use layline_world::GameMap;

const TPS: u32 = 30; // Ticks per second
const MAX_PACKETS_PER_TICK: usize = 64;

pub fn start_game_loop(cli_args: layline_config::cli_args::CLIArgs) {
    std::thread::spawn(|| {
        start_net_server();
    });
    let registry = handlers::registry();


    let config_path = get_user_dir().map(|path| {
        path.join("Server").join(format!("{}.ini", cli_args.save_name))
    });

    let config = match config_path {
        Ok(ref path) => match layline_config::Config::load_from_file(path) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to load config from {}: {}", path.display(), e);
                return;
            }
        },
        Err(e) => {
            error!("Failed to get user directory: {}", e);
            return;
        }
    };

    info!("Loaded config successfully. Starting game loop with save: {}", cli_args.save_name);

    let map = match GameMap::new() {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to create game map: {}", e);
            return;
        }
    };

    let mut world = GameState {
        world: hecs::World::new(),
        map,
        cli_args,
        config
    };
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
