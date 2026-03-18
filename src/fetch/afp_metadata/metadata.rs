use pubgrub::SemanticVersion;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Read;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

use crate::config::BelleConfig;
use crate::error::{AppError, ArchiveErrorContext, IoErrorContext, ParseErrorContext};
use crate::fetch::afp_metadata::error::MetadataError;
use crate::fetch::afp_metadata::{AuthorMetadata, RepoMetadata, TheoryMetadata, root_parser};
use crate::fetch::client::BelleClient;
use crate::fetch::error::FetchError;
use crate::fetch::{AFPRepo, ReturnedPackages};
use crate::registry::{AliasPackage, Package, PackageAuthor, PackageIdentifier, PackageSource, get_package_versions};
use crate::util::date_to_version;

impl RepoMetadata {
    /// Fetch metadata from repo and parse it into interpreted repo metadata
    pub async fn get(repo: &AFPRepo, client: &BelleClient) -> Result<Self, AppError> {
        // Download full metadata archive bytes from repo
        let bytes = client.get_afp_metadata_archive(repo).await?;

        let mut authors: HashMap<String, AuthorMetadata> = HashMap::default();
        let mut licences: HashMap<String, String> = HashMap::default();
        let mut theories: HashMap<String, TheoryMetadata> = HashMap::new();

        // Walk through the archive
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader).report_read(format!("{} metadata archive", repo.name))?;

        let legacy = archive.file_names().any(|name| name.ends_with("metadata"));
        if archive.is_empty() || legacy {
            return Err(FetchError::LegacyAfp {
                afp_name: repo.name.to_string(),
            }
            .into());
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).report_index(format!("{} metadata archive", repo.name), i)?;
            // If the path is unsafe, skip
            let Some(filename) = file.enclosed_name() else { continue };

            // Handler to read file content if required
            let mut read_content = || -> Result<String, std::io::Error> {
                let mut content = String::with_capacity(file.size() as usize);
                file.read_to_string(&mut content)?;
                Ok(content)
            };

            // Match file name to check if we should handle it
            if filename.ends_with("authors.toml") {
                // Create authors from "authors.toml"
                let content = read_content().report_read(format!("authors for {} repository", repo.name), &filename)?;
                authors = RepoMetadata::parse_authors(&content)
                    .report_data(format!("authors for {} repository", repo.name))?;
            } else if filename.ends_with("licenses.toml") {
                // Create licences from "licenses.toml"
                let content =
                    read_content().report_read(format!("licences for {} repository", repo.name), &filename)?;
                licences = RepoMetadata::parse_licences(&content)
                    .report_data(format!("licences for {} repository", repo.name))?;
            } else if filename.parent().is_some_and(|p| p.ends_with("entries")) {
                // Each TOML file in the `entries/` subfolder represents a theory
                let Some(thy_name) = filename.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };

                // Insert these separately into the hashable
                let content = read_content()
                    .report_read(format!("theory {} for {} repository", thy_name, repo.name), &filename)?;
                let theory_metadata = RepoMetadata::parse_theory(&content)
                    .report_data(format!("theory {} for {} repository", thy_name, repo.name))?;
                theories.insert(thy_name.to_string(), theory_metadata);
            }
        }

        Ok(RepoMetadata {
            repo: repo.clone(),
            authors,
            licences,
            theories,
            seen_aliases: RefCell::new(HashMap::new()),
        })
    }

    /// Get all theories within the repo metadata
    pub fn all_theories(&self) -> Vec<PackageIdentifier> {
        self.theories
            .iter()
            .map(|(theory, meta)| PackageIdentifier::new(theory, date_to_version(&meta.date)))
            .collect()
    }

    /// Create package metadata by collecting keys and fetching theory ROOT file for dependencies
    pub async fn create_package_meta(
        &self,
        thy_name: &String,
        client: &BelleClient,
    ) -> Result<(ReturnedPackages, bool), AppError> {
        let meta = self.theories.get(thy_name).ok_or_else(|| MetadataError::NoPackage {
            package: thy_name.to_string(),
        })?;
        let version = date_to_version(&meta.date);

        // Fetch theories ROOT file from the repo
        let thy_root = client.get_afp_thy_root(&self.repo, thy_name).await?;

        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

        // Extract sessions from the root file
        let sessions = root_parser::parse_root(&thy_root)?;

        let session_names: Vec<&String> = sessions.iter().map(|s| &s.name).collect();
        let entry_deps: HashSet<&String> = sessions
            .iter()
            // Collect dependencies from all sessions
            .flat_map(|s| s.iter_all())
            // Remove sessions that are defined in this entry, as to not produce circular dependencies
            .filter(|dep| !session_names.contains(dep))
            .collect();

        let provides_packages: Vec<String> = session_names.into_iter().filter(|s| !s.eq(&thy_name)).cloned().collect();
        let alias_packages: Vec<AliasPackage> = provides_packages
            .iter()
            .map(|s| AliasPackage {
                name: s.to_string(),
                version,
                alias: PackageIdentifier::new(thy_name, version),
            })
            .collect();

        // Add seen aliases to internal cache for resolving later
        let mut seen_aliases = self.seen_aliases.borrow_mut();
        for alias in &alias_packages {
            seen_aliases.insert(alias.name.clone(), thy_name.clone());
        }

        let mut fully_resolved = true;
        let dependencies: HashMap<String, SemanticVersion> = entry_deps
            .iter()
            .cloned()
            .map(|dependency| {
                if isabelle_packages.contains(dependency) {
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

                (dependency.to_string(), dep_version)
            })
            .collect();

        // Get licence from matching its key
        let licence = self.licences.get(&meta.licence_key).ok_or_else(|| MetadataError::MissingData {
            name: format!("licence {}", meta.licence_key),
            package: thy_name.to_string(),
        })?;

        // Get authors and contributors by matching there keys
        let authors = meta
            .author_keys
            .iter()
            .map(|author_key| {
                self.authors
                    .get(author_key)
                    .cloned()
                    .ok_or_else(|| MetadataError::MissingData {
                        name: format!("author {}", author_key),
                        package: thy_name.to_string(),
                    })
                    .map(PackageAuthor::from) // Convert to the correct format
            })
            .collect::<Result<Vec<_>, MetadataError>>()?;
        let contributors = meta
            .contributor_keys
            .iter()
            .map(|contributor_key| {
                self.authors
                    .get(contributor_key)
                    .cloned()
                    .ok_or_else(|| MetadataError::MissingData {
                        name: format!("author {}", contributor_key),
                        package: thy_name.to_string(),
                    })
                    .map(PackageAuthor::from) // Convert to the correct format
            })
            .collect::<Result<Vec<_>, MetadataError>>()?;

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
            authors,
            contributors,
            provides: provides_packages,
            dependencies,
            isabelles: HashSet::from([*self.repo.get_version()]),
            source: PackageSource::Afp(self.repo.clone()),
            extra: meta.extra.clone(),
        };

        Ok((
            ReturnedPackages {
                package,
                aliases: alias_packages,
            },
            fully_resolved,
        ))
    }

    pub fn resolve_package_meta(&self, package: &mut Package) -> Result<(), MetadataError> {
        let seen_aliases = self.seen_aliases.borrow();

        let deps = package
            .dependencies
            .iter()
            .map(|(dep_name, dep_version)| {
                // If the version is zero then this dependency hasn't been resolved properly, try it now
                let version = if dep_version.eq(&SemanticVersion::zero()) {
                    let mut found_meta = None;

                    // Use seen aliases first, to try and resolve
                    if let Some(package_name) = seen_aliases.get(dep_name) {
                        let meta = self.theories.get(package_name).expect("A seen alias was set but did not find");
                        found_meta = Some(meta)
                    // If there was no seen alias check the registry for the alias
                    } else {
                        // Go though each version in case there are multiple connected to different packages
                        for package in get_package_versions(dep_name) {
                            // If the alias points to a package in the repo then this is the correct package
                            if let Some(meta) = self.theories.get(&package.name) {
                                found_meta = Some(meta);
                                break;
                            }
                        }
                    }

                    match found_meta {
                        // Use the version of the original package, as the alias points to the same version number
                        Some(meta) => Ok(date_to_version(&meta.date)),
                        None => Err(MetadataError::DependencyMissing {
                            package: package.name.to_string(),
                            dependency: dep_name.to_string(),
                        }),
                    }
                } else {
                    Ok(*dep_version)
                };
                Ok((dep_name.clone(), version?))
            })
            .collect::<Result<HashMap<String, SemanticVersion>, MetadataError>>()?;

        package.dependencies = deps;
        Ok(())
    }
}
