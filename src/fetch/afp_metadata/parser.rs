use std::collections::HashMap;

use crate::fetch::afp_metadata::{
    AFPAuthorMeta, AFPEntryMeta, AFPLicenceMeta, AuthorMetadata, EntryMetadata, RepoMetadata,
};

impl RepoMetadata {
    /// Convert from raw received data to our author metadata interpretation
    pub(super) fn parse_authors(toml_content: &str) -> Result<HashMap<String, AuthorMetadata>, toml::de::Error> {
        let authors_raw: HashMap<String, AFPAuthorMeta> = toml::from_str(toml_content)?;

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

        Ok(authors)
    }

    /// Convert from raw received data to our licence metadata interpretation
    pub(super) fn parse_licences(toml_content: &str) -> Result<HashMap<String, String>, toml::de::Error> {
        let licences_raw: HashMap<String, AFPLicenceMeta> = toml::from_str(toml_content)?;

        let licences = licences_raw
            .into_iter()
            .map(|(licence_id, licence)| (licence_id, licence.name))
            .collect();

        Ok(licences)
    }

    /// Convert from raw received data to our entry metadata interpretation
    pub(super) fn parse_entry(toml_content: &str) -> Result<EntryMetadata, toml::de::Error> {
        let entry_raw: AFPEntryMeta = toml::from_str(toml_content)?;

        // Add dois, and pubs into extra if they exist
        let mut extra_table = entry_raw.extra;
        if !entry_raw.related.dois.is_empty() {
            extra_table.insert("dois".to_string(), entry_raw.related.dois.into());
        }
        if !entry_raw.related.pubs.is_empty() {
            extra_table.insert("pubs".to_string(), entry_raw.related.pubs.into());
        }

        let entry = EntryMetadata {
            title: entry_raw.title,
            date: entry_raw.date,
            r#abstract: entry_raw.r#abstract,
            licence_key: entry_raw.license,
            topics: entry_raw.topics,
            note: entry_raw.note.filter(|n| !n.is_empty()),
            author_keys: entry_raw.authors.into_keys().collect(),
            contributor_keys: entry_raw.contributors.into_keys().collect(),
            extra: extra_table,
        };

        Ok(entry)
    }
}
