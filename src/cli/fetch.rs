use std::{path::Path, time::Duration};

use console::style;
use indicatif::ProgressBar;
use url::Url;

use crate::{
    cli::core::{CliLine, DisplayVersion, ProgressBarTheme, pluralise},
    error::{AppError, CustomErrorContext},
    fetch::{AfpRepo, BelleClient, RepoMetadata, ReturnedPackages, get_local_package_meta},
    registry::{Package, PackageIdentifier, RegistrablePackage},
};

/// List AFP repositories and print them in a simple table
pub async fn list_afp_repositories(limit: usize) -> Result<(), AppError> {
    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message("Fetching repository list".to_string());

    // Get repositories
    let client = BelleClient::get()?;
    let mut afp_repos = client.get_afp_repos(limit).await?;
    afp_repos.reverse();

    pb.finish_and_clear();

    // Print list of AFPs
    if let Some((latest_repo, other_repos)) = afp_repos.split_last() {
        let render = |repo: &AfpRepo| format!("{:<11} {}", &repo.name, DisplayVersion::Implicit(repo.get_version()));

        for afp_repo in other_repos {
            CliLine::new().line(render(afp_repo)).print();
        }
        CliLine::new().prefix("Latest").line(render(latest_repo)).as_focus().print();
    }

    CliLine::new()
        .prefix("Found")
        .line(format!(
            "{} AFP {}",
            style(afp_repos.len()).bold(),
            pluralise(afp_repos.len(), "repository", "repositories")
        ))
        .as_success()
        .print();

    Ok(())
}

/// Fetch metadata for a specific repository (or the latest if not specified)
/// Register packages which do not yet exist locally
pub async fn fetch_afp_meta(repo_name: Option<&str>) -> Result<(), AppError> {
    // Get the repo structure
    let client = BelleClient::get()?;
    let repo = match repo_name {
        Some(name) => {
            // If a name is passed we need to get its id
            client
                .get_afp_repo(name)
                .await?
                // Warn if the repo does not exist
                .report_custom(format!("Could not find AFP with name '{}'", name))?
        }
        None => {
            // Get the most recent repo if none specified
            let latest_repo_collection = client.get_afp_repos(1).await?;
            latest_repo_collection
                .into_iter()
                .next()
                .report_custom("Could not find the latest repo")?
        }
    };

    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!(
        "Fetching package manifests from {} {}",
        style(format!("AFP {}", &repo.get_formatted_name())).cyan().bright(),
        DisplayVersion::Implicit(repo.get_version())
    ));

    // Get the metadata from the repo, and then create our metadata struct from this
    let repo_metadata = RepoMetadata::get(&repo, client).await?;
    let repo_packages = repo_metadata.all_packages();

    pb.finish_and_clear();
    CliLine::new()
        .prefix("Found")
        .line(format!(
            "{} {} from {} {}",
            style(repo_packages.len()).bold(),
            pluralise(repo_packages.len(), "package", "packages"),
            style(format!("AFP {}", &repo.get_formatted_name())).cyan().bright(),
            DisplayVersion::Implicit(repo.get_version())
        ))
        .as_success()
        .print();

    let pb = ProgressBar::new(repo_packages.len() as u64).with_belle_bar_style();

    let mut unresolved_packages: Vec<Package> = Vec::new();

    let mut failed = 0;
    for package in repo_metadata.all_packages() {
        pb.set_message(format!("Syncing {}", package.styled()));

        if package.package_exists() {
            // If the package already exists, ensure that we have this isabelle version listed
            let mut package_meta = package
                .get_resolved_package_manifest()?
                .expect("Package exists, but its manifest could not be found");
            if package_meta.isabelles.insert(*repo.get_version()) {
                // Only re-register if this modified to avoid unnecessary IO
                package_meta.register()?;
            }
        } else {
            // Create the package metadata and register it
            // Creating metadata will require network, so this could take some time
            let package_meta_res = repo_metadata.create_package_meta(&package.name, client).await;
            match package_meta_res {
                Ok((
                    ReturnedPackages {
                        package: package_meta,
                        aliases,
                    },
                    fully_resolved,
                )) => {
                    if fully_resolved {
                        package_meta.register()?;
                    } else {
                        // Add the package to be resolved later
                        pb.println(format!(
                            "{}",
                            style(format!("Deferred resolving {} due to unseen dependencies", package)).dim()
                        ));

                        // Increase the progress bar count, as these must be handled afterwards
                        pb.inc_length(1);
                        unresolved_packages.push(package_meta);
                    }

                    for alias in aliases {
                        alias.register()?;
                    }
                }
                // If this produces an error then don't crash the entire fetch process
                Err(e) => {
                    // todo error handling better
                    pb.println(format!("{} {}", style("Error:").bold().red(), style(e).bright().red()));
                    failed += 1
                }
            }
        }

        pb.inc(1);
    }

    for mut unresolved_package in unresolved_packages {
        pb.set_message(format!(
            "Resolving {}",
            PackageIdentifier::from(&unresolved_package).styled()
        ));

        match repo_metadata.resolve_package_meta(&mut unresolved_package) {
            Ok(_) => {
                unresolved_package.register()?;
            }
            Err(e) => {
                // todo error handling better
                pb.println(format!("{} {}", style("Error:").bold().red(), style(e).bright().red()));
                failed += 1
            }
        };

        pb.inc(1);
    }

    pb.finish_and_clear();

    let failed_str = if failed > 0 {
        format!(", {} failed", style(failed).bold())
    } else {
        "".to_string()
    };
    CliLine::new()
        .prefix("Synced")
        .line(format!(
            "{} {} from {} {} {}",
            style(repo_packages.len() - failed).bold(),
            pluralise(repo_packages.len() - failed, "package", "packages"),
            style(format!("AFP {}", &repo.get_formatted_name())).cyan().bright(),
            DisplayVersion::Implicit(repo.get_version()),
            failed_str
        ))
        .as_success()
        .print();

    Ok(())
}

pub async fn source_remote_repo(url: &Url, branch: &str) -> Result<(), AppError> {
    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message("Fetching package manifest".to_string());

    let client = BelleClient::get()?;
    let ReturnedPackages { package, aliases } = client.get_github_package_meta(url, branch).await?;

    // Create this now as `package` gets borrowed after
    let package_id = PackageIdentifier::from(&package);

    package.register()?;
    for alias in aliases {
        alias.register()?;
    }

    pb.finish_and_clear();

    CliLine::new()
        .prefix("Sourced")
        .line(format!(
            "remote package {} {}{}{}",
            package_id.styled(),
            style("(").dim(),
            style(url).dim(),
            style(")").dim(),
        ))
        .as_success()
        .print();

    Ok(())
}

pub fn source_local_package(path: &Path) -> Result<(), AppError> {
    let ReturnedPackages { package, aliases } = get_local_package_meta(path)?;

    // Create this now as `package` gets borrowed after
    let package_id = PackageIdentifier::from(&package);

    package.register()?;
    for alias in aliases {
        alias.register()?;
    }

    CliLine::new()
        .prefix("Sourced")
        .line(format!(
            "local package {} {}{}{}",
            package_id.styled(),
            style("(").dim(),
            style(path.display()).dim(),
            style(")").dim(),
        ))
        .as_success()
        .print();

    Ok(())
}
