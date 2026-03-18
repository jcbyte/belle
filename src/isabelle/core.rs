use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    config::BelleConfig,
    error::{AppError, IoPathErrorContext},
    isabelle::{
        error::{IsabelleCommandFailedContext, IsabelleError, IsabelleInvalidOutputContext},
        types::Isabelle,
    },
    util::get_isabelle_version,
};

impl Isabelle {
    pub fn locate(path: impl Into<PathBuf>) -> Result<Self, IsabelleError> {
        let path = path.into();

        let version_res = Self::exec_with_isabelle_from_path(&path, "isabelle version")?;
        let version = get_isabelle_version(&version_res);

        Ok(Self { version, path })
    }

    pub fn exec_with_isabelle_from_path(isabelle_root: &Path, cmd: &str) -> Result<String, IsabelleError> {
        let isabelle_bin = isabelle_root.join("bin");

        let mut command = if cfg!(windows) {
            let bash = isabelle_root.join("contrib").join("cygwin").join("bin").join("bash.exe");

            // Create a command using defaults from `Cygwin-Terminal.bat`
            let mut command = Command::new(bash);
            command
                .env("HOME", env::var("USERPROFILE").unwrap_or_default())
                .env(
                    "PATH",
                    format!("{};{}", isabelle_bin.display(), env::var("PATH").unwrap_or_default()),
                )
                .env("LANG", "en_US.UTF-8")
                .env("CHERE_INVOKING", "true")
                .arg("--login")
                .arg("-c")
                .arg(cmd);

            command
        } else {
            // Create a command using the shell, with access to the Isabelle executable
            let mut command = Command::new("sh");
            command
                .env(
                    "PATH",
                    format!("{}:{}", isabelle_bin.display(), env::var("PATH").unwrap_or_default()),
                )
                .arg("-c")
                .arg(cmd);

            command
        };

        let res = command.output().report_failed_command(cmd)?;
        let res_str = String::from_utf8(res.stdout).report_invalid_output(cmd)?;

        Ok(res_str)
    }

    fn exec_with_isabelle(&self, cmd: &str) -> Result<String, IsabelleError> {
        Self::exec_with_isabelle_from_path(&self.path, cmd)
    }

    pub fn get_isabelle_path(&self, path: &Path) -> Result<String, AppError> {
        if cfg!(windows) {
            let path_res = self.exec_with_isabelle(&format!("cygpath -u \"{}\"", path.display()))?;
            Ok(path_res.trim().to_string())
        } else {
            let path_str = path.to_str().report_path(&path)?;
            Ok(path_str.trim().to_string())
        }
    }

    fn manage_component(&self, add: bool) -> Result<(), AppError> {
        let active_env_dir = BelleConfig::read_config(|c| c.get_active_env_link());
        let isabelle_path = self.get_isabelle_path(&active_env_dir)?;

        // Add or remove the active environment directory as a component to isabelle
        let flag = if add { "-u" } else { "-x" };
        self.exec_with_isabelle(&format!("isabelle components {} {}", flag, isabelle_path))?;

        Ok(())
    }

    pub fn link(&self) -> Result<(), AppError> {
        self.manage_component(true)?;
        Ok(())
    }

    pub fn unlink(&self) -> Result<(), AppError> {
        self.manage_component(false)?;
        Ok(())
    }
}
