use anyhow::{Context, anyhow};
use pubgrub::SemanticVersion;
use std::convert::TryFrom;
use std::io::Read;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

use crate::fetch::client::{AFPRepo, BelleClient};
use crate::registry::{Package, PackageAuthor, PackageIdentifier};

pub mod dependency;
mod schema;

#[derive(Debug, Clone)]
struct AuthorMetadata {
    name: String,
    email: Option<String>,
    homepages: Option<Vec<String>>,
    orcid: Option<String>,
}

#[derive(Debug, Clone)]
struct TheoryMetadata {
    title: String,
    date: toml::value::Date,
    r#abstract: String,
    licence_key: String,
    topics: Vec<String>,
    note: Option<String>,
    author_keys: Vec<String>,
    contributor_keys: Vec<String>,
    extra: toml::Table,
}

#[derive(Debug)]
pub struct RepoMetadata {
    repo: AFPRepo,
    authors: HashMap<String, AuthorMetadata>,
    licences: HashMap<String, String>,
    theories: HashMap<String, TheoryMetadata>,
}

impl RepoMetadata {
    pub fn new(repo: AFPRepo, bytes: bytes::Bytes) -> anyhow::Result<Self> {
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader).context("Failed to read zip archive")?;

        let mut authors: HashMap<String, AuthorMetadata> = HashMap::default();
        let mut licences: HashMap<String, String> = HashMap::default();
        let mut theories: HashMap<String, TheoryMetadata> = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let Some(name) = file.enclosed_name() else { continue };

            let mut read_content = || -> anyhow::Result<String> {
                let mut content = String::with_capacity(file.size() as usize);
                file.read_to_string(&mut content)?;
                Ok(content)
            };

            if name.ends_with("authors.toml") {
                let content = read_content()?;
                authors = RepoMetadata::parse_authors(&content)?;
            } else if name.ends_with("licenses.toml") {
                let content = read_content()?;
                licences = RepoMetadata::parse_licences(&content)?;
            } else if name.parent().map_or(false, |p| p.ends_with("entries")) {
                let Some(thy_name) = name.file_stem().map(|tn| tn.to_string_lossy().to_string()) else {
                    continue;
                };

                let content = read_content()?;
                let theory_metadata = RepoMetadata::parse_theory(&content)?;
                theories.insert(thy_name, theory_metadata);
            }
        }

        return Ok(RepoMetadata {
            repo,
            authors,
            licences,
            theories,
        });
    }

    pub fn all_theories(&self) -> impl Iterator<Item = PackageIdentifier> {
        return self.theories.keys().map(|theory| PackageIdentifier {
            name: theory.clone(),
            version: self.repo.get_version(),
        });
    }
}

impl RepoMetadata {
    fn parse_authors(toml_content: &String) -> anyhow::Result<HashMap<String, AuthorMetadata>> {
        let authors_raw: HashMap<String, schema::MetaAuthor> =
            toml::from_str(toml_content).context("Failed to parse TOML for authors metadata")?;

        let authors = authors_raw
            .into_iter()
            .map(|(author_id, author)| {
                (
                    author_id,
                    AuthorMetadata {
                        name: author.name,
                        orcid: author.orcid,
                        email: author.emails.values().next().map(|email| email.to_string()),
                        homepages: if author.homepages.is_empty() {
                            None
                        } else {
                            Some(author.homepages.into_values().collect())
                        },
                    },
                )
            })
            .collect();

        return Ok(authors);
    }

    fn parse_licences(toml_content: &String) -> anyhow::Result<HashMap<String, String>> {
        let licences_raw: HashMap<String, schema::MetaLicence> =
            toml::from_str(toml_content).context("Failed to parse TOML for licences metadata")?;

        let licences = licences_raw
            .into_iter()
            .map(|(licence_id, licence)| (licence_id, licence.name))
            .collect();

        return Ok(licences);
    }

    fn parse_theory(toml_content: &String) -> anyhow::Result<TheoryMetadata> {
        let theory_raw: schema::MetaTheory =
            toml::from_str(&toml_content).context("Failed to parse TOML for theory metadata")?;

        let mut extra_table = theory_raw.extra;
        if !theory_raw.related.dois.is_empty() {
            extra_table.insert(String::from("dois"), theory_raw.related.dois.into());
        }
        if !theory_raw.related.pubs.is_empty() {
            extra_table.insert(String::from("pubs"), theory_raw.related.pubs.into());
        }

        let theory = TheoryMetadata {
            title: theory_raw.title,
            date: theory_raw.date,
            r#abstract: theory_raw.r#abstract,
            licence_key: theory_raw.license,
            topics: theory_raw.topics,
            note: theory_raw.note.filter(|n| !n.is_empty()),
            author_keys: theory_raw.authors.into_keys().collect(),
            contributor_keys: theory_raw.contributors.into_keys().collect(),
            extra: extra_table,
        };

        return Ok(theory);
    }
}

impl From<AuthorMetadata> for PackageAuthor {
    fn from(meta: AuthorMetadata) -> Self {
        Self {
            name: meta.name,
            email: meta.email,
            homepages: meta.homepages,
            orcid: meta.orcid,
        }
    }
}

impl RepoMetadata {
    pub async fn create_package_meta(&self, thy_name: &String, client: &BelleClient) -> anyhow::Result<Package> {
        let meta = self
            .theories
            .get(thy_name)
            .ok_or_else(|| anyhow!("Theory '{}' does not exist in the repo metadata", thy_name))?;

        let version = self.repo.get_version();

        let thy_root = client.get_thy_root(&self.repo, thy_name).await?;
        let deps = dependency::extract_root_deps(&thy_root)?;

        let dependencies: HashMap<String, SemanticVersion> = deps.iter_all().cloned().map(|s| (s, version)).collect();

        let licence = self.licences.get(&meta.licence_key).ok_or_else(|| {
            anyhow!(
                "Licence '{}' for theory '{}' does not exist in the repo metadata",
                meta.licence_key,
                thy_name
            )
        })?;

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
                    .map(PackageAuthor::from)
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
                    .map(PackageAuthor::from)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

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
            extra: meta.extra.clone(),
        };

        return Ok(package);
    }
}
