use std::path::Path;

use anyhow::Context;
use regex::Regex;
use serde::Deserialize;

use crate::fetch::metadata::RepoMetadata;

#[derive(Deserialize, Debug)]
pub struct AFPRepo {
    id: i32,
    name: String,
}

pub struct BelleClient {
    client: reqwest::Client,
}

impl BelleClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            // Include a custom user agent for politeness
            .user_agent("belle-client")
            .build()
            .context("Failed to create reqwest Client")?;

        return Ok(Self { client });
    }

    pub async fn get_afp_repos(&self) -> anyhow::Result<Vec<AFPRepo>> {
        let afp_repo_list_url =
            "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?order_by=last_activity_at&sort=desc";

        // Retrieve all repos/projects within the `isa-afp` group
        let repos: Vec<AFPRepo> = self
            .client
            .get(afp_repo_list_url)
            .send()
            .await
            .context("Failed to send request to Hetapod")?
            .json()
            .await
            .context("Failed to parse JSON response from Hetapod")?;

        // Only keep repos which match the name of the AFP
        let re = Regex::new(r"^afp-[\d-]+$").context("Invalid regex pattern for AFP repository name")?;
        let afp_repos: Vec<AFPRepo> = repos.into_iter().filter(|p| re.is_match(&p.name)).collect();

        return Ok(afp_repos);
    }

    pub async fn get_thys(&self, repo: &AFPRepo) -> anyhow::Result<Vec<String>> {
        let repo_entries_tree_url = format!(
            // ! Note this hard coded `2000` per page
            "https://foss.heptapod.net/api/v4/projects/{}/repository/tree?path=metadata%2Fentries&per_page=2000",
            repo.id
        );

        #[derive(Deserialize)]
        struct TreeItem {
            name: String,
            path: String,
        }

        // Retrieve tree listing of all files within the `/entries` (listing all all theories)
        let entries_tree: Vec<TreeItem> = self
            .client
            .get(repo_entries_tree_url)
            .header("User-Agent", "belle-client")
            .send()
            .await
            .with_context(|| format!("Failed to fetch entires list for '{}' repo", repo.name))?
            .json()
            .await
            .with_context(|| format!("Failed to parse JSON response of entires list for '{}' repo", repo.name))?;

        // Remove the `.toml` extension from each theories metadata file
        let thys: Vec<String> = entries_tree
            .iter()
            .filter_map(|e| Path::new(&e.name).file_stem().map(|f| f.to_string_lossy().to_string()))
            .collect();

        Ok(thys)
    }

    pub async fn get_metadata_archive(&self, repo: &AFPRepo) -> anyhow::Result<bytes::Bytes> {
        let meta_archive_url = format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata",
            repo.id
        );

        // Retrieve the bytes for the archive at `/metadata` for the given repo
        let bytes = self
            .client
            .get(meta_archive_url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch metadata archive for '{}' repo", repo.name))?
            .bytes()
            .await
            .with_context(|| format!("Failed to read metadata archive bytes for '{}' repo", repo.name))?;

        return Ok(bytes);
    }

    pub async fn get_thy_root(&self, repo: &AFPRepo, thy: &String) -> anyhow::Result<String> {
        let root_file_url = format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/files/{}/raw",
            repo.id,
            format!("thys%2F{}%2FROOT", thy)
        );

        // Retrieve the raw string of the ROOT file for the given theory and repo
        let file_content = self
            .client
            .get(root_file_url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch ROOT file for '{}' in '{}' repo", thy, repo.name))?
            .text()
            .await
            .with_context(|| format!("Failed to read ROOT file from '{}' in '{}' repo", thy, repo.name))?;

        return Ok(file_content);
    }
}
