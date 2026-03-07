use anyhow::Context;

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

// todo should client be globally accessible
