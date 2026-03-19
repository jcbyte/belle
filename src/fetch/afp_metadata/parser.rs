use std::collections::HashMap;

use crate::fetch::afp_metadata::{
    AFPAuthorMeta, AFPLicenceMeta, AFPTheoryMeta, AuthorMetadata, RepoMetadata, TheoryMetadata,
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

    /// Convert from raw received data to our theory metadata interpretation
    pub(super) fn parse_theory(toml_content: &str) -> Result<TheoryMetadata, toml::de::Error> {
        let theory_raw: AFPTheoryMeta = toml::from_str(toml_content)?;

        // Add dois, and pubs into extra if they exist
        let mut extra_table = theory_raw.extra;
        if !theory_raw.related.dois.is_empty() {
            extra_table.insert("dois".to_string(), theory_raw.related.dois.into());
        }
        if !theory_raw.related.pubs.is_empty() {
            extra_table.insert("pubs".to_string(), theory_raw.related.pubs.into());
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

        Ok(theory)
    }
}
