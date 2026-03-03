use console::style;
use pubgrub::SemanticVersion;

use crate::environment::Environment;

pub fn add_package(name: String, version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.add_package(name, version)?;

    return Ok(());
}

pub fn remove_package(name: &String) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.remove_package(name)?;
    return Ok(());
}
