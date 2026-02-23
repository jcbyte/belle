use clap::{ArgGroup, Args, Parser, Subcommand};
use pubgrub::SemanticVersion;

#[derive(Parser)]
#[command(name = "belle")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage and explore AFP repositories
    #[command(subcommand)]
    Repo(RepoAction),

    /// Manage locally cached theories and metadata
    #[command(subcommand)]
    Cache(CacheAction),

    /// Display detailed information for a specific package/theory
    Inspect(InspectArgs),

    /// View or modify application configuration
    #[command(subcommand)]
    Config(ConfigAction),
}

#[derive(Subcommand)]
pub enum RepoAction {
    /// List known AFP repositories
    List(MetaListArgs),

    /// Synchronize metadata from a repository to the local system
    #[command(alias = "fetch")]
    Update(RepoUpdateArgs),
}

#[derive(Args)]
pub struct MetaListArgs {
    /// Optional maximum number of AFP repos to show
    #[arg(short, long, value_name = "LIMIT", default_value_t = 20)]
    pub limit: usize,
}

#[derive(Args)]
pub struct RepoUpdateArgs {
    /// Optional name of AFP repo (defaults to latest)
    #[arg(value_name = "REPO")]
    pub name: Option<String>,

    /// Ignore cache and re-fetch all theories
    #[arg(long)]
    pub no_cache: bool,
}

#[derive(Subcommand)]
pub enum CacheAction {
    /// Remove downloaded files to free up disk space
    Clean(CacheCleanArgs),
}

#[derive(Args)]
#[command(group(ArgGroup::new("selection").required(true).args(["version", "all"])))]
pub struct CacheCleanArgs {
    /// The specific version to remove from cache
    pub version: Option<SemanticVersion>,

    /// Remove all cached versions
    #[arg(long)]
    pub all: bool,

    /// Also remove package/theory metadata (requires a 'repo update' to restore)
    #[arg(short, long)]
    pub meta: bool,
}

#[derive(Args)]
pub struct InspectArgs {
    /// The name of the package/theory to inspect
    pub name: String,

    /// Inspect a specific version (defaults to latest)
    #[arg(conflicts_with = "versions")]
    pub version: Option<SemanticVersion>,

    /// List all available versions for this package instead
    #[arg(short, long)]
    pub versions: bool,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// List all configuration parameters and their current values
    List,

    /// View the value of a specific configuration parameter
    Get(ConfigGetArgs),

    /// Assign a new value to a configuration parameter
    Set(ConfigSetArgs),
}

#[derive(Args)]
pub struct ConfigGetArgs {
    /// The name of configuration parameter to view
    pub key: String,
}

#[derive(Args)]
pub struct ConfigSetArgs {
    /// The name of configuration parameter to update
    pub key: String,

    /// The new value for the configuration parameter
    pub value: String,
}

// todo environments
// belle env create [name]
// belle env list
// belle env remove [name]
// belle switch [name]

// todo package management
// belle add [package]
// belle remove [package]
// belle update
// belle list

// todo package file (save, load, lock?)
