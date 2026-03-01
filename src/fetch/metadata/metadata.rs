use anyhow::{Context, anyhow};
use pubgrub::SemanticVersion;
use std::collections::HashSet;
use std::io::Read;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

use crate::config::BelleConfig;
use crate::fetch::AFPRepo;
use crate::fetch::client::BelleClient;
use crate::fetch::metadata::{AuthorMetadata, RepoMetadata, TheoryMetadata, dependency};
use crate::registry::{AliasPackage, Package, PackageAuthor, PackageIdentifier, PackageSource};
use crate::util::date_to_version;

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
            .iter()
            .map(|(theory, meta)| PackageIdentifier {
                name: theory.clone(),
                version: date_to_version(&meta.date),
            })
            .collect();
    }

    /// Create package metadata by collecting keys and fetching theory ROOT file for dependencies
    pub async fn create_package_meta(
        &self,
        thy_name: &String,
        client: &BelleClient,
    ) -> anyhow::Result<(Package, bool, Vec<AliasPackage>)> {
        let meta = self
            .theories
            .get(thy_name)
            .ok_or_else(|| anyhow!("Theory '{}' does not exist in the repo metadata", thy_name))?;
        let version = date_to_version(&meta.date);

        // Fetch theories ROOT file from the repo
        let thy_root = client.get_thy_root(&self.repo, thy_name).await?;

        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

        // Extract sessions from the root file
        let sessions = dependency::parse_root(&thy_root)?;

        let session_names: Vec<&String> = sessions.iter().map(|s| &s.name).collect();
        let entry_deps: HashSet<&String> = sessions
            .iter()
            // Collect dependencies from all sessions
            .flat_map(|s| s.iter_all())
            // Remove sessions that are defined in this entry, as to not produce circular dependencies
            .filter(|dep| !session_names.contains(dep))
            .collect();

        let alias_packages: Vec<AliasPackage> = session_names
            .iter()
            .filter(|s| !s.eq(&&thy_name))
            .map(|s| AliasPackage {
                name: s.to_string(),
                version: version.clone(),
                alias: PackageIdentifier {
                    name: thy_name.to_string(),
                    version: version.clone(),
                },
            })
            .collect();

        let mut fully_resolved = true;
        let dependencies: HashMap<String, SemanticVersion> = entry_deps
            .iter()
            .cloned()
            .map(|dependency| {
                if isabelle_packages.contains(&dependency) {
                    // Isabelle packages will depend on the isabelle version so this version does not matter
                    return (dependency.to_string(), SemanticVersion::one());
                }

                let dep_version = match self.theories.get(dependency) {
                    Some(meta) => date_to_version(&meta.date),
                    // Mark this version as none, meaning it needs to be further resolved (it may be an unknown alias)
                    None => {
                        fully_resolved = false;
                        SemanticVersion::zero()
                    }
                };

                //         "Theory '{}' depends on '{}' but that does not exist in the repo metadata",
                //         thy_name,
                //         dependency
                return (dependency.to_string(), dep_version);
            })
            .collect();

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

        // Return created package with all metadata
        let package = Package {
            name: thy_name.clone(),
            version,
            title: meta.title.clone(),
            date: meta.date,
            r#abstract: meta.r#abstract.clone(),
            licence: licence.clone(),
            topics: meta.topics.clone(),
            note: meta.note.clone(),
            authors: authors,
            contributors: contributors,
            dependencies,
            isabelles: HashSet::from([self.repo.get_version().clone()]),
            source: PackageSource { afp: self.repo.id },
            extra: meta.extra.clone(),
        };

        return Ok((package, fully_resolved, alias_packages));
    }
}
