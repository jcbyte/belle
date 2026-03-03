use std::time::Duration;

use anyhow::Context;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use url::Url;

use crate::{
    cli::environment,
    fetch::{self, BelleClient, RepoMetadata},
    registry::{Package, RegistrablePackage},
};

/// List AFP repositories and print them in a simple table
pub async fn list_afp_repositories(limit: usize) -> anyhow::Result<()> {
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
pub async fn fetch_afp_meta(repo_name: Option<String>) -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    // Get the repo structure
    let repo = match repo_name {
        Some(name) => {
            // If a name is passed we need to get its id
            let repo = client.get_afp_repo(&name).await?;
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

    let mut unresolved_packages: Vec<Package> = Vec::new();

    let mut failed = 0;
    for theory in repo_metadata.all_theories() {
        pb.set_message(format!("Syncing {}", style(&theory).cyan()));

        if theory.package_exists() {
            // If the package already exists, we must ensure that we have this isabelle version listed
            let mut theory_meta = theory
                .get_resolved_package_manifest()?
                .expect("Package exists, but its manifest could not be found");
            if theory_meta.isabelles.insert(repo.get_version().clone()) {
                // Only re-register if this modified to avoid unnecessary IO
                theory_meta.register()?;
            }
        } else {
            // Create the package metadata and register it
            // Creating metadata will require network, so this could take some time
            let package_meta = repo_metadata.create_package_meta(&theory.name, &client).await;
            match package_meta {
                Ok((package, fully_resolved, aliases)) => {
                    if fully_resolved {
                        package.register()?;
                    } else {
                        // Add the package to be resolved later
                        pb.println(format!(
                            "{}",
                            style(format!(
                                "Deferred resolving {} due to unseen dependencies",
                                &package.name
                            ))
                            .dim()
                        ));
                        pb.inc_length(1);
                        unresolved_packages.push(package);
                    }

                    for alias in aliases {
                        alias.register()?;
                    }
                }
                // If this produces an error then don't crash the entire fetch process
                Err(e) => {
                    pb.println(format!("{}", style(e).red()));
                    failed += 1
                }
            }
        }

        pb.inc(1);
    }

    for mut unresolved_package in unresolved_packages {
        pb.set_message(format!(
            "Resolving dependencies for {}",
            style(&unresolved_package.name).cyan()
        ));

        match repo_metadata.resolve_package_meta(&mut unresolved_package) {
            Ok(_) => {}
            Err(e) => {
                pb.println(format!("{}", style(e).red()));
                failed += 1
            }
        };

        unresolved_package.register()?;
    }

    pb.finish_and_clear();
    println!(
        "Synced {} packages from {} {}{}{}. {}",
        style(repo_theories.len() - failed).bold(),
        style(&repo.name).cyan().bold(),
        style("[").dim(),
        style(repo.get_version()).yellow(),
        style("]").dim(),
        style(if failed > 0 {
            format!("({} failed)", failed)
        } else {
            String::new()
        })
        .red()
    );

    return Ok(());
}

pub async fn source_remote_repo(url: Url) -> anyhow::Result<()> {
    let client = BelleClient::new()?;
    let (package, aliases) = client.get_github_package_meta(url).await?;

    package.register()?;
    for alias in aliases {
        alias.register()?;
    }

    return Ok(());
}
