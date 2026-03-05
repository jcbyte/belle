use anyhow::{Context, bail};
use reqwest::StatusCode;
use url::Url;

use crate::{
    fetch::{BelleClient, PACKAGE_FILE},
    registry::{AliasPackage, Package, PackageIdentifier},
};

impl BelleClient {
    pub async fn get_github_package_meta(
        &self,
        url: Url,
        branch: &str,
    ) -> anyhow::Result<(Package, Vec<AliasPackage>)> {
        // Ensure this is a github repo
        if url.host_str() != Some("github.com") {
            return Err(anyhow::anyhow!("Only github repositories are currently supported"));
        }

        let mut segments = url.path_segments().ok_or(anyhow::anyhow!(""))?;
        let (owner, repo) = match (segments.next(), segments.next()) {
            // Strip ".git" from the name if it exists
            (Some(o), Some(r)) => (o, r.strip_suffix(".git").unwrap_or(r)),
            _ => return Err(anyhow::anyhow!("Invalid GitHub Repo URL")),
        };

        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, PACKAGE_FILE
        );
        let zip_url = Url::parse(&format!("https://github.com/{}/{}/zipball/{}", owner, repo, branch))
            .context("Failed to construct remote archive URL")?;

        let response = self
            .client
            .get(raw_url)
            .send()
            .await
            .context("Failed to send request to GitHub")?;

        if response.status() == StatusCode::NOT_FOUND {
            bail!("Package manifest or repository could not be found");
        }

        let package_content = response.text().await.context("Failed to parse response from GitHub")?;

        let mut package =
            toml::from_str::<Package>(&package_content).context("Failed to parse TOML for package manifest")?;

        package.source = crate::registry::PackageSource::Remote { url: zip_url };

        let aliases: Vec<AliasPackage> = package
            .provides
            .iter()
            .map(|provided| AliasPackage {
                name: provided.clone(),
                version: package.version.clone(),
                alias: PackageIdentifier::from(&package),
            })
            .collect();

        return Ok((package, aliases));
    }

    pub async fn get_remote_package(&self, url: Url) -> anyhow::Result<bytes::Bytes> {
        let bytes = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request to fetch package")?
            .bytes()
            .await
            .context("Failed to read archive bytes")?;

        return Ok(bytes);
    }
}
