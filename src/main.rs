use crate::{
    config::BelleConfig,
    registry::{clean_metadata, clean_theories, list_versions},
};

mod cli;
mod config;
mod fetch;
mod registry;
mod resolver;

use anyhow::Result;
use clap::Parser;
use cli::{CacheAction, Cli, Commands, RepoAction};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure configuration/state is initialised
    BelleConfig::init()?;

    // Parse command-line arguments and dispatch to the appropriate action
    let args = Cli::parse();

    match args.command {
        Commands::Repo(action) => match action {
            // List AFP repositories
            RepoAction::List(args) => {
                fetch::list_repositories(args.limit).await?;
            }
            // Fetch metadata for a given repository
            RepoAction::Update(args) => {
                fetch::fetch_meta(args.name, !args.no_cache).await?;
            }
        },
        Commands::Cache(action) => match action {
            CacheAction::Clean(args) => {
                // This should be handled by clap, but just ensure here
                let target_version = if args.all { None } else { args.version };
                clean_theories(target_version)?;
                if args.meta {
                    clean_metadata(target_version)?;
                }
            }
        },
        Commands::Show(args) => {
            if args.versions {
                list_versions(args.name)?;
            } else {
                // todo get latest version if not supplied
                todo!("List meta for {}@{}", args.name, args.version.expect("msg").to_string());
            }
        }
    }

    Ok(())
}
