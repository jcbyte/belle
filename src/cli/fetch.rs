use std::time::Duration;

use anyhow::Context;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    fetch::{BelleClient, RepoMetadata},
    registry::PackageIdentifier,
};

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

    // Print list of AFPs
    println!("AFP Repositories Listing:");
    for afp_repo in &afp_repos {
        println!(
            " {:<11} {}{}{}",
            style(&afp_repo.name).bold(),
            style("[").dim(),
            style(afp_repo.get_version().to_string()).green(),
            style("]").dim(),
        )
    }
    println!("Found {} AFP repositories.", style(afp_repos.len()).bold());

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

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!(
        "Fetching theories list from {} {}{}{}",
        style(&repo.name).cyan().bold(),
        style("[").dim(),
        style(repo.get_version()).green(),
        style("]").dim()
    ));

    // Get the metadata from the repo, and then create our metadata struct from this
    let repo_metadata = RepoMetadata::get(&repo, &client).await?;
    let repo_theories = repo_metadata.all_theories();

    pb.finish_with_message(format!(
        "Found {} theories from {} {}{}{}.",
        style(repo_theories.len()).bold(),
        style(&repo.name).cyan().bold(),
        style("[").dim(),
        style(repo.get_version()).green(),
        style("]").dim(),
    ));

    let pb = ProgressBar::new(repo_theories.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
            .progress_chars("#>-"),
    );

    for theory in repo_metadata.all_theories() {
        pb.set_message(format!("Syncing {}", style(&theory).cyan()));

        if theory.package_exists() {
            // If the package already exists, we must ensure that we have this isabelle version listed
            let mut theory_meta = theory
                .get_package_meta()?
                .expect("Package exists, but its metadata could not be found");
            if theory_meta.isabelles.insert(repo.name.clone()) {
                // Only re-register if this modified to avoid unnecessary IO
                theory_meta.register()?;
            }
        } else {
            // Create the package metadata and register it
            // Creating metadata will require network, so this could take some time
            let package = repo_metadata.create_package_meta(&theory.name, &client).await?;
            package.register()?;
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    println!(
        "Synced {} packages from {} {}{}{}.",
        style(repo_theories.len()).bold(),
        style(&repo.name).cyan().bold(),
        style("[").dim(),
        style(repo.get_version()).yellow(),
        style("]").dim(),
    );

    return Ok(());
}
