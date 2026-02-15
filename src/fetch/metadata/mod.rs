use anyhow::Context;
use std::convert::TryFrom;
use std::io::Read;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

mod schema;
// use crate::fetch::metadata::schema;

struct AuthorMetadata {
    name: String,
    orcid: Option<String>,
    emails: Vec<String>,
    homepages: Vec<String>,
}

struct TheoryMetadata {
    title: String,
    date: toml::value::Date,
    thy_abstract: String,
    licence_key: String,
    topics: Vec<String>,
    note: Option<String>,
    author_keys: Vec<String>,
    contributor_keys: Vec<String>,
    extra: toml::Table,
}

pub struct RepoMetadata {
    authors: HashMap<String, AuthorMetadata>,
    licences: HashMap<String, String>,
    theories: HashMap<String, TheoryMetadata>,
}

impl TryFrom<bytes::Bytes> for RepoMetadata {
    type Error = anyhow::Error;

    fn try_from(bytes: bytes::Bytes) -> anyhow::Result<Self> {
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader).context("Failed to read zip archive")?;

        let mut authors: HashMap<String, AuthorMetadata> = HashMap::default();
        let mut licences: HashMap<String, String> = HashMap::default();
        let mut theories: HashMap<String, TheoryMetadata> = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let Some(name) = file.enclosed_name() else { continue };

            if name.ends_with("authors.toml") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                authors = RepoMetadata::parse_authors(&content)?;
            } else if name.ends_with("licenses.toml") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                licences = RepoMetadata::parse_licences(&content)?;
            } else if name.parent().map_or(false, |p| p.ends_with("entries")) {
                let mut content = String::new();
                file.read_to_string(&mut content)?;

                let Some(thy_name) = name.file_stem().map(|tn| tn.to_string_lossy().to_string()) else {
                    continue;
                };

                let theory_metadata = RepoMetadata::parse_theory(&content)?;
                theories.insert(thy_name, theory_metadata);
            }
        }

        return Ok(RepoMetadata {
            authors,
            licences,
            theories,
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
                        emails: author.emails.values().map(|email| email.to_string()).collect(),
                        homepages: author.homepages.into_values().collect(),
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
        extra_table.insert(String::from("dois"), toml::Value::Array(theory_raw.related.dois));
        extra_table.insert(String::from("pubs"), toml::Value::Array(theory_raw.related.pubs));

        let theory = TheoryMetadata {
            title: theory_raw.title,
            date: theory_raw.date,
            thy_abstract: theory_raw.r#abstract,
            licence_key: theory_raw.license,
            topics: theory_raw.topics,
            note: theory_raw.note,
            author_keys: theory_raw.authors.into_keys().collect(),
            contributor_keys: theory_raw.contributors.into_keys().collect(),
            extra: extra_table,
        };

        return Ok(theory);
    }
}
