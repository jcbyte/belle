use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    config::BelleConfig,
    error::AppError,
    isabelle::{
        error::{IsabelleCommandFailedContext, IsabelleError, IsabelleInvalidOutputContext},
        types::Isabelle,
    },
    util::get_isabelle_version,
};

impl Isabelle {
    pub fn locate(path: PathBuf) -> Result<Self, IsabelleError> {
        let res = Self::exec_with_isabelle_from_path(&path, "isabelle version")?;
        let version = get_isabelle_version(&res);

        Ok(Self { version, path })
    }

    pub fn exec_with_isabelle_from_path(isabelle_root: &Path, cmd: &str) -> Result<String, IsabelleError> {
        let isabelle_bin = isabelle_root.join("bin");

        let mut command = {
            #[cfg(windows)]
            {
                let bash = isabelle_root.join("contrib").join("cygwin").join("bin").join("bash.exe");

                // Create a command using defaults from `Cygwin-Terminal.bat`
                let mut command = Command::new(bash);
                command
                    .env("HOME", env::var("USERPROFILE").unwrap_or_default())
                    .env(
                        "PATH",
                        format!(
                            "{};{}",
                            isabelle_bin.to_string_lossy(),
                            env::var("PATH").unwrap_or_default()
                        ),
                    )
                    .env("LANG", "en_US.UTF-8")
                    .env("CHERE_INVOKING", "true")
                    .arg("--login")
                    .arg("-c")
                    .arg(cmd);

                command
            }
            #[cfg(unix)]
            {
                // Create a command using the shell, with access to the Isabelle executable
                let mut command = Command::new("sh");
                command
                    .env(
                        "PATH",
                        format!(
                            "{}:{}",
                            isabelle_bin.to_string_lossy(),
                            env::var("PATH").unwrap_or_default()
                        ),
                    )
                    .arg("-c")
                    .arg(cmd);

                command
            }
        };

        let res = command.output().report_failed_command(cmd)?;
        let res_str = String::from_utf8(res.stdout).report_invalid_output(cmd)?;

        Ok(res_str)
    }

    fn exec_with_isabelle(&self, cmd: &str) -> Result<String, IsabelleError> {
        Self::exec_with_isabelle_from_path(&self.path, cmd)
    }

    pub fn get_isabelle_path(&self, path: PathBuf) -> Result<String, AppError> {
        #[cfg(windows)]
        let path = self.exec_with_isabelle(&format!("cygpath -u \"{}\"", path.display()))?;
        #[cfg(unix)]
        let path = path.to_str().report_path(&path)?.to_string();

        Ok(path.trim().to_string())
    }

    fn manage_component(&self, add: bool) -> Result<(), AppError> {
        let active_env_dir = BelleConfig::read_config(|c| c.get_active_env_link());
        let isabelle_path = self.get_isabelle_path(active_env_dir)?;

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
