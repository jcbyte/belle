use std::time::Duration;

use anyhow::Context;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tabled::{Table, settings::Style};

use crate::{
    fetch::{client::BelleClient, metadata::RepoMetadata},
    registry::PackageIdentifier,
};

mod afp_repo;
pub mod client;
pub mod metadata;

/// List AFP repositories and print them in a simple table
pub async fn list_repositories(limit: usize) -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!("Fetching repository list"));

    // Get repositories
    let mut afp_repos = client.get_afp_repos(limit).await?;
    afp_repos.reverse();

    pb.finish_and_clear();

    // Print in a pretty table
    let mut table = Table::new(&afp_repos);
    table.with(Style::rounded());

    println!("{}", table);
    println!("Found {} AFP repositories.", style(afp_repos.len()).bold());

    return Ok(());
}

/// Fetch metadata for a specific repository (or the latest if not specified)
/// Register packages which do not yet exist locally
pub async fn fetch_meta(repo_name: Option<String>, use_cache: bool) -> anyhow::Result<()> {
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

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!(
        "Fetching theories list from {} ({})",
        style(&repo.name).cyan().bold(),
        style(repo.get_version()).yellow()
    ));

    // Get the metadata from the repo, and then create our metadata struct from this
    let repo_metadata = RepoMetadata::new(&repo, &client).await?;
    let repo_theories = repo_metadata.all_theories();

    pb.finish_with_message(format!(
        "Found {} theories from {} ({}).",
        style(repo_theories.len()).bold(),
        style(&repo.name).cyan().bold(),
        style(repo.get_version()).yellow()
    ));

    // No need to register packages that already exist
    let to_fetch: Vec<&PackageIdentifier> = if use_cache {
        repo_theories.iter().filter(|t| !t.package_exists()).collect()
    } else {
        // If no cache is set then we must collect all of them
        repo_theories.iter().collect()
    };

    if to_fetch.is_empty() {
        println!("No new theories, local registry is already up to date!");
    } else {
        println!("Found {} new theories to sync.", style(to_fetch.len()).bold());

        let pb = ProgressBar::new(to_fetch.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
                .progress_chars("#>-"),
        );

        // Iterate though all theories in the repository metadata
        for theory in to_fetch.iter() {
            // Create the package metadata and register it
            // Creating metadata will require network, so this could take some time
            pb.set_message(format!("Syncing {}", style(theory).cyan()));
            let package = repo_metadata.create_package_meta(&theory.name, &client).await?;
            package.register()?;

            pb.inc(1);
        }

        pb.finish_and_clear();
        println!(
            "Synced {} packages from {} ({}).",
            style(to_fetch.len()).bold(),
            style(&repo.name).cyan().bold(),
            style(repo.get_version()).yellow()
        );
    }

    return Ok(());
}

/// todo
pub async fn get_package() {
    todo!()
}
