use crate::cli::{
    self,
    schema::{CacheAction, Commands, EnvAction, RepoAction},
};

mod environment;
mod fetch;
mod package;
mod registry;
mod schema;

pub use schema::Cli;

pub async fn run(args: Cli) -> anyhow::Result<()> {
    match args.command {
        Commands::Repo(action) => match action {
            RepoAction::List(args) => fetch::list_repositories(args.limit).await?,
            RepoAction::Update(args) => fetch::fetch_meta(args.name).await?,
        },
        Commands::Cache(action) => match action {
            CacheAction::Clean(args) => {
                cli::registry::clean_theories()?;
                if args.meta {
                    cli::registry::clean_metadata()?;
                }
            }
            CacheAction::Purge => {
                todo!("purge");
            }
        },
        Commands::Inspect(args) => {
            if args.versions {
                cli::registry::list_versions(args.name)?;
            } else {
                cli::registry::print_package_meta(args.name, args.version)?;
            }
        }
        Commands::Switch(args) | Commands::Env(EnvAction::Switch(args)) => environment::switch_env(args.name)?,
        Commands::Env(action) => match action {
            EnvAction::Create(args) => environment::create_env(args.name)?,
            EnvAction::List => environment::list_envs()?,
            EnvAction::Remove(args) => environment::remove_env(&args.name)?,
            EnvAction::Switch(_args) => unreachable!(),
            EnvAction::Freeze => environment::freeze_env()?,
            EnvAction::Sync => environment::sync_env()?,
        },
        Commands::Add(args) => package::add_package(args.name, args.version)?,
        Commands::Remove(args) => package::remove_package(&args.name)?,
        Commands::Update => todo!("update packages"),
        Commands::List(args) => package::list_packages(args.all)?,
    }

    return Ok(());
}
