use std::{collections::HashMap, fmt::format};

use anyhow::Context;
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct AFPRepo {
    id: i32,
    name: String,
}

async fn get_afp_repos() -> anyhow::Result<Vec<AFPRepo>> {
    let afp_repo_list_url =
        "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?order_by=last_activity_at&sort=desc";

    let client = reqwest::Client::new();
    let repos: Vec<AFPRepo> = client
        .get(afp_repo_list_url)
        .header("User-Agent", "belle-client")
        .send()
        .await
        .context("Failed to send request to Hetapod")?
        .json()
        .await
        .context("Failed to parse JSON response from Hetapod")?;

    let re = Regex::new(r"^afp-[\d-]+$").context("Invalid regex pattern for AFP repository name")?;
    let afp_repos: Vec<AFPRepo> = repos.into_iter().filter(|p| re.is_match(&p.name)).collect();

    return Ok(afp_repos);
}

type ReleaseMap = HashMap<String, HashMap<String, String>>;

async fn get_releases(repo: &AFPRepo) -> anyhow::Result<ReleaseMap> {
    let releases_file_url = format!(
        "https://foss.heptapod.net/api/v4/projects/{}/repository/files/metadata%2Freleases.toml/raw",
        repo.id
    );

    let client = reqwest::Client::new();
    let releases_content = client
        .get(releases_file_url)
        .header("User-Agent", "belle-client")
        .send()
        .await
        .with_context(|| format!("Failed to fetch releases file for '{}' repo", repo.name))?
        .text()
        .await
        .with_context(|| format!("Failed to read releases file for '{}' repo", repo.name))?;

    let releases = toml::from_str(&releases_content)
        .with_context(|| format!("Failed to parse TOML from releases file for '{}' repo", repo.name))?;

    Ok(releases)
}

async fn get_packages_meta(releases: ReleaseMap) {}
// https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata/entries

// todo get dependencies through ROOT files (this will be more difficult)

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let afp_repos = get_afp_repos().await?;
    let latest_repo = afp_repos.first().context("No latest AFP repository could be found")?;
    println!("name: {} {}", latest_repo.name, latest_repo.id);

    let releases = get_releases(latest_repo).await?;
    println!("{:#?}", releases);

    return Ok(());
}
