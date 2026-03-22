use regex::Regex;
use url::Url;

use crate::{
    config::BelleConfig,
    fetch::{
        AfpRepo, BelleClient,
        error::{FetchError, FetchErrorContext, FetchUrlContext},
    },
};

impl BelleClient {
    /// Retrieve all repos within the AFP repository up to given limit
    pub async fn get_afp_repos(&self, limit: usize) -> Result<Vec<AfpRepo>, FetchError> {
        // Regex to match an AFP repos name
        let re = Regex::new(r"^afp-[\d-]+$").expect("Invalid hardcoded regex expression");

        let mut afp_repos: Vec<AfpRepo> = Vec::new();
        let mut page = 1;

        let per_page: usize = 25;

        let afp_group = BelleConfig::read_config(|c| c.afp_group.clone());

        // Continue iterating over pages of results until there is no more results or we reach our limit
        loop {
            // Retrieve repos/projects within the specified group
            let afp_repo_list_url = Url::parse(&format!(
                "https://foss.heptapod.net/api/v4/groups/{}/projects?order_by=created_at&sort=desc&per_page={}&page={}",
                afp_group, per_page, page
            ))
            .report_invalid_url("Hetapod afp list")?;

            let repos: Vec<AfpRepo> = self
                .client
                .get(afp_repo_list_url.clone())
                .send()
                .await
                .report_fetch("Hetapod afp list", &afp_repo_list_url)?
                .json()
                .await
                .report_reading_fetched("Hetapod afp list", &afp_repo_list_url)?;

            // If repos is empty then there are no more results
            if repos.is_empty() {
                break;
            }

            let received_count = repos.len();
            // Only keep repos which match the name of the AFP
            let retrieved_repos: Vec<AfpRepo> = repos.into_iter().filter(|p| re.is_match(&p.name)).collect();

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

        Ok(afp_repos)
    }

    /// Get a singular repo (id) from its name, or `None` is it does not exist
    pub async fn get_afp_repo(&self, name: &str) -> Result<Option<AfpRepo>, FetchError> {
        let mut page = 1;

        let afp_group = BelleConfig::read_config(|c| c.afp_group.clone());

        loop {
            // Query AFP group for repo searching for name (this is a fuzzy search)
            let afp_repo_details_url = Url::parse(&format!(
                "https://foss.heptapod.net/api/v4/groups/{}/projects?search={}&per_page=1&page={}",
                afp_group, name, page
            ))
            .report_invalid_url("Hetapod project data")?;

            let repo_collection: Vec<AfpRepo> = self
                .client
                .get(afp_repo_details_url.clone())
                .send()
                .await
                .report_fetch("Hetapod project data", &afp_repo_details_url)?
                .json()
                .await
                .report_reading_fetched("Hetapod project data", &afp_repo_details_url)?;

            let possible_repo = repo_collection.first();

            match possible_repo {
                // If the results are empty, the repo name doesn't exist in the project
                None => return Ok(None),

                // If we have a repo check the name is exact, as Hetapod uses a fuzzy search
                Some(repo) if repo.name == name => return Ok(Some(repo.clone())),

                // If we have a repo but it is not an exact match, check the next page
                _ => {}
            }

            page += 1;
        }
    }

    /// Retrieve the metadata archive for a given repo
    pub async fn get_afp_metadata_archive(&self, repo: &AfpRepo) -> Result<bytes::Bytes, FetchError> {
        // Retrieve the bytes for the archive at `/metadata` for the given repo
        let meta_archive_url = Url::parse(&format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata",
            repo.id
        ))
        .report_invalid_url(format!("{} metadata archive", repo.name))?;

        let bytes = self
            .client
            .get(meta_archive_url.clone())
            .send()
            .await
            .report_fetch(format!("{} metadata archive", repo.name), &meta_archive_url)?
            .bytes()
            .await
            .report_reading_fetched(format!("{} metadata archive", repo.name), &meta_archive_url)?;

        Ok(bytes)
    }

    /// Retrieve the ROOT file for a given entry
    pub async fn get_afp_entry_root(&self, repo: &AfpRepo, entry: &str) -> Result<String, FetchError> {
        // Retrieve the raw string of the ROOT file at `/thys/$thy/ROOT` for the given entry and repo
        let root_file_url = Url::parse(&format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/files/thys%2F{}%2FROOT/raw",
            repo.id, entry
        ))
        .report_invalid_url(format!("ROOT file for {} in {}", entry, repo.name))?;

        let file_content = self
            .client
            .get(root_file_url.clone())
            .send()
            .await
            .report_fetch(format!("ROOT file for {} in {}", entry, repo.name), &root_file_url)?
            .text()
            .await
            .report_reading_fetched(format!("ROOT file for {} in {}", entry, repo.name), &root_file_url)?;

        Ok(file_content)
    }

    pub async fn get_afp_package(&self, entry: &str, repo: &AfpRepo) -> Result<bytes::Bytes, FetchError> {
        let package_archive_url = Url::parse(&format!(
            "https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=thys%2F{}",
            repo.id, entry
        ))
        .report_invalid_url(format!("package source for {} in {}", entry, repo.name))?;

        let bytes = self
            .client
            .get(package_archive_url.clone())
            .send()
            .await
            .report_fetch(
                format!("package source for {} in {}", entry, repo.name),
                &package_archive_url,
            )?
            .bytes()
            .await
            .report_reading_fetched(
                format!("package source for {} in {}", entry, repo.name),
                &package_archive_url,
            )?;

        Ok(bytes)
    }
}
