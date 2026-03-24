use std::path::Path;

use console::style;
use hinted::{Hinted, HintedResultExt};
use indicatif::ProgressBar;
use pubgrub::SemanticVersion;

use crate::{
    cli::core::{CliLine, DisplayVersion, ProgressBarTheme},
    config::BelleConfig,
    environment::{self, Environment},
    error::AppError,
    isabelle::{Isabelle, error::IsabelleError},
    util::get_isabelle_name,
};

pub fn link(path: &Path, force: bool) -> Result<(), AppError> {
    // If there is not an active environment then switch to the null environment
    // So that isabelle can register correctly to the env/active symlink
    if !Environment::has_active() {
        environment::manager::set_env_none()?;
    };

    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.set_belle_prefix("Linking");
    pb.set_message(format!("Isabelle at {}", path.display()));

    let isabelle = Isabelle::locate(path)?;

    // If force is set we do not error here, and we will overwrite
    if BelleConfig::read_config(|c| c.isabelles.contains_key(&isabelle.version)) && !force {
        return Err(IsabelleError::AlreadyLinked {
            version: isabelle.version,
        }
        .into());
    }

    isabelle.link()?;
    BelleConfig::write_config(|c| c.isabelles.insert(isabelle.version, isabelle.path));

    pb.finish_and_clear();
    CliLine::new()
        .prefix("Linked")
        .line(format!(
            "Isabelle {} {} {}{}{}",
            get_isabelle_name(&isabelle.version),
            DisplayVersion::Implicit(&isabelle.version),
            style("(").dim(),
            style(path.display()).dim(),
            style(")").dim()
        ))
        .with_success()
        .print();

    Ok(())
}

pub fn unlink(version: SemanticVersion, force: bool) -> Result<(), Hinted<AppError>> {
    let Some(isabelle) = BelleConfig::read_config(|c| {
        c.isabelles.get(&version).map(|path| Isabelle {
            version,
            path: path.clone(),
        })
    }) else {
        CliLine::new()
            .line(format!(
                "Isabelle {} {} is not linked; nothing to unlink",
                get_isabelle_name(&version),
                DisplayVersion::Implicit(&version)
            ))
            .with_skipped()
            .print();
        CliLine::new()
            .line("use `belle isabelle list` to see linked versions of Isabelle")
            .with_note()
            .print();
        return Ok(());
    };

    let pb = ProgressBar::new_spinner().with_belle_spinner_style();
    pb.set_belle_prefix("Unlinking");
    pb.set_message(format!(
        "Isabelle {} {}",
        get_isabelle_name(&version),
        DisplayVersion::Implicit(&version)
    ));

    let link_res = isabelle.unlink();
    if !force {
        link_res.hint("use `belle unlink <version> --force` to force removal")?;
    }

    BelleConfig::write_config(|c| c.isabelles.remove(&version));

    pb.finish_and_clear();
    CliLine::new()
        .prefix("Unlinked")
        .line(format!(
            "Isabelle {} {}",
            get_isabelle_name(&isabelle.version),
            DisplayVersion::Implicit(&isabelle.version),
        ))
        .with_success()
        .print();

    Ok(())
}

pub fn list() -> Result<(), AppError> {
    let active_env: Option<SemanticVersion> = Environment::active()?.and_then(|env| env.get_isabelle_version().into());

    BelleConfig::read_config(|c| {
        for (version, path) in &c.isabelles {
            let line = format!(
                "Isabelle {:<8} {:<12} {}",
                get_isabelle_name(version),
                DisplayVersion::Implicit(version),
                style(path.display()).dim(),
            );
            if active_env == Some(*version) {
                CliLine::new().prefix("Environment").line(line).with_focus().print();
            } else {
                CliLine::new().line(line).print();
            }
        }

        CliLine::new()
            .prefix("Listed")
            .line(format!("{} linked Isabelle versions", style(c.isabelles.len()).bold()))
            .with_success()
            .print();
    });

    Ok(())
}
