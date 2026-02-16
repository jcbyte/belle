use pubgrub::SemanticVersion;

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

pub async fn update_meta() -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    let afp_repos = client.get_afp_repos().await?;
    let latest_repo = afp_repos.first().unwrap();
    // todo we need to check this repo is a new toml one

    // let thys = client.get_thys(latest_repo).await?;
    // todo check this against local copy, if this is invalid we need to refetch the repo

    let meta_bytes = client.get_metadata_archive(latest_repo).await?;
    let repo_metadata = RepoMetadata::new(latest_repo.clone(), meta_bytes)?;

    for theory in repo_metadata.all_theories() {
        if Package::package_exists(theory, &repo_metadata.repo.get_version()) {
            continue;
        }

        println!("Retrieving package {}", theory);
        let package = repo_metadata.create_package_meta(theory, &client).await?;
        package.register()?;
    }

    return Ok(());
}

pub async fn get_package() {}
