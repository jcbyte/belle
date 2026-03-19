use std::path::PathBuf;

use console::style;
use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    error::AppError,
    isabelle::{Isabelle, error::IsabelleError},
    util::get_isabelle_name,
};

pub fn link(path: PathBuf) -> Result<(), AppError> {
    let isabelle = Isabelle::locate(path)?;

    isabelle.link()?;
    BelleConfig::write_config(|c| c.isabelles.insert(isabelle.version, isabelle.path));

    println!(
        "Linked {} {} {}{}{}",
        style("Isabelle").cyan(),
        style(get_isabelle_name(&isabelle.version)).cyan().bold(),
        style("[").dim(),
        style(isabelle.version).green(),
        style("]").dim()
    );

    Ok(())
}

pub fn unlink(version: SemanticVersion) -> Result<(), AppError> {
    let isabelle = BelleConfig::read_config(|c| {
        c.isabelles.get(&version).map(|path| Isabelle {
            version,
            path: path.clone(),
        })
    })
    .ok_or(IsabelleError::VersionNotLinked { version })?;

    isabelle.unlink()?;
    BelleConfig::write_config(|c| c.isabelles.remove(&version));

    println!(
        "Unlinked {} {} {}{}{}",
        style("Isabelle").cyan(),
        style(get_isabelle_name(&isabelle.version)).cyan().bold(),
        style("[").dim(),
        style(isabelle.version).green(),
        style("]").dim()
    );

    Ok(())
}
