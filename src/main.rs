use crate::{
    cli::{MetaFetchArgs, MetaListArgs},
    config::BelleConfig,
};

mod config;
mod fetch;
mod registry;

mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, MetaAction};

#[tokio::main]
async fn main() -> Result<()> {
    BelleConfig::init()?;

    let args = Cli::parse();

    match args.command {
        Commands::Meta(action) => match action {
            MetaAction::List(MetaListArgs { limit }) => {
                fetch::list_repositories(limit).await?;
            }
            MetaAction::Fetch(MetaFetchArgs { name }) => {
                fetch::fetch_meta(name).await?;
            }
        },
    }

    Ok(())
}
