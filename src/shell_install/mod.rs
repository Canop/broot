use {
    crate::{cli, conf, errors::ProgramError, skin},
    std::{
        fs, io, os,
        path::{Path, PathBuf},
        str::FromStr,
    },
    termimad::{mad_print_inline, MadSkin},
};

mod bash;
mod fish;
mod util;

const MD_INSTALL_REQUEST: &str = r#"
**Broot** should be launched using a shell function.
This function most notably makes it possible to `cd` from inside broot
(see *https://dystroy.org/broot/install* for explanations).

Can I install it now? [**Y**/n]
"#;

const MD_INSTALL_CANCELLED: &str = r#"
You refused the installation (for now).
You can still used `broot` but some features won't be available.
If you want the `br` shell function, you may either
* do `broot --install`
* install the various pieces yourself
(see *https://dystroy.org/broot/install* for details).

"#;

const MD_INSTALL_DONE: &str = r#"
The **br** function has been successfully installed.
You may have to restart your shell or source your shell init files.
Afterwards, you should start broot with `br` in order to use its full power.

"#;

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

pub struct ShellInstall {
    force_install: bool, // when the program was launched with --install
    skin: MadSkin,
    pub should_quit: bool,
    authorization: Option<bool>,
    done: bool, // true if the installation was just made
}

#[derive(Debug, Clone, Copy)]
pub enum ShellInstallState {
    Undefined, // before any install, this is the initial state
    Refused,
    Installed,
}
impl FromStr for ShellInstallState {
    type Err = ProgramError;
    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "undefined" => Ok(Self::Undefined),
            "refused" => Ok(Self::Refused),
            "installed" => Ok(Self::Installed),
            _ => Err(ProgramError::InternalError {
                // not supposed to happen because claps check the values
                details: format!("unexpected install state: {:?}", state),
            }),
        }
    }
}
impl ShellInstallState {
    /// write either the "installed" or the "refused" file, or remove
    ///  those files.
    ///
    /// This is useful in installation
    /// or test scripts when we don't want the user to be prompted
    /// to install the function, or in case something doesn't properly
    /// work in shell detections
    pub fn write_file(self) -> Result<(), ProgramError> {
        let refused_path = get_refused_path();
        let installed_path = get_installed_path();
        if installed_path.exists() {
            fs::remove_file(&installed_path)?;
        }
        if refused_path.exists() {
            fs::remove_file(&refused_path)?;
        }
        match self {
            Self::Refused => {
                fs::create_dir_all(refused_path.parent().unwrap())?;
                fs::write(&refused_path, REFUSED_FILE_CONTENT)?;
            }
            Self::Installed => {
                fs::create_dir_all(installed_path.parent().unwrap())?;
                fs::write(&installed_path, INSTALLED_FILE_CONTENT)?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn get_refused_path() -> PathBuf {
    conf::dir().join("launcher").join("refused")
}

fn get_installed_path() -> PathBuf {
    conf::dir().join("launcher").join("installed-v1")
}

impl ShellInstall {
    pub fn new(force_install: bool) -> Self {
        Self {
            force_install,
            skin: skin::make_cli_mad_skin(),
            should_quit: false,
            authorization: if force_install { Some(true) } else { None },
            done: false,
        }
    }

    /// write on stdout the script building the function for
    /// the given shell
    pub fn print(shell: &str) -> Result<(), ProgramError> {
        match shell {
            "bash" | "zsh" => println!("{}", bash::get_script()),
            "fish" => println!("{}", fish::get_script()),
            _ => {
                return Err(ProgramError::UnknowShell {
                    shell: shell.to_string(),
                });
            }
        }
        Ok(())
    }

    /// check whether the shell function is installed, install
    /// it if it wasn't refused before or if broot is launched
    /// with --install.
    pub fn check(&mut self) -> Result<(), ProgramError> {
        let installed_path = get_installed_path();
        if self.force_install {
            self.skin.print_text("You requested a clean (re)install.");
            self.remove(&get_refused_path())?;
            self.remove(&installed_path)?;
        } else {
            if installed_path.exists() {
                debug!("Shell script already installed. Doing nothing.");
                return Ok(());
            }
            debug!("No 'installed' : we ask if we can install");
            if !self.can_do()? {
                debug!("User refuses the installation. Doing nothing.");
                return Ok(());
            }
        }
        debug!("Starting install");
        bash::install(self)?;
        fish::install(self)?;
        self.should_quit = true;
        if self.done {
            ShellInstallState::Installed.write_file()?;
            self.skin.print_text(MD_INSTALL_DONE);
        }
        Ok(())
    }

    pub fn remove(&self, path: &Path) -> io::Result<()> {
        // path.exists() doesn't work when the file is a link (it checks whether
        // the link destination exists instead of checking the link exists
        // so we first check whether the link exists
        if fs::read_link(path).is_ok() || path.exists() {
            let path_str = path.to_string_lossy();
            mad_print_inline!(self.skin, "Removing `$0`.\n", &path_str);
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// check whether we're allowed to install.
    fn can_do(&mut self) -> Result<bool, ProgramError> {
        if let Some(authorization) = self.authorization {
            return Ok(authorization);
        }
        let refused_path = get_refused_path();
        if refused_path.exists() {
            debug!("User already refused the installation");
            return Ok(false);
        }
        self.skin.print_text(MD_INSTALL_REQUEST);
        let proceed = cli::ask_authorization()?;
        debug!("proceed: {:?}", proceed);
        self.authorization = Some(proceed);
        if !proceed {
            ShellInstallState::Refused.write_file()?;
            self.skin.print_text(MD_INSTALL_CANCELLED);
        }
        Ok(proceed)
    }

    /// write the script at the given path
    fn write_script(&self, script_path: &Path, content: &str) -> Result<(), ProgramError> {
        self.remove(&script_path)?;
        info!("Writing `br` shell function in `{:?}`", &script_path);
        let script_path_str = script_path.to_string_lossy();
        mad_print_inline!(
            &self.skin,
            "Writing *br* shell function in `$0`.\n",
            &script_path_str
        );
        fs::create_dir_all(script_path.parent().unwrap())?;
        fs::write(&script_path, content)?;
        Ok(())
    }

    /// create a link
    fn create_link(&self, link_path: &Path, script_path: &Path) -> Result<(), ProgramError> {
        info!("Creating link from {:?} to {:?}", &link_path, &script_path);
        self.remove(&link_path)?;
        let link_path_str = link_path.to_string_lossy();
        let script_path_str = script_path.to_string_lossy();
        mad_print_inline!(
            &self.skin,
            "Creating link from `$0` to `$1`.\n",
            &link_path_str,
            &script_path_str,
        );
        fs::create_dir_all(link_path.parent().unwrap())?;
        #[cfg(unix)]
        os::unix::fs::symlink(&script_path, &link_path)?;
        #[cfg(windows)]
        os::windows::fs::symlink_file(&script_path, &link_path)?;
        Ok(())
    }
}
