use {
    super::ShellInstall,
    crate::{
        cli,
        conf,
        errors::*,
    },
    std::{
        fs,
        path::PathBuf,
    },
};

/// must be incremented when the architecture changes or one of the shell
/// specific scripts is upgraded to a new version
const CURRENT_VERSION: usize = 4;

const REFUSED_FILE_CONTENT: &str = r#"
This file tells broot you refused the installation of the companion shell function.
If you want to install it run
    broot -- install
"#;

const INSTALLED_FILE_CONTENT: &str = r#"
This file tells broot the installation of the br function was done.
If there's a problem and you want to install it again run
    broot -- install
"#;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ShellInstallState {
    NotInstalled, // before any install, this is the initial state
    Refused, // user doesn't want anything to be installed
    Obsolete,
    UpToDate,
}

impl From<cli::CliShellInstallState> for ShellInstallState {
    fn from(cs: cli::CliShellInstallState) -> Self {
        match cs {
            cli::CliShellInstallState::Undefined => Self::NotInstalled,
            cli::CliShellInstallState::Refused => Self::Refused,
            cli::CliShellInstallState::Installed => Self::UpToDate,
        }
    }
}

impl ShellInstallState {
    pub fn get_refused_path() -> PathBuf {
        conf::dir().join("launcher").join("refused")
    }
    pub fn get_installed_path(version: usize) -> PathBuf {
        conf::dir().join("launcher").join(format!("installed-v{version}"))
    }
    pub fn detect() -> Self {
        let current = Self::get_installed_path(CURRENT_VERSION);
        if current.exists() {
            return Self::UpToDate;
        }
        if Self::get_refused_path().exists() {
            return Self::Refused;
        }
        for version in 0..CURRENT_VERSION {
            let installed = Self::get_installed_path(version);
            if installed.exists() {
                return Self::Obsolete;
            }
        }
        Self::NotInstalled
    }
    pub fn remove(si: &ShellInstall) -> Result<(), ShellInstallError> {
        si.remove(&Self::get_refused_path())?;
        for version in 0..=CURRENT_VERSION {
            let installed = Self::get_installed_path(version);
            si.remove(&installed)?;
        }
        Ok(())
    }
    /// write either the "installed" or the "refused" file, or remove
    ///  those files.
    ///
    /// This is useful in installation
    /// or test scripts when we don't want the user to be prompted
    /// to install the function, or in case something doesn't properly
    /// work in shell detections
    pub fn write(self, si: &ShellInstall) -> Result<(), ShellInstallError> {
        Self::remove(si)?;
        match self {
            ShellInstallState::Refused => {
                let refused_path = Self::get_refused_path();
                fs::create_dir_all(refused_path.parent().unwrap())
                    .context(&|| format!("creating parents of {refused_path:?}"))?;
                fs::write(&refused_path, REFUSED_FILE_CONTENT)
                    .context(&|| format!("writing in {refused_path:?}"))?;
            }
            ShellInstallState::UpToDate => {
                let installed_path = Self::get_installed_path(CURRENT_VERSION);
                fs::create_dir_all(installed_path.parent().unwrap())
                    .context(&|| format!("creating parents of {installed_path:?}"))?;
                fs::write(&installed_path, INSTALLED_FILE_CONTENT)
                    .context(&|| format!("writing in {installed_path:?}"))?;
            }
            _ => {
                warn!("not writing state {self:?}");
            }
        }
        Ok(())
    }
}

