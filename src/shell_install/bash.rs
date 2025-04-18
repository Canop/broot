//! The goal of this mod is to ensure the launcher shell function
//! is available for bash and zsh i.e. the `br` shell function can
//! be used to launch broot (and thus make it possible to execute
//! some commands, like `cd`, from the starting shell.
//!
//! In a correct installation, we have:
//! - a function declaration script in ~/.local/share/broot/launcher/bash/br/1
//! - a link to that script in ~/.config/broot/launcher/bash/br/1
//! - a line to source the link in ~/.bashrc and ~/.zshrc
//!
//! (exact paths depend on XDG variables)

use {
    super::{util, ShellInstall},
    crate::{
        conf,
        errors::*,
    },
    directories::UserDirs,
    lazy_regex::regex,
    regex::Captures,
    std::{env, path::PathBuf},
    termimad::{
        mad_print_inline,
    },
};

const NAME: &str = "bash";
const SOURCING_FILES: &[&str] = &[".bashrc", ".bash_profile", ".zshrc", "$ZDOTDIR/.zshrc"];
const VERSION: &str = "1";

// This script has been tested on bash and zsh.
// It's installed under the bash name (~/.config/broot
// but linked from both the .bashrc and the .zshrc files
const BASH_FUNC: &str = r#"
# This script was automatically generated by the broot program
# More information can be found in https://github.com/Canop/broot
# This function starts broot and executes the command
# it produces, if any.
# It's needed because some shell commands, like `cd`,
# have no useful effect if executed in a subshell.
function br {
    local cmd cmd_file code
    cmd_file=$(mktemp)
    if broot --outcmd "$cmd_file" "$@"; then
        cmd=$(<"$cmd_file")
        command rm -f "$cmd_file"
        eval "$cmd"
    else
        code=$?
        command rm -f "$cmd_file"
        return "$code"
    fi
}
"#;

const MD_NO_SOURCING: &str = r"
I found no sourcing file for the bash/zsh family.
If you're using bash or zsh, then installation isn't complete:
the br function initialization script won't be sourced unless you source it yourself.
";

pub fn get_script() -> &'static str {
    BASH_FUNC
}

/// return the path to the link to the function script
fn get_link_path() -> PathBuf {
    conf::dir().join("launcher").join(NAME).join("br")
}

/// return the path to the script containing the function.
///
/// At version 0.10.4 we change the location of the script:
/// It was previously with the link, but it's now in
/// XDG_DATA_HOME (typically ~/.local/share on linux)
fn get_script_path() -> PathBuf {
    conf::app_dirs()
        .data_dir()
        .join("launcher")
        .join(NAME)
        .join(VERSION)
}

/// return the paths to the files in which the br function is sourced.
/// Paths in SOURCING_FILES can be absolute or relative to the home
/// directory. Environment variables designed as $NAME are interpolated.
fn get_sourcing_paths() -> Vec<PathBuf> {
    let homedir_path = UserDirs::new()
        .expect("no home directory!")
        .home_dir()
        .to_path_buf();
    SOURCING_FILES
        .iter()
        .map(|name| {
            regex!(r#"\$(\w+)"#)
                .replace(name, |c: &Captures<'_>| {
                    env::var(&c[1]).unwrap_or_else(|_| (*name).to_string())
                })
                .to_string()
        })
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                homedir_path.join(path)
            }
        })
        .filter(|path| {
            debug!("considering path: {:?}", &path);
            path.exists()
        })
        .collect()
}

/// check for bash and zsh shells.
/// check whether the shell function is installed, install
/// it if it wasn't refused before or if broot is launched
/// with --install.
pub fn install(si: &mut ShellInstall) -> Result<(), ShellInstallError> {
    let script_path = get_script_path();
    si.write_script(&script_path, BASH_FUNC)?;
    let link_path = get_link_path();
    si.create_link(&link_path, &script_path)?;
    let sourcing_paths = get_sourcing_paths();
    if sourcing_paths.is_empty() {
        warn!("no sourcing path for bash/zsh!");
        si.skin.print_text(MD_NO_SOURCING);
        return Ok(());
    }
    let escaped_path = link_path.to_string_lossy().replace(' ', "\\ ");
    let source_line = {
        if env::consts::OS == "windows" {
            // Bash on Windows doesn't like C:\Users\... but will accept "C:\Users\..."
            format!("source \"{}\"", &escaped_path)
        } else {
            format!("source {}", &escaped_path)
        }
    };
    for sourcing_path in &sourcing_paths {
        let sourcing_path_str = sourcing_path.to_string_lossy();
        if util::file_contains_line(sourcing_path, &source_line)? {
            mad_print_inline!(
                &si.skin,
                "`$0` already patched, no change made.\n",
                &sourcing_path_str,
            );
        } else {
            util::append_to_file(sourcing_path, format!("\n{source_line}\n"))?;
            let is_zsh = sourcing_path_str.contains(".zshrc");
            if is_zsh {
                mad_print_inline!(
                    &si.skin,
                    "`$0` successfully patched, you can make the function immediately available with `exec zsh`\n",
                    &sourcing_path_str,
                );
            } else {
                mad_print_inline!(
                    &si.skin,
                    "`$0` successfully patched, you can make the function immediately available with `source $0`\n",
                    &sourcing_path_str,
                );
            }
        }
    }
    si.done = true;
    Ok(())
}
