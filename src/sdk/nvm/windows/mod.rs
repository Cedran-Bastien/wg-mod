use crate::sdk::node::windows::WindowsNode;
use crate::sdk::node::Node;
use crate::sdk::nvm::windows::install::install_nvm_windows;
use crate::sdk::nvm::NVM;
use crate::sdk::{nvm, InstallResult, Installable};
use crate::utils::command::command;
use crate::utils::convert_pathbuf_to_string::Stringify;
use crate::utils::Env;
use std::path::PathBuf;
use std::process::Output;

mod install;

pub struct WindowsNVM {
    nvm_path: PathBuf,
}

impl WindowsNVM {
    fn get_executable_path(&self) -> PathBuf {
        self.nvm_path.join("nvm.exe")
    }
}

impl From<&PathBuf> for WindowsNVM {
    fn from(nvm_path: &PathBuf) -> Self {
        Self {
            nvm_path: nvm_path.clone(),
        }
    }
}

impl Installable for WindowsNVM {
    fn is_installed(&self) -> bool {
        self.nvm_path.exists()
    }

    fn install(&self) -> InstallResult {
        if self.is_installed() {
            Err("NVM already installed".into())
        } else {
            install_nvm_windows(&self.nvm_path).map_err(|err| err.to_string())
        }
    }
}

impl NVM for WindowsNVM {
    fn install_node(&self) -> nvm::Result<()> {
        println!("Installing Node via nvm...");

        let args = vec!["install", "latest"];
        self.exec(args)
            .map_err(|e| nvm::Error::InstallError(e.to_string()))?;

        self.nvm_use("latest")?;

        Ok(())
    }

    fn exec(&self, args: Vec<&str>) -> nvm::Result<Output> {
        let executable = self.get_executable_path();
        let executable_str = executable.as_os_str();

        let env = vec![Env {
            key: "NVM_HOME".to_string(),
            value: self.nvm_path.to_string()?,
        }];

        command(executable_str, args, env).map_err(|_| nvm::Error::ExecError)
    }

    fn get_node(&self) -> nvm::Result<Box<dyn Node>> {
        let mut version = self.current_node_version()?;
        let mut node_path = self.nvm_path.join(version);

        if !node_path.exists() {
            self.install_node()?;
        }

        version = self.current_node_version()?;
        node_path = self.nvm_path.join(version);

        Ok(Box::new(WindowsNode::from(node_path)))
    }

    fn current_node_version(&self) -> nvm::Result<String> {
        let out = self.exec(vec!["list", "installed"])?;
        if !out.status.success() {
            return Err(nvm::Error::ExecError);
        }

        let stdout = String::from_utf8(out.stdout)
            .map_err(|_| nvm::Error::ExecCurrentError)?
            .trim()
            .to_string()
            .replace("* ", "")
            .replace(" (Currently using 64-bit executable)", "");

        Ok(format!("v{}", stdout))
    }
}
