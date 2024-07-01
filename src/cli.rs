use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "TOML FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create snapshot
    Snapshot {
        #[arg(short, long)]
        dir: String,

        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Configuration options
    Config(ConfigArgs),

    /// Prepare folder to create a snapshot
    Init {
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Run terminal-user-interface
    Tui {},
}
#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigOptions>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigOptions {
    /// Show the current configuration
    Show,
}

// TODO: snapshots metadata
// PERF: add tags (names) to the snapshots
