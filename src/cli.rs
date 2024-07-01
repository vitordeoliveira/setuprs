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
    /// Snapshot commands
    Snapshot(SnapshotArgs),

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

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    pub command: SnapshotOptions,
}

#[derive(Debug, Subcommand)]
pub enum SnapshotOptions {
    /// Create new snapshot
    #[command(arg_required_else_help = true)]
    Create {
        /// Define FROM here setuprs should create the snapshot
        project_path: String,

        /// If set will create a name for the snapshot, if not will create an unique ID
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Clone snapshot
    #[command(arg_required_else_help = true)]
    Clone {
        /// Select snapshot
        snapshot: String,

        /// Define TO here setuprs should clone the snapshot
        #[arg(short, long)]
        destination_path: Option<String>,
    },
}

// TODO: snapshots metadata
// PERF: add tags (names) to the snapshots
