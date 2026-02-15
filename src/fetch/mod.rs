use crate::fetch::{
    client::{AFPRepo, BelleClient},
    metadata::{RepoMetadata, dependency},
};

pub(in crate::fetch) mod client;
pub(in crate::fetch) mod metadata;

pub async fn update_meta() -> anyhow::Result<()> {
    let client = BelleClient::new()?;

    let afp_repos = client.get_afp_repos().await?;

    let latest_repo = afp_repos.first().unwrap();

    // let thys = client.get_thys(latest_repo).await?;
    // todo check this against local copy, if this is invalid we need to refetch the repo

    // let meta_bytes = client.get_metadata_archive(latest_repo).await?;
    // let repo_metadata = RepoMetadata::try_from(meta_bytes)?;
    // println!("{:#?}", repo_metadata);

    let a = client
        .get_thy_root(latest_repo, &String::from("Abortable_Linearizable_Modules"))
        .await?;

    println!("{:#?} {}", latest_repo, a);

    return Ok(());
}

pub async fn get_package() {}
