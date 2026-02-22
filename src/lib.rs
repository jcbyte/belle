use crate::{
    cli::ConfigAction,
    config::BelleConfig,
    registry::{clean_metadata, clean_theories, list_versions, print_meta},
};

pub mod cli;
pub mod config;
mod fetch;
mod registry;
mod resolver;

use anyhow::Result;
use cli::{CacheAction, Commands, RepoAction};

pub async fn run(args: cli::Cli) -> Result<()> {
    match args.command {
        Commands::Repo(action) => match action {
            RepoAction::List(args) => {
                fetch::list_repositories(args.limit).await?;
            }
            RepoAction::Update(args) => {
                fetch::fetch_meta(args.name, !args.no_cache).await?;
            }
        },
        Commands::Cache(action) => match action {
            CacheAction::Clean(args) => {
                // This should be handled by clap, but ensure it is correct here
                let target_version = if args.all { None } else { args.version };
                clean_theories(target_version)?;
                if args.meta {
                    clean_metadata(target_version)?;
                }
            }
        },
        Commands::Inspect(args) => {
            if args.versions {
                list_versions(args.name)?;
            } else {
                print_meta(args.name, args.version)?;
            }
        }
        Commands::Config(action) => match action {
            ConfigAction::List => {
                BelleConfig::read_config(|c| c.print_all());
            }
            ConfigAction::Get(args) => {
                BelleConfig::read_config(|c| c.print(&args.key))?;
            }
            ConfigAction::Set(args) => {
                BelleConfig::write_config(|c| c.set(&args.key, &args.value))?;
            }
        },
    }

    return Ok(());
}
