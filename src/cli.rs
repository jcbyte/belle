use crate::cli::{
    self,
    schema::{CacheAction, Commands, ConfigAction, RepoAction},
};

mod config;
mod fetch;
mod registry;
mod schema;
pub use schema::Cli;

pub async fn run(args: Cli) -> anyhow::Result<()> {
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
                cli::registry::clean_theories(target_version)?;
                if args.meta {
                    cli::registry::clean_metadata(target_version)?;
                }
            }
        },
        Commands::Inspect(args) => {
            if args.versions {
                cli::registry::list_versions(args.name)?;
            } else {
                cli::registry::print_package_meta(args.name, args.version)?;
            }
        }
        Commands::Config(action) => match action {
            ConfigAction::List => {
                cli::config::print_all_config();
            }
            ConfigAction::Get(args) => {
                cli::config::print_config(&args.key)?;
            }
            ConfigAction::Set(args) => {
                cli::config::set_config(&args.key, &args.value)?;
            }
        },
    }

    return Ok(());
}
