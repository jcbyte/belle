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
}
