use clap::Parser;

#[derive(Parser)]
pub struct CLIArgs {
    #[arg(long, default_value = "save1", help = "What named save to use.")]
    /// What named save to use.
    pub save_name: String,
}
