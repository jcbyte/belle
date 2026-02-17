use anyhow::Context;
use tabled::{Table, settings::Style};

use crate::fetch::{client::BelleClient, metadata::RepoMetadata};

mod afp_repo;
pub mod client;
pub mod metadata;

/// List AFP repositories and print them in a simple table
pub async fn list_repositories(limit: usize) -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    // Get repositories
    let mut afp_repos = client.get_afp_repos(limit).await?;
    afp_repos.reverse();

    // Print in a pretty table
    let mut table = Table::new(&afp_repos);
    table.with(Style::rounded());

    println!("{}", table);
    println!("Found {} repositories.", afp_repos.len());

    return Ok(());
}

/// Fetch metadata for a specific repository (or the latest if not specified)
/// Register packages which do not yet exist locally
pub async fn fetch_meta(repo_name: Option<String>) -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    // Get the repo structure
    let repo = match repo_name {
        Some(name) => {
            // If a name is passed we need to get its id
            let repo = client.get_repo(&name).await?;
            // Warn if the repo does not exist
            repo.with_context(|| format!("Could not find repo with name '{}'", name))?
        }
        None => {
            // Get the most recent repo if none specified
            let latest_repo_collection = client.get_afp_repos(1).await?;
            let latest_repo = latest_repo_collection.first().map(|repo| repo.clone());
            latest_repo.context("Failed to fetch latest repo name")?
        }
    };

    print!("Fetching metadata for {} ({}).", repo.name, repo.get_version());

    // Get the metadata from the repo, and then create our metadata struct from this
    let repo_metadata = RepoMetadata::new(&repo, &client).await?;

    // todo can i list how many we currently have how many in repo
    // todo progress bar

    // Iterate though all theories in the repository metadata
    for theory in repo_metadata.all_theories() {
        // If this theory is already saved locally don't bother registering again
        if theory.package_exists() {
            continue;
        }

        // Create the package metadata and register it
        // Creating metadata will require network, so this could take some time
        println!("Generating (fetching) metadata for {}", theory);
        let package = repo_metadata.create_package_meta(&theory.name, &client).await?;
        package.register()?;
    }

    return Ok(());
}

/// todo
pub async fn get_package() {
    todo!()
}
