use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    config::BelleConfig,
    error::{AppError, IoPathErrorContext},
    isabelle::{
        IsabellePathContext,
        error::{IsabelleCommandFailedContext, IsabelleError, IsabelleInvalidOutputContext},
        types::Isabelle,
    },
    util::get_isabelle_version,
};

impl Isabelle {
    pub fn locate(path: impl Into<PathBuf>) -> Result<Self, IsabelleError> {
        let path = path.into();

        let version_res = Self::exec_with_isabelle_from_path(&path, vec!["version"])?;
        let version = get_isabelle_version(&version_res);

        Ok(Self { version, path })
    }

    fn exec_with_isabelle_from_path(isabelle_root: &Path, args: Vec<&str>) -> Result<String, IsabelleError> {
        let isabelle_bin_dir = isabelle_root.join("bin");

        let mut isabelle_cmd = if cfg!(windows) {
            let bash = isabelle_root.join("contrib").join("cygwin").join("bin").join("bash.exe");

            if !bash.is_file() {
                return Err(IsabelleError::NoIsabelle {
                    path: isabelle_root.to_path_buf(),
                })?;
            }

            // Create a command using defaults from `Cygwin-Terminal.bat`
            let mut command = Command::new(bash);
            command
                .env("HOME", env::var("USERPROFILE").unwrap_or_default())
                .env(
                    "PATH",
                    format!(
                        "{};{}",
                        isabelle_bin_dir.display(),
                        env::var("PATH").unwrap_or_default()
                    ),
                )
                .env("LANG", "en_US.UTF-8")
                .env("CHERE_INVOKING", "true")
                .arg("--login")
                .arg("-c")
                // Use the isabelle command, with args given
                .arg("isabelle");

            command
        } else {
            // Execute the isabelle binary directly
            let isabelle_bin = isabelle_bin_dir.join("isabelle");

            if !isabelle_bin.is_file() {
                return Err(IsabelleError::NoIsabelle {
                    path: isabelle_root.to_path_buf(),
                })?;
            }

            Command::new(isabelle_bin)
        };

        // Add the args to the command
        isabelle_cmd.args(&args);

        let res = isabelle_cmd.output().report_failed_isabelle_command(args.iter().copied())?;
        let res_str = String::from_utf8(res.stdout).report_invalid_isabelle_command_output(args.iter().copied())?;

        Ok(res_str)
    }

    fn exec_with_isabelle(&self, args: Vec<&str>) -> Result<String, IsabelleError> {
        Self::exec_with_isabelle_from_path(&self.path, args)
    }

    fn manage_component(&self, add: bool) -> Result<(), AppError> {
        let active_env_dir = BelleConfig::get_active_env_link();
        let formatted_active_env_dir = active_env_dir.to_isabelle_path().report_path(&active_env_dir)?;

        // Add or remove the active environment directory as a component to isabelle
        let flag = if add { "-u" } else { "-x" };
        self.exec_with_isabelle(vec!["components", flag, &formatted_active_env_dir])?;

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
