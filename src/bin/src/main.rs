use clap::Parser;
use layline_game_loop::start_game_loop;
use layline_logging::init_logging;
use tracing::info;

fn main() {
    init_logging();
    info!("Starting Layline...");
    let args = layline_config::cli_args::CLIArgs::parse();
    start_game_loop(args);
}
