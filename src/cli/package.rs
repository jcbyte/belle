use pubgrub::SemanticVersion;

use crate::{environment::Environment, fetch::BelleClient};

pub async fn add_package(name: String, version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.add_package(name, version.into())?;

    // todo i copy this a lot should it be abstracted
    // todo spinner and nice cli for this
    // todo should client be globally accessible
    active_env.resolve_lock()?;
    let client = BelleClient::new()?;
    active_env.fetch_env_packages(&client).await?;
    active_env.save()?;

    return Ok(());
}

pub async fn remove_package(name: &String) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.remove_package(name)?;

    active_env.resolve_lock()?;
    let client = BelleClient::new()?;
    active_env.fetch_env_packages(&client).await?;
    active_env.save()?;

    return Ok(());
}
