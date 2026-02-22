use crate::{
    cli::{
        self,
        schema::{CacheAction, Commands, ConfigAction, RepoAction},
    },
    config::BelleConfig,
    fetch,
};

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
