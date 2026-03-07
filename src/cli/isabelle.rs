use std::path::PathBuf;

use pubgrub::SemanticVersion;

use crate::isabelle::Isabelle;

pub fn link(path: PathBuf) -> anyhow::Result<()> {
    let isabelle = Isabelle::locate(path)?;

    isabelle.link()?;
    // todo add to config

    return Ok(());
}

pub fn unlink(version: SemanticVersion) -> anyhow::Result<()> {
    // todo get from config
    // todo unlink
    // todo remove from config

    return Ok(());
}
