use crate::cli::{
    self,
    schema::{CacheAction, Commands, EnvAction, SourceAction, SourceAfpAction},
};

mod environment;
mod fetch;
mod package;
mod registry;
mod schema;

pub use schema::Cli;

pub async fn run(args: Cli) -> anyhow::Result<()> {
    match args.command {
        Commands::Source(action) => match action {
            SourceAction::Afp(action) => match action {
                SourceAfpAction::List(args) => fetch::list_afp_repositories(args.limit).await?,
                SourceAfpAction::Update(args) => fetch::fetch_afp_meta(args.name).await?,
            },
            SourceAction::Remote(args) => fetch::source_remote_repo(args.url, &args.branch).await?,
            SourceAction::Local(args) => fetch::source_local_package(args.directory)?,
        },
        Commands::Cache(action) => match action {
            CacheAction::Clean(args) => {
                cli::registry::clean_theories()?;
                if args.meta {
                    cli::registry::clean_metadata()?;
                }
            }
            CacheAction::Purge => todo!("3 purge"),
        },
        Commands::Inspect(args) => {
            if args.versions {
                cli::registry::list_versions(args.name)?;
            } else {
                cli::registry::print_package_meta(args.name, args.version)?;
            }
        }
        Commands::Search(args) => registry::search_registry(args.query),
        Commands::Switch(args) | Commands::Env(EnvAction::Switch(args)) => environment::switch_env(args.name)?,
        Commands::Env(action) => match action {
            EnvAction::Create(args) => environment::create_env(args.name, args.new, args.isabelle)?,
            EnvAction::List => environment::list_envs()?,
            EnvAction::Remove(args) => environment::remove_env(&args.name)?,
            EnvAction::Switch(_args) => unreachable!(),
            EnvAction::Freeze => environment::freeze_env()?,
            EnvAction::Sync => environment::sync_env()?,
        },
        Commands::Add(args) => package::add_package(args.name, args.version)?,
        Commands::Remove(args) => package::remove_package(&args.name)?,
        Commands::Migrate(args) => environment::migrate_isabelle(args.version, args.unpin)?,
        Commands::List(args) => environment::list_packages(args.all)?,
    }

    return Ok(());
}
