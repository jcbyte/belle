use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "belle")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Metadata operations
    #[command(subcommand)]
    Meta(MetaAction),
}

#[derive(Subcommand)]
pub enum MetaAction {
    /// List all available AFP repositories
    List(MetaListArgs),
    /// Fetch metadata from a repository
    Fetch(MetaFetchArgs),
}

#[derive(Args)]
pub struct MetaListArgs {
    /// Optional maximum number of AFP repos to fetch
    #[arg(short, long, value_name = "LIMIT", default_value_t = 20)]
    pub limit: usize,
}

#[derive(Args)]
pub struct MetaFetchArgs {
    /// Optional name of AFP repo (defaults to latest)
    #[arg(value_name = "REPO")]
    pub name: Option<String>,
}

// todo belle meta clean [version]
// todo belle meta clean --all
