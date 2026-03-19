use crate::{
    cli::{
        self,
        schema::{CacheAction, Commands, EnvAction, SourceAction, SourceAfpAction},
    },
    error::AppError,
};

mod environment;
pub mod error;
mod fetch;
mod isabelle;
mod package;
mod registry;
mod schema;
mod theming;
mod types;

use pubgrub::SemanticVersion;
pub use schema::Cli;
use types::IsabelleVersion;

pub async fn run(args: Cli) -> Result<(), AppError> {
    match args.command {
        Commands::Link(args) => isabelle::link(args.path)?,
        Commands::Unlink(args) => isabelle::unlink(args.version.into())?,
        Commands::Source(action) => match action {
            SourceAction::Afp(action) => match action {
                SourceAfpAction::List(args) => fetch::list_afp_repositories(args.limit).await?,
                SourceAfpAction::Update(args) => fetch::fetch_afp_meta(args.name.as_deref()).await?,
            },
            SourceAction::Remote(args) => fetch::source_remote_repo(&args.url, &args.branch).await?,
            SourceAction::Local(args) => fetch::source_local_package(&args.directory)?,
        },
        Commands::Cache(action) => match action {
            CacheAction::Clean(args) => {
                cli::registry::clean_theories()?;
                if args.meta {
                    cli::registry::clean_metadata()?;
                }
            }
            CacheAction::Purge => registry::purge_packages()?,
        },
        Commands::Inspect(args) => {
            if args.versions {
                cli::registry::list_versions(&args.name)?;
            } else {
                cli::registry::print_package_meta(args.name, args.version)?;
            }
        }
        Commands::Search(args) => registry::search_registry(args.query),
        Commands::Switch(args) | Commands::Env(EnvAction::Switch(args)) => environment::switch_env(args.name)?,
        Commands::Env(action) => match action {
            EnvAction::Create(args) => {
                environment::create_env(args.name, args.new, args.isabelle.map(SemanticVersion::from)).await?
            }
            EnvAction::List => environment::list_envs()?,
            EnvAction::Remove(args) => environment::remove_env(&args.name)?,
            EnvAction::Switch(_args) => unreachable!(),
            EnvAction::Freeze => environment::freeze_env()?,
            EnvAction::Sync => environment::sync_env().await?,
        },
        Commands::Migrate(args) => {
            environment::migrate_isabelle(args.version.map(SemanticVersion::from), args.unpin).await?
        }
        Commands::Add(args) => package::add_package(args.name, args.version).await?,
        Commands::Remove(args) => package::remove_package(&args.name).await?,
        Commands::List(args) => package::list_packages(args.all)?,
    }

    Ok(())
}
