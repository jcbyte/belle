use std::path::PathBuf;

use console::style;
use pubgrub::SemanticVersion;

use crate::{config::BelleConfig, isabelle::Isabelle, util::get_isabelle_name};

pub fn link(path: PathBuf) -> anyhow::Result<()> {
    let isabelle = Isabelle::locate(path)?;

    isabelle.link()?;
    BelleConfig::write_config(|c| c.isabelles.insert(isabelle.version, isabelle.path.clone()));

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

pub fn unlink(version: SemanticVersion) -> anyhow::Result<()> {
    let isabelle = BelleConfig::read_config(|c| {
        c.isabelles.get(&version).map(|path| Isabelle {
            version,
            path: path.clone(),
        })
    })
    .ok_or_else(|| anyhow::anyhow!("No Linked Isabelle matching version {}", version))?;

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
