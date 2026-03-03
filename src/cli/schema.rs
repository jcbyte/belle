use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
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
    /// Add packages source (from AFP or externally)
    #[command(subcommand)]
    Source(SourceAction),

    /// Manage locally cached theories and metadata
    #[command(subcommand)]
    Cache(CacheAction),

    /// Display detailed information for a specific package/theory
    #[command(visible_alias = "show")]
    Inspect(InspectArgs),

    /// Manage isolated environments
    #[command(subcommand)]
    Env(EnvAction),

    /// Change the active environment
    Switch(SwitchArgs),

    /// Add package into current environment
    Add(AddArgs),

    /// Remove package from current environment
    Remove(RemoveArgs),

    /// Update all packages in the current environment
    Upgrade,

    /// List all packages in the current environment
    List(ListArgs),
}

#[derive(Subcommand)]
pub enum SourceAction {
    /// Add packages from the AFP
    #[command(subcommand)]
    Afp(SourceAfpAction),

    /// Add packages from a remote source
    Remote(SourceRemoteAction),

    /// Add packages from a local source
    Local(SourceLocalAction),
}

#[derive(Subcommand)]
pub enum SourceAfpAction {
    /// List known AFP repositories
    List(MetaListArgs),

    /// Synchronize metadata from a repository to the local system
    #[command(visible_alias = "fetch")]
    Update(RepoUpdateArgs),
}

#[derive(Args)]
pub struct RepoUpdateArgs {
    /// Optional name of AFP repo (defaults to latest)
    #[arg(value_name = "REPO")]
    pub name: Option<String>,
}

#[derive(Args)]
pub struct SourceRemoteAction {
    /// GitHub repository containing the package
    pub url: String,
}

#[derive(Args)]
pub struct SourceLocalAction {
    /// Directory containing the package
    pub directory: PathBuf,
}

#[derive(Args)]
pub struct MetaListArgs {
    /// Optional maximum number of AFP repos to show
    #[arg(short, long, value_name = "LIMIT", default_value_t = 20)]
    pub limit: usize,
}

#[derive(Subcommand)]
pub enum CacheAction {
    /// Remove downloaded packages which are not used within any environments
    Purge,

    /// Remove downloaded files to free up disk space
    Clean(CacheCleanArgs),
}

#[derive(Args)]
pub struct CacheCleanArgs {
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
pub enum EnvAction {
    /// Create a new environment
    Create(EnvCreateArgs),

    /// List all environments
    List,

    /// Remove an environment
    Remove(EnvRemoveArgs),

    /// Switch to environment
    Switch(SwitchArgs),

    /// Export current environment to a requirements file
    Freeze,

    /// Install packages from a requirements file to match state
    Sync,
}

#[derive(Args)]
pub struct EnvCreateArgs {
    /// The name of the environment (defaults to name inside belle file, or overwrites if both provided)
    pub name: Option<String>,

    /// Ignore belle file and create fresh environment
    #[arg(short, long)]
    pub new: bool,
}

#[derive(Args)]
pub struct EnvRemoveArgs {
    /// The name of environment to remove
    pub name: String,
}

#[derive(Args)]
pub struct SwitchArgs {
    /// The name of environment to switch to (defaults to name inside belle file)
    pub name: Option<String>,
}

#[derive(Args)]
pub struct AddArgs {
    /// The name of package to add
    pub name: String,

    /// Specific version to add (defaults to latest)
    pub version: Option<SemanticVersion>,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// The name of package to remove
    pub name: String,
}

#[derive(Args)]
pub struct ListArgs {
    /// List all packages for environment (includes transitive dependencies)
    #[arg(short, long)]
    pub all: bool,
}
