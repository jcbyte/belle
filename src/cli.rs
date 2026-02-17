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
    /// AFP repository operations
    #[command(subcommand)]
    Repo(RepoAction),
}

#[derive(Subcommand)]
pub enum RepoAction {
    /// List all available AFP repositories
    List(MetaListArgs),
    /// Fetch and update metadata from an AFP repository
    Update(RepoUpdateArgs),
}

#[derive(Args)]
pub struct MetaListArgs {
    /// Optional maximum number of AFP repos to fetch
    #[arg(short, long, value_name = "LIMIT", default_value_t = 20)]
    pub limit: usize,
}

#[derive(Args)]
pub struct RepoUpdateArgs {
    /// Optional name of AFP repo (defaults to latest)
    #[arg(value_name = "REPO")]
    pub name: Option<String>,
    /// Ignore cache and refetch all theories
    #[arg(long)]
    pub no_cache: bool,
}

// todo belle cache clean [version]
// todo belle cache clean --all
// todo belle show [name] <version>
