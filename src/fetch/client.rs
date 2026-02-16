use std::path::Path;

use anyhow::Context;
use pubgrub::SemanticVersion;
use regex::Regex;
use serde::Deserialize;

use crate::fetch::metadata::RepoMetadata;
use tabled::Tabled;

#[derive(Deserialize, Debug, Clone, Tabled)]
pub struct AFPRepo {
    #[tabled(skip)]
    pub id: i32,
    #[tabled(rename = "Repo Name")]
    pub name: String,

    #[serde(skip)]
    #[tabled(display("display_version_private", self))]
    pub version: (), // Use () as a placeholder
}

impl AFPRepo {
    pub fn get_version(&self) -> SemanticVersion {
        let name_parts: Vec<u32> = self.name.split('-').filter_map(|s| s.parse::<u32>().ok()).collect();

        let major = name_parts.get(0).unwrap_or(&0);
        let minor = name_parts.get(1).unwrap_or(&0);
        let patch = name_parts.get(2).unwrap_or(&0);

        return SemanticVersion::new(*major, *minor, *patch);
    }
}

fn display_version_private(_: &(), repo: &AFPRepo) -> String {
    repo.get_version().to_string()
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

    pub async fn get_afp_repos(&self, limit: usize) -> anyhow::Result<Vec<AFPRepo>> {
        let re = Regex::new(r"^afp-[\d-]+$").context("Failed to create regex pattern for AFP repository name")?;

        let mut afp_repos: Vec<AFPRepo> = Vec::new();
        let mut page = 0;

        loop {
            let afp_repo_list_url = format!(
                "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?order_by=last_activity_at&sort=desc&per_page=25&page={}",
                page
            );

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

            if repos.is_empty() {
                break;
            }

            // Only keep repos which match the name of the AFP
            let retrieved_repos: Vec<AFPRepo> = repos.into_iter().filter(|p| re.is_match(&p.name)).collect();

            afp_repos.extend(retrieved_repos);

            if afp_repos.len() >= limit {
                afp_repos.truncate(limit);
                break;
            }
            page += 1;
        }

        return Ok(afp_repos);
    }

    pub async fn get_repo(&self, name: &String) -> anyhow::Result<Option<AFPRepo>> {
        let afp_repo_details_url = format!(
            "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?search={}&per_page=1",
            name
        );

        let repo: Vec<AFPRepo> = self
            .client
            .get(afp_repo_details_url)
            .send()
            .await
            .context("Failed to send request to Hetapod")?
            .json()
            .await
            .context("Failed to parse JSON response from Hetapod")?;

        return Ok(repo.first().map(|repo| repo.clone()));
    }

    // pub async fn get_thys(&self, repo: &AFPRepo) -> anyhow::Result<Vec<String>> {
    //     let repo_entries_tree_url = format!(
    //         // ! Note this hard coded `2000` per page
    //         "https://foss.heptapod.net/api/v4/projects/{}/repository/tree?path=metadata%2Fentries&per_page=2000",
    //         repo.id
    //     );

    //     #[derive(Deserialize)]
    //     struct TreeItem {
    //         name: String,
    //         path: String,
    //     }

    //     // Retrieve tree listing of all files within the `/entries` (listing all all theories)
    //     let entries_tree: Vec<TreeItem> = self
    //         .client
    //         .get(repo_entries_tree_url)
    //         .header("User-Agent", "belle-client")
    //         .send()
    //         .await
    //         .with_context(|| format!("Failed to fetch entires list for '{}' repo", repo.name))?
    //         .json()
    //         .await
    //         .with_context(|| format!("Failed to parse JSON response of entires list for '{}' repo", repo.name))?;

    //     // Remove the `.toml` extension from each theories metadata file
    //     let thys: Vec<String> = entries_tree
    //         .iter()
    //         .filter_map(|e| Path::new(&e.name).file_stem().map(|f| f.to_string_lossy().to_string()))
    //         .collect();

    //     Ok(thys)
    // }

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
