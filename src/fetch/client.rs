use anyhow::Context;

pub struct BelleClient {
    client: reqwest::Client,
}

impl BelleClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("belle-client")
            .build()
            .context("Failed to create reqwest Client")?;

        return Ok(Self { client });
    }
}
