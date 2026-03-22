use std::path::Path;

use console::style;
use pubgrub::SemanticVersion;

use crate::{
    cli::core::{CliLine, DisplayVersion},
    config::BelleConfig,
    environment::{self, Environment},
    error::AppError,
    isabelle::{Isabelle, error::IsabelleVersionLinkedContext},
    util::get_isabelle_name,
};

pub fn link(path: &Path) -> Result<(), AppError> {
    // If there is not an active environment then switch to the null environment
    // So that isabelle can register correctly to the env/active symlink
    if !Environment::has_active() {
        environment::manager::set_env_none()?;
    }

    let isabelle = Isabelle::locate(path)?;

    isabelle.link()?;
    BelleConfig::write_config(|c| c.isabelles.insert(isabelle.version, isabelle.path));

    CliLine::new()
        .prefix("Linked")
        .line(format!(
            "Isabelle {} {} {}{}{}",
            style(get_isabelle_name(&isabelle.version)).cyan().bright(),
            DisplayVersion::Implicit(&isabelle.version),
            style("(").dim(),
            style(path.display()).dim(),
            style(")").dim()
        ))
        .as_success()
        .print();

    Ok(())
}

pub fn unlink(version: SemanticVersion) -> Result<(), AppError> {
    let isabelle = BelleConfig::read_config(|c| {
        c.isabelles.get(&version).map(|path| Isabelle {
            version,
            path: path.clone(),
        })
    })
    .report_not_linked(version)?;

    isabelle.unlink()?;
    BelleConfig::write_config(|c| c.isabelles.remove(&version));

    CliLine::new()
        .prefix("Unlinked")
        .line(format!(
            "Isabelle {} {}",
            style(get_isabelle_name(&isabelle.version)).cyan().bright(),
            DisplayVersion::Implicit(&isabelle.version),
        ))
        .as_success()
        .print();

    Ok(())
}
