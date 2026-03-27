use pubgrub::SemanticVersion;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Read;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

use crate::config::BelleConfig;
use crate::error::{AppError, ArchiveErrorContext, IoErrorContext, ParseErrorContext};
use crate::fetch::afp_metadata::error::AfpMetadataError;
use crate::fetch::afp_metadata::{AuthorMetadata, EntryMetadata, RepoMetadata, root_parser};
use crate::fetch::client::BelleClient;
use crate::fetch::error::FetchError;
use crate::fetch::{AfpRepo, ReturnedPackages};
use crate::registry::{AliasPackage, Package, PackageAuthor, PackageIdentifier, PackageSource, get_package_versions};
use crate::util::date_to_version;

impl RepoMetadata {
    /// Fetch metadata from repo and parse it into interpreted repo metadata
    pub async fn get(repo: &AfpRepo, client: &BelleClient) -> Result<Self, AppError> {
        // Download full metadata archive bytes from repo
        let bytes = client.get_afp_metadata_archive(repo).await?;

        // Walk through the archive
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader).report_read(format!("{} metadata archive", repo.name))?;

        let legacy = archive.file_names().any(|name| name.ends_with("metadata"));
        if archive.is_empty() || legacy {
            return Err(FetchError::LegacyAfp { repo: repo.clone() }.into());
        }

        let mut authors: HashMap<String, AuthorMetadata> = HashMap::default();
        let mut licences: HashMap<String, String> = HashMap::default();
        let mut entries: HashMap<String, EntryMetadata> = HashMap::new();

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
                // Each TOML file in the `entries/` subfolder represents an entry (package)
                let Some(entry_name) = filename.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };

                // Insert these separately into the hashable
                let content = read_content()
                    .report_read(format!("entry {} for {} repository", entry_name, repo.name), &filename)?;
                let entry_metadata = RepoMetadata::parse_entry(&content)
                    .report_data(format!("entry {} for {} repository", entry_name, repo.name))?;
                entries.insert(entry_name.to_string(), entry_metadata);
            }
        }

        Ok(RepoMetadata {
            repo: repo.clone(),
            authors,
            licences,
            entries,
            seen_aliases: RefCell::new(HashMap::new()),
        })
    }

    /// Get all packages within the repo metadata
    pub fn all_packages(&self) -> Vec<PackageIdentifier> {
        self.entries
            .iter()
            .map(|(entry, meta)| PackageIdentifier::new(entry, date_to_version(&meta.date)))
            .collect()
    }

    /// Create package metadata by collecting keys and fetching entry ROOT file for session dependencies
    pub async fn create_package_meta(
        &self,
        entry_name: &str,
        client: &BelleClient,
    ) -> Result<(ReturnedPackages, Vec<String>), AppError> {
        let Some(meta) = self.entries.get(entry_name) else {
            return Err(AfpMetadataError::NoPackage {
                package: entry_name.to_string(),
            }
            .into());
        };
        let version = date_to_version(&meta.date);

        // Fetch entry ROOT file from the repo
        let entry_root = client.get_afp_entry_root(&self.repo, entry_name).await?;

        // Extract sessions from the root file
        let sessions = root_parser::parse_root(&entry_root)?;

        // Get all top-level session names the root file defines
        let session_names: Vec<&String> = sessions.iter().map(|s| &s.name).collect();
        // Get dependencies from all sessions
        let entry_deps: HashSet<&String> = sessions
            .iter()
            .flat_map(|s| s.iter_all())
            // Remove sessions that are defined in this entry, as to not produce circular dependencies
            .filter(|dep| !session_names.contains(dep))
            .collect();

        // Get packages that this session provides
        let provides_packages: Vec<String> = session_names
            .into_iter()
            // By filtering out the main session name, from all sessions defined in the root file
            .filter(|&name| name != entry_name)
            .cloned()
            .collect();
        // Convert this list into `AliasPackages`
        let alias_packages: Vec<AliasPackage> = provides_packages
            .iter()
            .map(|name| AliasPackage {
                name: name.to_string(),
                version,
                alias: PackageIdentifier::new(entry_name, version),
            })
            .collect();

        // Add seen aliases to internal cache for resolving later
        let mut seen_aliases = self.seen_aliases.borrow_mut();
        for alias in &alias_packages {
            seen_aliases.insert(alias.name.clone(), entry_name.to_string());
        }

        let mut missing_dependencies: Vec<String> = Vec::new();
        let dependencies: HashMap<String, SemanticVersion> = entry_deps
            .into_iter()
            .map(|dependency| {
                if BelleConfig::read_config(|c| c.isabelle_packages.contains(dependency)) {
                    // Isabelle packages will depend on the isabelle version so this version does not matter
                    return (dependency.to_string(), SemanticVersion::one());
                }

                let dep_version = match self.entries.get(dependency) {
                    // If the dependency is within the metadata we can get its correct version
                    Some(meta) => date_to_version(&meta.date),
                    // If not then mark this version as zero, meaning it needs to be further resolved (it may be an unknown alias)
                    None => {
                        missing_dependencies.push(dependency.clone());
                        SemanticVersion::zero()
                    }
                };

                (dependency.to_string(), dep_version)
            })
            .collect();

        // Get licence from matching its key
        let licence = self
            .licences
            .get(&meta.licence_key)
            .ok_or_else(|| AfpMetadataError::DataMissing {
                name: format!("licence {}", meta.licence_key),
                package: entry_name.to_string(),
            })?;

        // Get authors and contributors by matching their keys
        let authors = meta
            .author_keys
            .iter()
            .map(|author_key| {
                self.authors
                    .get(author_key)
                    .ok_or_else(|| AfpMetadataError::DataMissing {
                        name: format!("author {}", author_key),
                        package: entry_name.to_string(),
                    })
                    // Convert to the correct format
                    .cloned()
                    .map(PackageAuthor::from)
            })
            .collect::<Result<Vec<_>, AfpMetadataError>>()?;

        let contributors = meta
            .contributor_keys
            .iter()
            .map(|contributor_key| {
                self.authors
                    .get(contributor_key)
                    .ok_or_else(|| AfpMetadataError::DataMissing {
                        name: format!("contributor {}", contributor_key),
                        package: entry_name.to_string(),
                    })
                    // Convert to the correct format
                    .cloned()
                    .map(PackageAuthor::from)
            })
            .collect::<Result<Vec<_>, AfpMetadataError>>()?;

        // Return created package with all metadata
        Ok((
            ReturnedPackages {
                package: Package {
                    name: entry_name.to_string(),
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
                },
                aliases: alias_packages,
            },
            missing_dependencies,
        ))
    }

    pub fn resolve_package_meta(&self, package: &mut Package) -> Result<(), AppError> {
        let mut seen_aliases = self.seen_aliases.borrow_mut();

        for (dep_name, dep_version) in package.dependencies.iter_mut() {
            // If the version isn't zero then this dependency has already been resolved properly
            if *dep_version != SemanticVersion::zero() {
                continue;
            }

            // Use seen aliases first, to try and resolve
            if let Some(package_name) = seen_aliases.get(dep_name) {
                let meta = self.entries.get(package_name).expect("A seen alias was set but did not find");
                // Use the version of the original package, as the alias points to the same version number
                *dep_version = date_to_version(&meta.date);
                continue;
            }

            // If there was no seen alias check the registry for the alias
            // Go though each version in case there are multiple connected to different packages
            let mut resolved = false;
            if let Some(versions) = get_package_versions(dep_name) {
                for package in versions {
                    let resolved_package = package
                        .get_resolved_package_manifest()?
                        .expect("Package version listed, but not cannot be found");
                    // If the alias points to a package in the repo then this is the correct package
                    if let Some(meta) = self.entries.get(&resolved_package.name) {
                        // Use the version of the original package, as the alias points to the same version number
                        *dep_version = date_to_version(&meta.date);

                        // Update the seen aliases in case the appears again
                        seen_aliases.insert(dep_name.clone(), resolved_package.name.clone());
                        resolved = true;
                        break;
                    }
                }
            }

            if !resolved {
                return Err(AfpMetadataError::DependencyMissing {
                    package: package.name.clone(),
                    dependency: dep_name.clone(),
                }
                .into());
            }
        }

        Ok(())
    }
}
