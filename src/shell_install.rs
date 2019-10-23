//! The goal of this mod is to ensure the launcher shell function
//! is available i.e. the `br` shell function can be used to launch
//! broot (and thus make it possible to execute some commands, like `cd`,
//! from the starting shell.
//!
//! When everybody's OK, the resulting config dir looks like this:
//!
//!    ~/.config/broot
//!    ├──conf.toml
//!    └──launcher
//!       ├──bash
//!       │  ├──1
//!       │  └──br -> /home/dys/.config/broot/launcher/bash/1
//!       └──installed
//!
//! and a "source .config/broot/launcher/bash/br" line is written in
//! the .bashrc file (and the .zshrc file if found)
//!
//!
//! If the user refused the installation, a "refused" file takes the
//! place of the "installed" one.
//!

use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    os,
    path::Path,
};

use directories::UserDirs;

use crate::{
    cli::{self, AppLaunchArgs},
    conf,
    errors::ProgramError,
    shell_bash::BASH,
    shell_fish::FISH,
};

const SHELL_FAMILIES: &[ShellFamily<'static>] = &[BASH, FISH];

const MD_INSTALL_REQUEST: & str = r#"
**Broot** should be launched using a shell function (see *https://github.com/Canop/broot* for explanations).
The function is either missing, old or badly installed.
"#;

pub struct ShellFamily<'a> {
    pub name: &'a str,
    pub sourcing_files: &'a [&'a str],
    pub version: usize,
    pub script: &'a str,
}

impl ShellFamily<'static> {
    // make sure the script and symlink are installed
    // but don't touch the shellrc files
    // (i.e. this isn't enough to make the function available)
    fn ensure_script_installed(&self, launcher_dir: &Path) -> Result<(), ProgramError> {
        let dir = launcher_dir.join(self.name);
        let link_path = dir.join("br");
        let link_present = link_path.exists();
        let script_path = dir.join(self.version.to_string());
        let func_present = script_path.exists();
        if !func_present {
            info!("script_path not present: writing it");
            fs::create_dir_all(dir)?;
            fs::write(&script_path, self.script)?;
            if link_present {
                fs::remove_file(&link_path)?;
            }
        }
        if !func_present || !link_present {
            info!("creating link from {:?} to {:?}", &link_path, &script_path);
            #[cfg(unix)]
            os::unix::fs::symlink(&script_path, &link_path)?;
            #[cfg(windows)]
            os::windows::fs::symlink_file(&script_path, &link_path)?;
        }
        Ok(())
    }

    /// return true if the application should quit
    fn maybe_patch_all_sourcing_files(
        &self,
        launcher_dir: &Path,
        installation_required: bool,
        motivation_already_explained: bool,
    ) -> Result<bool, ProgramError> {
        let installed_path = launcher_dir.join("installed");
        if installed_path.exists() {
            debug!("*installed* file found");
            // everything seems OK
            // Note that if a new shell has been installed, we don't
            // look again at all the .shellrc files, by design.
            // This means the user having installed a new shell after
            // broot should run `broot --install`
            if !installation_required {
                return Ok(false);
            }
        }
        let refused_path = launcher_dir.join("refused");
        if refused_path.exists() {
            debug!("*refused* file found :(");
            if installation_required {
                fs::remove_file(&refused_path)?;
            } else {
                // user doesn't seem to want the shell function
                return Ok(false);
            }
        }
        // it looks like the shell function is neither installed nor refused
        let ms = cli::mad_skin();
        let homedir_path = match UserDirs::new() {
            Some(user_dirs) => user_dirs.home_dir().to_path_buf(),
            None => {
                warn!("no home directory found!");
                return Ok(false);
            }
        };
        let rc_files: Vec<_> = self
            .sourcing_files
            .iter()
            .map(|name| (name, homedir_path.join(name)))
            .filter(|t| t.1.exists())
            .collect();
        if rc_files.is_empty() {
            warn!(
                "no {} compatible shell config file found, no installation possible",
                self.name
            );
            if installation_required {
                let mut md =
                    String::from("No shell config found, we can't install the br function.\n");
                md.push_str("We were looking for the following file(s):\n");
                for name in self.sourcing_files {
                    md.push_str(&format!("* {:?}\n", homedir_path.join(name)));
                }
                ms.print_text(&md);
            }
            return Ok(installation_required);
        }
        if !installation_required {
            if !motivation_already_explained {
                let mut md = String::from(MD_INSTALL_REQUEST);
                md.push_str(&format!(
                    "Can I add a line in {:?} ? `[Y n]`",
                    rc_files
                        .iter()
                        .map(|f| *f.0)
                        .collect::<Vec<&str>>()
                        .join(" and ")
                ));
                ms.print_text(&md);
            }
            let proceed = cli::ask_authorization()?;
            debug!("proceed: {:?}", proceed);
            if !proceed {
                // user doesn't want the shell function, let's remember it
                fs::write(
                    &refused_path,
                    "to install the br function, run broot --install\n",
                )?;
                ms.print_text("**Okey**. If you change your mind, use `broot --install`.");
                return Ok(false);
            }
        }

        let br_path = launcher_dir.join(self.name).join("br");
        let source_line = format!("source {}", br_path.to_string_lossy());
        let mut changes_made = false;
        for rc_file in rc_files {
            if file_contains_line(&rc_file.1, &source_line)? {
                println!("{} already patched, no change made.", rc_file.0);
            } else {
                let mut shellrc = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&rc_file.1)?;
                shellrc.write_all(b"\n")?;
                shellrc.write_all(source_line.as_bytes())?;
                shellrc.write_all(b"\n")?;
                println!(
                    "{} successfully patched, you should now refresh it with",
                    rc_file.0
                );
                println!("  source {}", rc_file.1.to_string_lossy());
                changes_made = true;
            }
            // signal if there's an old br function declared in the shellrc file
            // (which was the normal way to install before broot 0.6)
            if file_contains_line(&rc_file.1, "function br {")? {
                println!(
                    "Your {} contains another br function, maybe dating from an old version of broot.",
                    rc_file.0
                );
                println!("You should remove it.");
            }
        }
        if changes_made {
            ms.print_text("You should afterwards start broot with just **br**.\n");
        }
        // and remember we did it
        fs::write(
            &installed_path,
            "to reinstall the br function, run broot --install\n",
        )?;
        Ok(changes_made)
    }
}

fn file_contains_line(path: &Path, searched_line: &str) -> io::Result<bool> {
    for line in BufReader::new(fs::File::open(path)?).lines() {
        if line? == searched_line {
            return Ok(true);
        }
    }
    Ok(false)
}

/// check whether the shell function is installed, install
/// it if it wasn't refused before or if broot is launched
/// with --install.
/// returns true if the app should quit
pub fn init(launch_args: &AppLaunchArgs) -> Result<bool, ProgramError> {
    let launcher_dir = conf::dir().join("launcher");
    let mut should_quit = false;
    for family in SHELL_FAMILIES {
        family.ensure_script_installed(&launcher_dir)?;
        let done = family.maybe_patch_all_sourcing_files(
            &launcher_dir,
            launch_args.install,
            should_quit,
        )?;
        should_quit |= done;
    }
    Ok(should_quit)
}
