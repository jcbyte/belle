use std::path::PathBuf;

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
    /// AFP repository operations
    #[command(subcommand)]
    Repo(RepoAction),

    /// Internal cache operations
    #[command(subcommand)]
    Cache(CacheAction),

    /// Show package/theory information
    Show(ShowArgs),

    /// View/Modify config variables
    #[command(subcommand)]
    Config(ConfigAction),
}

#[derive(Subcommand)]
pub enum RepoAction {
    /// List all available AFP repositories
    List(MetaListArgs),
    /// Fetch and update metadata from an AFP repository
    #[command(alias = "fetch")]
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
    /// Ignore cache and re-fetch all theories
    #[arg(long)]
    pub no_cache: bool,
}

#[derive(Subcommand)]
pub enum CacheAction {
    /// Clean internal cache
    Clean(CacheCleanArgs),
}

#[derive(Args)]
#[command(group(ArgGroup::new("selection").required(true).args(["version", "all"])))]
pub struct CacheCleanArgs {
    /// Clear cached files for a specific version
    pub version: Option<SemanticVersion>,

    /// Clear cached files for all versions
    #[arg(long)]
    pub all: bool,

    /// Force removal of internal metadata (not done by default; requires re-fetch)
    #[arg(short, long)]
    pub meta: bool,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Name of package/theory to view
    pub name: String,

    /// Version of package/theory to view (defaults to latest)
    #[arg(conflicts_with = "versions")]
    pub version: Option<SemanticVersion>,

    #[arg(short, long)]
    pub versions: bool,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// List all config variables and their values
    List,

    /// View config setting value
    Get(ConfigGetArgs),

    /// Update config setting value
    Set(ConfigSetArgs),
}

#[derive(Args)]
pub struct ConfigGetArgs {
    /// Name of config setting to view
    pub key: String,
}

#[derive(Args)]
pub struct ConfigSetArgs {
    /// Name of config setting to change
    pub key: String,

    /// New value of the setting
    pub value: String,
}
