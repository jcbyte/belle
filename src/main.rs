use crate::{
    cli::{MetaListArgs, RepoUpdateArgs},
    config::BelleConfig,
};

mod cli;
mod config;
mod fetch;
mod registry;
mod resolver;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, RepoAction};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure configuration/state is initialised
    BelleConfig::init()?;

    // Parse command-line arguments and dispatch to the appropriate action
    let args = Cli::parse();

    match args.command {
        Commands::Repo(action) => match action {
            // List AFP repositories
            RepoAction::List(MetaListArgs { limit }) => {
                fetch::list_repositories(limit).await?;
            }
            // Fetch metadata for a given repository
            RepoAction::Update(RepoUpdateArgs { name, no_cache }) => {
                fetch::fetch_meta(name, !no_cache).await?;
            }
        },
    }

    Ok(())
}
