use {
    crate::{
        cli::{self, ShellInstallState},
        conf,
        errors::*,
        skin,
    },
    std::{
        fs,
        os,
        path::{Path, PathBuf},
    },
    termimad::{mad_print_inline, MadSkin},
};

mod bash;
mod fish;
mod nushell;
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

const MD_PERMISSION_DENIED: &str = r#"
Installation check resulted in **Permission Denied**.
Please relaunch with elevated privilege.
This is typically only needed once.
Error details:
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

/// write either the "installed" or the "refused" file, or remove
///  those files.
///
/// This is useful in installation
/// or test scripts when we don't want the user to be prompted
/// to install the function, or in case something doesn't properly
/// work in shell detections
pub fn write_state(state: ShellInstallState) -> Result<(), ShellInstallError> {
    let refused_path = get_refused_path();
    let installed_path = get_installed_path();
    if installed_path.exists() {
        fs::remove_file(&installed_path)
            .context(&|| format!("removing {:?}", &installed_path))?;
    }
    if refused_path.exists() {
        fs::remove_file(&refused_path)
            .context(&|| format!("removing {:?}", &refused_path))?;
    }
    match state {
        ShellInstallState::Refused => {
            fs::create_dir_all(refused_path.parent().unwrap())
                .context(&|| format!("creating parents of {refused_path:?}"))?;
            fs::write(&refused_path, REFUSED_FILE_CONTENT)
                .context(&|| format!("writing in {refused_path:?}"))?;
        }
        ShellInstallState::Installed => {
            fs::create_dir_all(installed_path.parent().unwrap())
                .context(&|| format!("creating parents of {installed_path:?}"))?;
            fs::write(&installed_path, INSTALLED_FILE_CONTENT)
                .context(&|| format!("writing in {installed_path:?}"))?;
        }
        _ => {}
    }
    Ok(())
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
            "nushell" => println!("{}", nushell::get_script()),
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
    pub fn check(&mut self) -> Result<(), ShellInstallError> {
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
            // even if the installation isn't really complete (for example
            // when no bash file was found), we don't want to ask the user
            // again, we'll assume it's done
            write_state(ShellInstallState::Installed)?;
        }
        debug!("Starting install");
        bash::install(self)?;
        fish::install(self)?;
        nushell::install(self)?;
        self.should_quit = true;
        if self.done {
            self.skin.print_text(MD_INSTALL_DONE);
        }
        Ok(())
    }

    /// print some additional information on the error (typically before
    /// the error itself is dumped)
    pub fn comment_error(&self, err: &ShellInstallError) {
        if err.is_permission_denied() {
            self.skin.print_text(MD_PERMISSION_DENIED);
        }
    }

    pub fn remove(&self, path: &Path) -> Result<(), ShellInstallError> {
        // path.exists() doesn't work when the file is a link (it checks whether
        // the link destination exists instead of checking the link exists
        // so we first check whether the link exists
        if fs::read_link(path).is_ok() || path.exists() {
            mad_print_inline!(self.skin, "Removing `$0`.\n", path.to_string_lossy());
            fs::remove_file(path)
                .context(&|| format!("removing {path:?}"))?;
        }
        Ok(())
    }

    /// check whether we're allowed to install.
    fn can_do(&mut self) -> Result<bool, ShellInstallError> {
        if let Some(authorization) = self.authorization {
            return Ok(authorization);
        }
        let refused_path = get_refused_path();
        if refused_path.exists() {
            debug!("User already refused the installation");
            return Ok(false);
        }
        self.skin.print_text(MD_INSTALL_REQUEST);
        let proceed = cli::ask_authorization()
            .context(&|| "asking user".to_string())?; // read_line failure
        debug!("proceed: {:?}", proceed);
        self.authorization = Some(proceed);
        if !proceed {
            write_state(ShellInstallState::Refused)?;
            self.skin.print_text(MD_INSTALL_CANCELLED);
        }
        Ok(proceed)
    }

    /// write the script at the given path
    fn write_script(&self, script_path: &Path, content: &str) -> Result<(), ShellInstallError> {
        self.remove(script_path)?;
        info!("Writing `br` shell function in `{:?}`", &script_path);
        mad_print_inline!(
            &self.skin,
            "Writing *br* shell function in `$0`.\n",
            script_path.to_string_lossy(),
        );
        fs::create_dir_all(script_path.parent().unwrap())
            .context(&|| format!("creating parent dirs to {script_path:?}"))?;
        fs::write(script_path, content)
            .context(&|| format!("writing script in {script_path:?}"))?;
        Ok(())
    }


    /// create a link
    fn create_link(&self, link_path: &Path, script_path: &Path) -> Result<(), ShellInstallError> {
        info!("Creating link from {:?} to {:?}", &link_path, &script_path);
        self.remove(link_path)?;
        let link_path_str = link_path.to_string_lossy();
        let script_path_str = script_path.to_string_lossy();
        mad_print_inline!(
            &self.skin,
            "Creating link from `$0` to `$1`.\n",
            &link_path_str,
            &script_path_str,
        );
        let parent = link_path.parent().unwrap();
        fs::create_dir_all(parent)
            .context(&|| format!("creating directory {parent:?}"))?;
        #[cfg(unix)]
        os::unix::fs::symlink(script_path, link_path)
            .context(&|| format!("linking from {link_path:?} to {script_path:?}"))?;
        #[cfg(windows)]
        os::windows::fs::symlink_file(&script_path, &link_path)
            .context(&|| format!("linking from {link_path:?} to {script_path:?}"))?;
        Ok(())
    }
}
