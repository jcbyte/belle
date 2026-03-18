use reqwest::StatusCode;
use url::Url;

use crate::{
    error::{AppError, ParseErrorContext},
    fetch::{
        BelleClient, PACKAGE_FILE,
        error::{FetchError, FetchErrorContext, FetchUrlContext},
        types::ReturnedPackages,
    },
    registry::{AliasPackage, Package, PackageIdentifier},
};

impl BelleClient {
    pub async fn get_github_package_meta(&self, url: Url, branch: &str) -> Result<ReturnedPackages, AppError> {
        // Ensure this is a github repo
        match url.host_str() {
            Some("github.com") => {}
            Some(host) => {
                return Err(FetchError::RepositoryNotSupported { repo: host.to_string() }.into());
            }
            None => {
                return Err(FetchError::NoRepository.into());
            }
        };

        let mut segments = url.path_segments().expect("URL has a hostname so must have segments");
        let (owner, repo) = match (segments.next(), segments.next()) {
            // Strip ".git" from the name if it exists
            (Some(o), Some(r)) => (o, r.strip_suffix(".git").unwrap_or(r)),
            _ => return Err(FetchError::InvalidRepositoryURL { url }.into()),
        };

        let raw_url = Url::parse(&format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, PACKAGE_FILE
        ))
        .report_invalid_url("repository manifest file")?;
        let zip_url = Url::parse(&format!("https://github.com/{}/{}/zipball/{}", owner, repo, branch))
            .report_invalid_url("repository source archive")?;

        let response = self
            .client
            .get(raw_url.clone())
            .send()
            .await
            .report_fetch("package meta", &raw_url)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(FetchError::NotFound {
                name: "package manifest".to_string(),
                url: raw_url,
            }
            .into());
        }

        let package_content = response
            .text()
            .await
            .report_reading_fetched("package manifest file", &raw_url)?;

        let mut package = toml::from_str::<Package>(&package_content).report_data("package manifest")?;

        package.source = crate::registry::PackageSource::Remote { url: zip_url };

        let aliases: Vec<AliasPackage> = package
            .provides
            .iter()
            .map(|provided| AliasPackage {
                name: provided.clone(),
                version: package.version,
                alias: PackageIdentifier::from(&package),
            })
            .collect();

        Ok(ReturnedPackages { package, aliases })
    }

    pub async fn get_remote_package(&self, url: &Url) -> Result<bytes::Bytes, FetchError> {
        let bytes = self
            .client
            .get(url.clone())
            .send()
            .await
            .report_fetch("package source", url)?
            .bytes()
            .await
            .report_reading_fetched("package source archive", url)?;

        Ok(bytes)
    }
}
