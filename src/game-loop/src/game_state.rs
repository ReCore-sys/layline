use hecs::World;
use layline_config::cli_args::CLIArgs;
use layline_config::Config;
use layline_world::GameMap;

pub struct GameState {
    pub world: World,
    pub map: GameMap,
    pub cli_args: CLIArgs,
    pub config: Config
}
