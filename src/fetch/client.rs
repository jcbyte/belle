use std::{sync::OnceLock, time::Duration};

use crate::fetch::error::{FetchError, FetchErrorContext};

pub struct BelleClient {
    pub client: reqwest::Client,
}

static CONFIG_INSTANCE: OnceLock<BelleClient> = OnceLock::new();

impl BelleClient {
    /// Create reqwest client to use throughout
    fn new() -> Result<Self, FetchError> {
        let client = reqwest::Client::builder()
            // Include a custom user agent for politeness
            .user_agent("belle-client")
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .report_failed_init()?;

        Ok(Self { client })
    }

    /// Get the reqwest client
    pub fn get() -> Result<&'static Self, FetchError> {
        if let Some(instance) = CONFIG_INSTANCE.get() {
            return Ok(instance);
        }

        let client = Self::new()?;
        CONFIG_INSTANCE.set(client).ok().expect("Client has already been initialised");
        Ok(CONFIG_INSTANCE.get().expect("Client was just created"))
    }
}
