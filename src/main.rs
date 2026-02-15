use std::{
    collections::HashMap,
    fmt::format,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::Context;
use regex::Regex;
use reqwest::header::Entry;
use serde::Deserialize;
use zip::ZipArchive;

#[derive(Deserialize)]
struct AFPRepo {
    id: i32,
    name: String,
}

async fn get_afp_repos() -> anyhow::Result<Vec<AFPRepo>> {
    let afp_repo_list_url =
        "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?order_by=last_activity_at&sort=desc";

    let client = reqwest::Client::new();
    let repos: Vec<AFPRepo> = client
        .get(afp_repo_list_url)
        .header("User-Agent", "belle-client")
        .send()
        .await
        .context("Failed to send request to Hetapod")?
        .json()
        .await
        .context("Failed to parse JSON response from Hetapod")?;

    let re = Regex::new(r"^afp-[\d-]+$").context("Invalid regex pattern for AFP repository name")?;
    let afp_repos: Vec<AFPRepo> = repos.into_iter().filter(|p| re.is_match(&p.name)).collect();

    return Ok(afp_repos);
}

#[derive(Deserialize)]
struct TreeItem {
    name: String,
    path: String,
}
// let releases_file_url = format!(
//     "https://foss.heptapod.net/api/v4/projects/{}/repository/files/metadata%2Freleases.toml/raw",
//     repo.id
// );

async fn get_thys(repo: &AFPRepo) -> anyhow::Result<Vec<String>> {
    let repo_entries_tree_url = format!(
        // ! Note this hard coded `2000` per page
        "https://foss.heptapod.net/api/v4/projects/{}/repository/tree?path=metadata%2Fentries&per_page=2000",
        repo.id
    );

    let client = reqwest::Client::new();
    let entries_tree: Vec<TreeItem> = client
        .get(repo_entries_tree_url)
        .header("User-Agent", "belle-client")
        .send()
        .await
        .with_context(|| format!("Failed to fetch entires list for '{}' repo", repo.name))?
        .json()
        .await
        .with_context(|| format!("Failed to parse JSON response of entires list for '{}' repo", repo.name))?;

    let thys: Vec<String> = entries_tree
        .iter()
        .filter_map(|e| Path::new(&e.name).file_stem().map(|f| f.to_string_lossy().to_string()))
        .collect();
    Ok(thys)
}

#[derive(Debug)]
struct AuthorMeta {
    name: String,
    orcid: Option<String>,
    emails: Vec<String>,
    homepages: Vec<String>,
}

#[derive(Debug)]
pub struct RelatedMeta {
    pub dois: Vec<String>,
    pub pubs: Vec<String>,
}

#[derive(Debug)]
struct EntryMeta {
    title: String,
    date: String,
    r#abstract: String,
    license: String,
    topics: Vec<String>,
    note: Option<String>,
    authors: Vec<String>,
    contributors: Vec<String>,
    related: RelatedMeta,
    extra: HashMap<String, String>,
}

#[derive(Debug)]
struct RepoMeta {
    authors: HashMap<String, AuthorMeta>,
    licences: HashMap<String, String>,
    // releases: HashMap<String, Vec<(String, String)>>, // ? This is not used/needed
    // topics:, // ? This is not used/needed
    entries: HashMap<String, EntryMeta>,
}

async fn get_repo_meta(repo: &AFPRepo) -> anyhow::Result<RepoMeta> {
    let meta_archive_url = format!(
        "https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata",
        repo.id
    );

    let client = reqwest::Client::builder()
        .user_agent("belle-client")
        .build()
        .context("Failed to create reqwest Client")?;

    let response_bytes = client
        .get(meta_archive_url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch metadata for '{}' repo", repo.name))?
        .bytes()
        .await
        .with_context(|| format!("Failed to read metadata archive bytes for '{}' repo", repo.name))?;

    let reader = Cursor::new(response_bytes);
    let mut archive = ZipArchive::new(reader).context("Failed to read zip archive")?;

    let mut authors: HashMap<String, AuthorMeta> = HashMap::default();
    let mut licences: HashMap<String, String> = HashMap::default();
    let mut entries: HashMap<String, EntryMeta> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(name) = file.enclosed_name() else { continue };

        if name.ends_with("authors.toml") {
            #[derive(serde::Deserialize, Debug)]
            pub struct Email {
                user: Vec<String>,
                host: Vec<String>,
            }
            impl Email {
                pub fn to_string(&self) -> String {
                    format!("{}@{}", self.user.join("."), self.host.join("."))
                }
            }
            #[derive(serde::Deserialize, Debug)]
            pub struct Author {
                name: String,
                orcid: Option<String>,

                // Default to empty map if the section is missing in TOML
                #[serde(default)]
                emails: HashMap<String, Email>,

                #[serde(default)]
                homepages: HashMap<String, String>,
            }

            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let authors_toml: HashMap<String, Author> = toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML for 'authors.toml' in {} repo", repo.name))?;

            authors = authors_toml
                .into_iter()
                .map(|(author_id, author)| {
                    (
                        author_id,
                        AuthorMeta {
                            name: author.name,
                            orcid: author.orcid,
                            emails: author.emails.values().map(|email| email.to_string()).collect(),
                            homepages: author.homepages.into_values().collect(),
                        },
                    )
                })
                .collect();
        } else if name.ends_with("licenses.toml") {
            #[derive(serde::Deserialize)]
            struct Licence {
                name: String,
            }

            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let licence_toml: HashMap<String, Licence> = toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML for 'licenses.toml' in {} repo", repo.name))?;

            licences = licence_toml
                .into_iter()
                .map(|(licence_id, licence)| (licence_id, licence.name))
                .collect();
        } else if name.parent().map_or(false, |p| p.ends_with("entries")) {
            #[derive(Debug, Deserialize, Default)]
            pub struct Related {
                #[serde(default)]
                pub dois: Vec<String>,
                #[serde(default)]
                pub pubs: Vec<String>,
            }
            #[derive(Debug, Deserialize)]
            pub struct Entry {
                pub title: String,
                pub date: toml::value::Datetime,
                pub topics: Vec<String>,
                pub r#abstract: String,
                pub license: String,
                pub note: Option<String>,

                // Tables with dynamic keys
                #[serde(default)]
                pub authors: HashMap<String, toml::Value>,
                #[serde(default)]
                pub contributors: HashMap<String, toml::Value>,
                #[serde(default)]
                pub extra: HashMap<String, String>,
                #[serde(default)]
                pub related: Related,
            }

            let Some(thy_name) = name.file_stem().map(|tn| tn.to_string_lossy().to_string()) else {
                continue;
            };

            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let entry_toml: Entry = toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML for entry '{}' in {} repo", thy_name, repo.name))?;

            entries.insert(
                thy_name,
                EntryMeta {
                    title: entry_toml.title,
                    date: entry_toml.date.to_string(),
                    r#abstract: entry_toml.r#abstract,
                    license: entry_toml.license,
                    topics: entry_toml.topics,
                    note: entry_toml.note,
                    authors: entry_toml.authors.into_keys().collect(),
                    contributors: entry_toml.contributors.into_keys().collect(),
                    extra: entry_toml.extra,
                    related: RelatedMeta {
                        dois: entry_toml.related.dois,
                        pubs: entry_toml.related.pubs,
                    },
                },
            );
        }
    }

    let meta = RepoMeta {
        authors,
        licences,
        entries,
    };

    return Ok(meta);
    // https://foss.heptapod.net/api/v4/projects/{}/repository/archive.zip?path=metadata/entries
}

// todo get dependencies through ROOT files (this will be more difficult)

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let afp_repos = get_afp_repos().await?;
    // let latest_repo = afp_repos.first().context("No latest AFP repository could be found")?;
    let latest_repo = &AFPRepo {
        id: 2228,
        name: String::from("afp-2025-2"),
    };
    println!("name: {} {}", latest_repo.name, latest_repo.id);

    let a = get_repo_meta(latest_repo).await?;
    // let a = get_thys(latest_repo).await?;
    println!("{:#?}", a);

    return Ok(());
}
