use std::{
    env,
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::Context;

use crate::{config::BelleConfig, isabelle::types::Isabelle, util::get_isabelle_version};

impl Isabelle {
    pub fn locate(path: PathBuf) -> anyhow::Result<Self> {
        let res = Self::exec_isabelle_from_path(&path, "isabelle version")?;
        let version = get_isabelle_version(&res);

        return Ok(Self { version, path });
    }

    fn exec_isabelle_from_path(isabelle_root: &PathBuf, cmd: &str) -> anyhow::Result<String> {
        // todo unix-ify a variant
        let bash = isabelle_root.join("contrib").join("cygwin").join("bin").join("bash.exe");
        let isabelle_bin = isabelle_root.join("bin");

        // Create a command using defaults from `Cygwin-Terminal.bat`
        let mut command = Command::new(bash);
        command
            .env("HOME", env::var("USERPROFILE").unwrap_or_default())
            .env(
                "PATH",
                format!(
                    "{};{}",
                    isabelle_bin.to_string_lossy().to_string(),
                    env::var("PATH").unwrap_or_default()
                ),
            )
            .env("LANG", "en_US.UTF-8")
            .env("CHERE_INVOKING", "true")
            .arg("--login")
            .arg("-c")
            .arg(cmd);

        let res = command
            .output()
            .with_context(|| format!("Failed to execute Isabelle command '{}'.", cmd))?;

        let res_str = String::from_utf8(res.stdout)
            .with_context(|| format!("Isabelle command output for '{}' was not valid UTF-8.", cmd))?;

        return Ok(res_str);
    }

    fn exec_isabelle(&self, cmd: &str) -> anyhow::Result<String> {
        return Self::exec_isabelle_from_path(&self.path, cmd);
    }

    pub fn get_isabelle_path(&self, path: PathBuf) -> anyhow::Result<String> {
        // todo unix-ify a variant
        let path = self.exec_isabelle(&format!("cygpath -u {}", path.to_string_lossy().to_string()))?;

        return Ok(path.trim().to_string());
    }

    fn manage_component(&self, add: bool) -> anyhow::Result<()> {
        let active_env_dir = BelleConfig::read_config(|c| c.get_active_env_link());
        let isabelle_path = self.get_isabelle_path(active_env_dir)?;

        // Add or remove the active environment directory as a component to isabelle
        let flag = if add { "-u" } else { "-x" };
        self.exec_isabelle(&format!("isabelle components {} {}", flag, isabelle_path))?;

        return Ok(());
    }

    pub fn link(&self) -> anyhow::Result<()> {
        self.manage_component(true)?;
        return Ok(());
    }

    pub fn unlink(&self) -> anyhow::Result<()> {
        self.manage_component(false)?;
        return Ok(());
    }
}
