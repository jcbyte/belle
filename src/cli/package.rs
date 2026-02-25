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

pub fn list_packages(all: bool) -> anyhow::Result<()> {
    let active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    let packages = if all {
        active_env.get_all_packages()
    } else {
        active_env.get_packages()
    }?;

    for package in packages {
        let name = style(package.name);
        let styled_name = if !package.transitive {
            name.magenta()
        } else {
            name.dim()
        };

        let version = style(package.version.to_string());
        let styled_version = if package.given_version {
            version.green()
        } else {
            version.dim()
        };

        println!(
            "- {} {}{}{}",
            styled_name,
            style("[").dim(),
            styled_version,
            style("]").dim()
        )
    }

    return Ok(());
}
