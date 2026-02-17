use std::collections::HashMap;

use anyhow::Context;

use super::schema;

use super::{AuthorMetadata, RepoMetadata, TheoryMetadata};

impl RepoMetadata {
    /// Convert from raw received data to our author metadata interpretation
    pub(super) fn parse_authors(toml_content: &String) -> anyhow::Result<HashMap<String, AuthorMetadata>> {
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
                        email: author.emails.values().next().map(|email| email.to_string()), // Only keep one email from authors
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

    /// Convert from raw received data to our licence metadata interpretation
    pub(super) fn parse_licences(toml_content: &String) -> anyhow::Result<HashMap<String, String>> {
        let licences_raw: HashMap<String, schema::MetaLicence> =
            toml::from_str(toml_content).context("Failed to parse TOML for licences metadata")?;

        let licences = licences_raw
            .into_iter()
            .map(|(licence_id, licence)| (licence_id, licence.name))
            .collect();

        return Ok(licences);
    }

    /// Convert from raw received data to our theory metadata interpretation
    pub(super) fn parse_theory(toml_content: &String) -> anyhow::Result<TheoryMetadata> {
        let theory_raw: schema::MetaTheory =
            toml::from_str(&toml_content).context("Failed to parse TOML for theory metadata")?;

        // Add dois, and pubs into extra if they exist
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
