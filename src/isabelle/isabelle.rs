use std::{
    env,
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::Context;

use crate::{isabelle::types::Isabelle, util::get_isabelle_version};

impl Isabelle {
    pub fn locate(path: PathBuf) -> anyhow::Result<Self> {
        let res = Self::exec_isabelle(&path, "version")?;
        let res_str = String::from_utf8(res.stdout).context("Isabelle command output was not valid UTF-8")?;
        let version = get_isabelle_version(&res_str);

        return Ok(Self { version, path });
    }

    fn exec_isabelle(isabelle_root: &PathBuf, cmd: &str) -> anyhow::Result<Output> {
        // todo unix-ify a variant
        let bash = isabelle_root.join("contrib").join("cygwin").join("bin").join("bash.exe");
        let isabelle_bin = isabelle_root.join("bin");

        let isabelle_cmd = format!("isabelle {}", cmd);

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
            .arg(&isabelle_cmd);

        let res = command
            .output()
            .with_context(|| format!("Failed to execute Isabelle command {}", isabelle_cmd))?;

        return Ok(res);
    }
}
