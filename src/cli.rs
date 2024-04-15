use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help=true)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "TOML FILE")]
    pub config: Option<PathBuf>,

    /// Show the current configuration
    #[arg(long)]
    pub current_config: bool,

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

    /// Select snapshot
    Init {

    }
}

// TODO: .setuprsignore.toml
// TODO: snapshots metadata
// PERF: add tags (names) to the snapshots
// TODO: add TUI with RataTUI.rs
