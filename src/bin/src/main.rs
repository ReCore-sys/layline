use layline_game_loop::start_game_loop;
use layline_logging::init_logging;
use tracing::info;

fn main() {
    init_logging();
    info!("Starting Layline...");
    start_game_loop();
}
