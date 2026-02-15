use std::{collections::HashMap, fmt::format, io::Cursor, path::Path};

use anyhow::Context;
use regex::Regex;
use serde::Deserialize;
use zip::ZipArchive;

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

#[derive(Deserialize)]
struct TreeItem {
    name: String,
    path: String,
}
// let releases_file_url = format!(
//     "https://foss.heptapod.net/api/v4/projects/{}/repository/files/metadata%2Freleases.toml/raw",
//     repo.id
// );

async fn get_thys(repo: &AFPRepo) -> anyhow::Result<Vec<String>> {
    let repo_entries_tree_url = format!(
        // ! Note this hard coded `2000` per page
        "https://foss.heptapod.net/api/v4/projects/{}/repository/tree?path=metadata%2Fentries&per_page=2000",
        repo.id
    );

    let client = reqwest::Client::new();
    let entries_tree: Vec<TreeItem> = client
        .get(repo_entries_tree_url)
        .header("User-Agent", "belle-client")
        .send()
        .await
        .with_context(|| format!("Failed to fetch entires list for '{}' repo", repo.name))?
        .json()
        .await
        .with_context(|| format!("Failed to parse JSON response of entires list for '{}' repo", repo.name))?;

    let thys: Vec<String> = entries_tree
        .iter()
        .filter_map(|e| Path::new(&e.name).file_stem().map(|f| f.to_string_lossy().to_string()))
        .collect();
    Ok(thys)
}

async fn get_repo_meta(repo: &AFPRepo) -> anyhow::Result<()> {
    let meta_archive_url = format!(
        "https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata",
        repo.id
    );

    let client = reqwest::Client::builder()
        .user_agent("belle-client")
        .build()
        .context("Failed to create reqwest Client")?;

    let response_bytes = client
        .get(meta_archive_url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch metadata for '{}' repo", repo.name))?
        .bytes()
        .await
        .with_context(|| format!("Failed to read metadata archive bytes for '{}' repo", repo.name))?;

    let tmp = tempfile::tempdir().context("Failed to create temporary directory")?;
    let reader = Cursor::new(response_bytes);
    let mut archive = ZipArchive::new(reader).context("Failed to read zip archive")?;
    archive.extract(tmp).context("Failed to extract zip archive")?;

    // todo continue

    return Ok(());
    // https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata/entries
}

// todo get dependencies through ROOT files (this will be more difficult)

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let afp_repos = get_afp_repos().await?;
    // let latest_repo = afp_repos.first().context("No latest AFP repository could be found")?;
    let latest_repo = &AFPRepo {
        id: 2228,
        name: String::from("afp-2025-2"),
    };
    println!("name: {} {}", latest_repo.name, latest_repo.id);

    // get_repo_meta(latest_repo).await?;
    let thys = get_thys(latest_repo).await?;
    println!("{:#?}", thys);

    return Ok(());
}
