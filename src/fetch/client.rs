use anyhow::{Context, bail};

use crate::registry::{Package, PackageSource};

pub struct BelleClient {
    pub client: reqwest::Client,
}

impl BelleClient {
    /// Create reqwest client to use throughout
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            // Include a custom user agent for politeness
            .user_agent("belle-client")
            .build()
            .context("Failed to create reqwest Client")?;

        return Ok(Self { client });
    }

    pub async fn get_package(&self, package: &Package) -> anyhow::Result<Option<bytes::Bytes>> {
        // Route the package fetching to its respective source
        let bytes = match &package.source {
            PackageSource::Afp(repo) => self.get_afp_package(&package.name, repo).await?,
            PackageSource::Remote { url } => self.get_remote_package(url.clone()).await?,
            // Do not need to retrieve local packages as they can be used from there respective folders
            PackageSource::Local { .. } => return Ok(None),
            PackageSource::Default => bail!("Source is not given for this package"),
        };

        return Ok(Some(bytes));
    }
}
