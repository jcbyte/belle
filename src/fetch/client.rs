use anyhow::Context;
use regex::Regex;

use crate::fetch::afp_repo::AFPRepo;

pub struct BelleClient {
    client: reqwest::Client,
}

impl BelleClient {
    /// Create reqwest client to use throughout
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            // Include a custom user agent for politeness
            .user_agent("belle-client")
            .build()
            .context("Failed to create reqwest Client")?;

        return Ok(Self { client });
    }

    /// Retrieve all repos within the AFP repository up to given limit
    pub async fn get_afp_repos(&self, limit: usize) -> anyhow::Result<Vec<AFPRepo>> {
        // Regex to match an AFP repos name
        let re = Regex::new(r"^afp-[\d-]+$").context("Failed to create regex pattern for AFP repository name")?;

        let mut afp_repos: Vec<AFPRepo> = Vec::new();
        let mut page = 0;

        let per_page: usize = 25;

        // Continue iterating over pages of results until there is no more results or we reach our limit
        loop {
            // Retrieve repos/projects within the `isa-afp` group
            let afp_repo_list_url = format!(
                "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?order_by=last_activity_at&sort=desc&per_page={}&page={}",
                per_page, page
            );

            let repos: Vec<AFPRepo> = self
                .client
                .get(afp_repo_list_url)
                .send()
                .await
                .context("Failed to send request to Hetapod")?
                .json()
                .await
                .context("Failed to parse JSON response from Hetapod")?;

            let received_count = repos.len();

            // If repos is empty then there are no more results
            if repos.is_empty() {
                break;
            }

            // Only keep repos which match the name of the AFP
            let retrieved_repos: Vec<AFPRepo> = repos.into_iter().filter(|p| re.is_match(&p.name)).collect();

            // Add the found repos into our collecting list
            afp_repos.extend(retrieved_repos);

            // If the received amount was less than the requested per page there is no more pages
            // Or if we have enough repos then return
            if received_count < per_page || afp_repos.len() >= limit {
                // Truncate to ensure we have exactly the number requested (in case we went over)
                afp_repos.truncate(limit);
                break;
            }

            // Continue collecting repos on the next page
            page += 1;
        }

        return Ok(afp_repos);
    }

    /// Get a singular repo (id) from its name, or `None` is it does not exist
    pub async fn get_repo(&self, name: &String) -> anyhow::Result<Option<AFPRepo>> {
        // Query AFP group for repo with exact name
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

        // Return the first entry (if it exists) as this will be the requested repo
        return Ok(repo.first().map(|repo| repo.clone()));
    }

    /// Retrieve the metadata archive for a given repo
    pub async fn get_metadata_archive(&self, repo: &AFPRepo) -> anyhow::Result<bytes::Bytes> {
        // Retrieve the bytes for the archive at `/metadata` for the given repo
        let meta_archive_url = format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata",
            repo.id
        );

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

    /// Retrieve the ROOT file for a given theory
    pub async fn get_thy_root(&self, repo: &AFPRepo, thy: &String) -> anyhow::Result<String> {
        // Retrieve the raw string of the ROOT file at `/thys/$thy/ROOT` for the given theory and repo
        let root_file_url = format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/files/{}/raw",
            repo.id,
            format!("thys%2F{}%2FROOT", thy)
        );

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
