mod install;

use crate::sdk::conda::install::install_conda;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    str::Utf8Error,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Can't access to configs")]
    ConfigError(#[from] crate::config::Error),

    #[error("Cannot download provided url")]
    DownloadError(#[from] crate::utils::downloader::Error),

    #[error("Cannot create the conda directory")]
    CreateCondaDirectory(std::io::Error),

    #[error("Cannot create a download directory")]
    PathError,

    #[error("Conda isn't installed")]
    NotInstalledError,

    #[error("Conda is already installed")]
    CondaAlreadyInstalled,

    #[error("Conda install error")]
    InstallError(std::io::Error),

    #[error("Can't invoke command")]
    CommandInvokationError(std::io::Error),

    #[error("Command error")]
    CommandError(Output),

    #[error("Cannot read the command output")]
    CommandOutputParsingError(#[from] Utf8Error),
}

pub struct Conda {
    conda_path: PathBuf,
}

impl From<PathBuf> for Conda {
    fn from(path: PathBuf) -> Self {
        Self { conda_path: path }
    }
}

impl Conda {
    fn get_executable_name(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "_conda"
        } else {
            "bin/conda"
        }
    }

    fn command(&self, args: Vec<&str>) -> Result<String, Error> {
        if !self.is_installed()? {
            return Err(Error::NotInstalledError);
        }

        let executable_path = self.conda_path.join(self.get_executable_name());
        let mut command = Command::new(executable_path);

        let output = command
            .args(args)
            .output()
            .map_err(Error::CommandInvokationError)?;

        if !output.status.success() {
            return Err(Error::CommandError(output));
        }

        let stdout = std::str::from_utf8(&output.stdout)?.to_string();
        Ok(stdout)
    }

    pub fn is_installed(&self) -> Result<bool, Error> {
        let path = if cfg!(target_os = "windows") {
            "_conda.exe"
        } else {
            "bin/conda"
        };

        match fs::metadata(self.conda_path.join(path)) {
            | Ok(metadata) => Ok(metadata.is_file()),
            | Err(_) => Ok(false),
        }
    }

    pub fn version(&self) -> Result<String, Error> {
        let mut out = self.command(vec!["--version"])?;
        out = out.trim().to_string();
        out = out.replace("conda ", "");
        Ok(out)
    }

    pub fn create_env(
        &self, name: &str, python_version: &str,
    ) -> Result<(), Error> {
        self.command(vec![
            "create",
            "-p",
            self.conda_path
                .join("envs")
                .join(name)
                .to_str()
                .ok_or(Error::PathError)?,
            &format!("python={}", python_version),
        ])
        .and(Ok(()))
    }

    pub async fn install(&self) -> Result<(), Error> {
        if self.is_installed()? {
            Err(Error::CondaAlreadyInstalled)
        } else {
            install_conda(&self.conda_path).await
        }
    }
}
