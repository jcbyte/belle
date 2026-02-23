use anyhow::{Context, anyhow};
use pubgrub::SemanticVersion;
use std::io::Read;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

use crate::fetch::AFPRepo;
use crate::fetch::client::BelleClient;
use crate::fetch::metadata::{AuthorMetadata, RepoMetadata, TheoryMetadata, dependency};
use crate::registry::{Package, PackageAuthor, PackageIdentifier, PackageSource};

impl RepoMetadata {
    /// Fetch metadata from repo and parse it into interpreted repo metadata
    pub async fn get(repo: &AFPRepo, client: &BelleClient) -> anyhow::Result<Self> {
        // Download full metadata archive bytes from repo
        let bytes = client.get_metadata_archive(repo).await?;

        let mut authors: HashMap<String, AuthorMetadata> = HashMap::default();
        let mut licences: HashMap<String, String> = HashMap::default();
        let mut theories: HashMap<String, TheoryMetadata> = HashMap::new();

        // Walk through the archive
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader).context("Failed to read zip archive")?;

        let legacy = archive.file_names().any(|name| name.ends_with("metadata"));
        if archive.is_empty() || legacy {
            anyhow::bail!("Legacy AFP repo, the metadata cannot be fetched");
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let Some(name) = file.enclosed_name() else { continue };

            // Handler to read file content if required
            let mut read_content = || -> anyhow::Result<String> {
                let mut content = String::with_capacity(file.size() as usize);
                file.read_to_string(&mut content)?;
                Ok(content)
            };

            // Match file name to check if we should handle it
            if name.ends_with("authors.toml") {
                // Create authors from "authors.toml"
                let content = read_content()?;
                authors = RepoMetadata::parse_authors(&content)?;
            } else if name.ends_with("licenses.toml") {
                // Create licences from "licenses.toml"
                let content = read_content()?;
                licences = RepoMetadata::parse_licences(&content)?;
            } else if name.parent().map_or(false, |p| p.ends_with("entries")) {
                // Each TOML file in the `entries/` subfolder represents a theory
                let Some(thy_name) = name.file_stem().map(|tn| tn.to_string_lossy().to_string()) else {
                    continue;
                };

                // Insert these separately into the hashable
                let content = read_content()?;
                let theory_metadata = RepoMetadata::parse_theory(&content)?;
                theories.insert(thy_name, theory_metadata);
            }
        }

        return Ok(RepoMetadata {
            repo: repo.clone(),
            authors,
            licences,
            theories,
        });
    }

    /// Get all theories within the repo metadata
    pub fn all_theories(&self) -> Vec<PackageIdentifier> {
        return self
            .theories
            .keys()
            .map(|theory| PackageIdentifier {
                name: theory.clone(),
                version: self.repo.get_version().clone(),
            })
            .collect();
    }

    /// Create package metadata by collecting keys and fetching theory ROOT file for dependencies
    pub async fn create_package_meta(&self, thy_name: &String, client: &BelleClient) -> anyhow::Result<Package> {
        let meta = self
            .theories
            .get(thy_name)
            .ok_or_else(|| anyhow!("Theory '{}' does not exist in the repo metadata", thy_name))?;

        // Fetch theories ROOT file from the repo
        let thy_root = client.get_thy_root(&self.repo, thy_name).await?;
        // Extract the dependency list
        let deps = dependency::extract_root_deps(&thy_root)?;
        // All dependencies will require the same version that this theory file is part of
        let dependencies: HashMap<String, SemanticVersion> =
            deps.iter_all().cloned().map(|s| (s, self.repo.get_version().clone())).collect();

        // Get licence from matching its key
        let licence = self.licences.get(&meta.licence_key).ok_or_else(|| {
            anyhow!(
                "Licence '{}' for theory '{}' does not exist in the repo metadata",
                meta.licence_key,
                thy_name
            )
        })?;

        // Get authors and contributors by matching there keys
        let authors = meta
            .author_keys
            .iter()
            .map(|ak| {
                self.authors
                    .get(ak)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow!(
                            "Author '{}' for theory '{}' does not exist in the repo metadata",
                            ak,
                            thy_name
                        )
                    })
                    .map(PackageAuthor::from) // Convert to the correct format
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let contributors = meta
            .contributor_keys
            .iter()
            .map(|ck| {
                self.authors
                    .get(ck)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow!(
                            "Author '{}' for theory '{}' does not exist in the repo metadata",
                            ck,
                            thy_name
                        )
                    })
                    .map(PackageAuthor::from) // Convert to the correct format
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Return created package will all metadata
        let package = Package {
            name: thy_name.clone(),
            version: self.repo.get_version().clone(),
            title: meta.title.clone(),
            date: meta.date,
            r#abstract: meta.r#abstract.clone(),
            licence: licence.clone(),
            topics: meta.topics.clone(),
            note: meta.note.clone(),
            authors: authors,
            contributors: contributors,
            dependencies,
            source: PackageSource { repo: self.repo.id },
            extra: meta.extra.clone(),
        };

        return Ok(package);
    }
}
