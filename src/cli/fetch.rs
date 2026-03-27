use std::{borrow::Cow, path::Path};

use console::style;
use hinted::{Hinted, HintedResultExt};
use indicatif::ProgressBar;
use url::Url;

use crate::{
    cli::core::{CliLine, DisplayVersion, ProgressBarTheme, pluralise},
    error::{AppError, CustomErrorContext},
    fetch::{AfpRepo, BelleClient, RepoMetadata, ReturnedPackages, get_local_package_meta},
    registry::{AliasPackage, Package, PackageIdentifier, RegistrablePackage},
};

/// List AFP repositories and print them in a simple table
pub async fn list_afp_repositories(limit: usize) -> Result<(), AppError> {
    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.set_belle_prefix("Fetching".to_string());
    pb.set_message("repository list");

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
        CliLine::new().prefix("Latest").line(render(latest_repo)).with_focus().print();
    }

    CliLine::new()
        .prefix("Found")
        .line(format!(
            "{} AFP {}",
            style(afp_repos.len()).bold(),
            pluralise(afp_repos.len(), "repository", "repositories")
        ))
        .with_success()
        .print();

    Ok(())
}

/// Fetch metadata for a specific repository (or the latest if not specified)
/// Register packages which do not yet exist locally
pub async fn source_afp_meta(repo_name: Option<&str>) -> Result<(), Hinted<AppError>> {
    // Get the repo structure
    let client = BelleClient::get().into_hinted()?;
    let repo = match repo_name {
        Some(name) => {
            // If a name is passed we need to get its id
            client
                .get_afp_repo(name)
                .await
                .into_hinted()?
                // Warn if the repo does not exist
                .report_custom(format!("could not find afp repository with name '{}'", name))
                .hint("check name, or use `belle source afp list` to see available afp repository names")?
        }
        None => {
            // Get the most recent repo if none specified
            let latest_repo_collection = client.get_afp_repos(1).await.into_hinted()?;
            latest_repo_collection
                .into_iter()
                .next()
                .report_custom("could not find the latest afp repository")
                .into_hinted()?
        }
    };

    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.set_belle_prefix("Fetching");
    pb.set_message(format!(
        "package manifests from {} {}",
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
        .with_success()
        .print();

    let pb = ProgressBar::new(repo_packages.len() as u64).with_belle_bar_style();
    pb.set_belle_prefix("Syncing");

    let mut unresolved_packages: Vec<(Package, Vec<AliasPackage>)> = Vec::new();

    let mut failed = 0;
    for package in repo_metadata.all_packages() {
        pb.set_message(package.styled());

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
                    missing_dependencies,
                )) => {
                    if missing_dependencies.is_empty() {
                        package_meta.register()?;

                        for alias in aliases {
                            alias.register()?;
                        }
                    } else {
                        // Add the package to be resolved later
                        // And make note of aliases to register, as these should only be registered once the root package has been
                        pb.println(
                            CliLine::new()
                                .prefix("Deferring")
                                .line(
                                    style(format!(
                                        "{} until dependencies ({}) are resolved",
                                        package,
                                        missing_dependencies.join(", ")
                                    ))
                                    .dim()
                                    .to_string(),
                                )
                                // A custom prefix has been applied, so this just affects aesthetics
                                .with_skipped()
                                .get(),
                        );

                        // Increase the progress bar count, as these must be handled afterwards
                        pb.inc_length(1);
                        unresolved_packages.push((package_meta, aliases));
                    }
                }
                // If this produces an error then don't crash the entire fetch process
                Err(e) => {
                    pb.println(CliLine::new().line(e.to_string()).with_error().get());
                    failed += 1
                }
            }
        }

        pb.inc(1);
    }

    pb.set_belle_prefix("Resolving");

    for (mut unresolved_package, package_aliases) in unresolved_packages {
        pb.set_message(PackageIdentifier::from(&unresolved_package).styled());

        match repo_metadata.resolve_package_meta(&mut unresolved_package) {
            Ok(_) => {
                unresolved_package.register()?;

                // Register the aliases now once thr ROOT has been registered
                for alias in package_aliases {
                    alias.register()?;
                }
            }
            Err(e) => {
                pb.println(CliLine::new().line(e.to_string()).with_error().get());
                failed += 1
            }
        };

        pb.inc(1);
    }

    pb.finish_and_clear();
    CliLine::new()
        .prefix("Sourced")
        .line(format!(
            "{} {} from {} {} {}",
            style(repo_packages.len() - failed).bold(),
            pluralise(repo_packages.len() - failed, "package", "packages"),
            style(format!("AFP {}", &repo.get_formatted_name())).cyan().bright(),
            DisplayVersion::Implicit(repo.get_version()),
            if failed > 0 {
                Cow::Owned(format!(", {} failed", style(failed).bold()))
            } else {
                Cow::Borrowed("")
            }
        ))
        .with_success()
        .print();

    Ok(())
}

pub async fn source_remote_repo(url: &Url, branch: &str) -> Result<(), AppError> {
    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.set_belle_prefix("Fetching");
    pb.set_message("package manifest");

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
        .with_success()
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
        .with_success()
        .print();

    Ok(())
}
