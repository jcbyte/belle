use std::sync::OnceLock;

use anyhow::Context;

pub struct BelleClient {
    pub client: reqwest::Client,
}

static CONFIG_INSTANCE: OnceLock<BelleClient> = OnceLock::new();

impl BelleClient {
    /// Create reqwest client to use throughout
    fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            // Include a custom user agent for politeness
            .user_agent("belle-client")
            .build()
            .context("Failed to create reqwest Client")?;

        Ok(Self { client })
    }

    /// Get the reqwest client
    pub fn get() -> anyhow::Result<&'static Self> {
        if let Some(instance) = CONFIG_INSTANCE.get() {
            return Ok(instance);
        }

        let client = Self::new()?;
        let _ = CONFIG_INSTANCE.set(client);

        Ok(CONFIG_INSTANCE.get().expect("Client was just created"))
    }
}
