use anyhow::Context;
use nom::combinator::Opt;
use pubgrub::SemanticVersion;
use tabled::{Table, settings::Style};

use crate::{
    config,
    fetch::{
        client::{AFPRepo, BelleClient},
        metadata::{RepoMetadata, dependency},
    },
    registry::Package,
};

pub(in crate::fetch) mod client;
pub(in crate::fetch) mod metadata;

pub async fn list_repositories(limit: usize) -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    let mut afp_repos = client.get_afp_repos(limit).await?;
    afp_repos.reverse();

    let mut table = Table::new(&afp_repos);
    table.with(Style::rounded());

    println!("{}", table);
    println!("Found {} repositories.", afp_repos.len());

    return Ok(());
}

pub async fn fetch_meta(repo_name: Option<String>) -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    let repo = match repo_name {
        Some(name) => {
            let repo = client.get_repo(&name).await?;
            repo.with_context(|| format!("Could not find repo with name '{}'", name))?
        }
        None => {
            let latest_repo = client.get_afp_repos(1).await?;
            let a = latest_repo.first().map(|repo| repo.clone());
            a.context("Failed to fetch latest repo name")?
        }
    };

    let meta_bytes = client.get_metadata_archive(&repo).await?;
    let repo_metadata = RepoMetadata::new(repo, meta_bytes)?;

    for theory in repo_metadata.all_theories() {
        if theory.package_exists() {
            continue;
        }

        println!("Retrieving package {}", theory);
        let package = repo_metadata.create_package_meta(&theory.name, &client).await?;
        package.register()?;
    }

    return Ok(());
}

pub async fn get_package() {}
