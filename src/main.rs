use crate::config::BelleConfig;

mod config;
mod fetch;
mod registry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    BelleConfig::init()?;

    fetch::update().await?;

    return Ok(());
}
