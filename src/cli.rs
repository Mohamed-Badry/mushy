use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "A terminal pet that walks along the inner borders"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to a custom config.toml file
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// Path to the gif file
    #[arg(short, long, global = true)]
    pub gif: Option<String>,

    /// Target size of the gif in pixels
    #[arg(short, long, global = true)]
    pub size: Option<u32>,

    /// Speed multiplier for the animation (e.g. 2.0 for double speed)
    #[arg(short = 'x', long, global = true)]
    pub speed: Option<f32>,

    /// Rotate the gif clockwise instead of counter-clockwise
    #[arg(long, global = true)]
    pub cw: bool,

    /// Internal flag to run as daemon
    #[arg(long, hide = true)]
    pub daemon: bool,

    /// Internal flag to pass the specific TTY run file
    #[arg(long, hide = true)]
    pub run_file: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Stop the currently running mushy daemon in this terminal
    Stop {
        /// Stop all running instances across all terminals
        #[arg(short, long)]
        all: bool,
    },
}
