use crate::{
    cli::{MetaFetchArgs, MetaListArgs},
    config::BelleConfig,
};

mod cli;
mod config;
mod fetch;
mod registry;
mod resolver;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, MetaAction};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure configuration/state is initialised
    BelleConfig::init()?;

    // Parse command-line arguments and dispatch to the appropriate action
    let args = Cli::parse();

    match args.command {
        Commands::Meta(action) => match action {
            // List AFP repositories
            MetaAction::List(MetaListArgs { limit }) => {
                fetch::list_repositories(limit).await?;
            }
            // Fetch metadata for a given repository
            MetaAction::Fetch(MetaFetchArgs { name }) => {
                fetch::fetch_meta(name).await?;
            }
        },
    }

    Ok(())
}
