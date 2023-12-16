//! The goal of this mod is to ensure the launcher shell function
//! is available for nushell i.e. the `br` shell function can
//! be used to launch broot (and thus make it possible to execute
//! some commands, like `cd`, from the starting shell.
//!
//! In a correct installation, we have:
//! - a function declaration script in ~/.local/share/broot/launcher/nushell/br/1
//! - a link to that script in ~/.config/broot/launcher/nushell/br/1
//! - a line to source the link in ~/.config/nushell/config.nu
//! (exact paths depend on XDG variables)
//!
//! Please note that this function doesn't allow other commands than cd,
//! contrary to the similar function of other shells.

use {
    super::{util, ShellInstall},
    crate::{conf, errors::*},
    directories::BaseDirs,
    std::path::PathBuf,
    termimad::mad_print_inline,
};

const NAME: &str = "nushell";
const VERSION: &str = "5";

const NU_FUNC: &str = r#"
# Launch broot
#
# Examples:
#   > br -hi some/path
#   > br
#   > br -sdp
#   > br -hi -c "vacheblan.svg;:open_preview" ..
#
# See https://dystroy.org/broot/install-br/
def --env br [
    --cmd(-c): string               # Semicolon separated commands to execute
    --color: string = "auto"        # Whether to have styles and colors (auto is default and usually OK) [possible values: auto, yes, no]
    --conf: string                  # Semicolon separated paths to specific config files"),
    --dates(-d)                     # Show the last modified date of files and directories"
    --no-dates(-D)                  # Don't show the last modified date"
    --only-folders(-f)              # Only show folders
    --no-only-folders(-F)           # Show folders and files alike
    --show-git-info(-g)             # Show git statuses on files and stats on repo
    --no-show-git-info(-G)          # Don't show git statuses on files and stats on repo
    --git-status                    # Only show files having an interesting git status, including hidden ones
    --hidden(-h)                    # Show hidden files
    --no-hidden(-H)                 # Don't show hidden files
    --height: int                   # Height (if you don't want to fill the screen or for file export)
    --help                          # Print help information
    --git-ignored(-i)               # Show git ignored files
    --no-git-ignored(-I)            # Don't show git ignored files
    --install                       # Install or reinstall the br shell function
    --no-sort                       # Don't sort
    --permissions(-p)               # Show permissions
    --no-permissions(-P)            # Don't show permissions
    --print-shell-function: string  # Print to stdout the br function for a given shell
    --sizes(-s)                     # Show the size of files and directories
    --no-sizes(-S)                  # Don't show sizes
    --set-install-state: path       # Where to write the produced cmd (if any) [possible values: undefined, refused, installed]
    --show-root-fs                  # Show filesystem info on top
    --sort-by-count                 # Sort by count (only show one level of the tree)
    --sort-by-date                  # Sort by date (only show one level of the tree)
    --sort-by-size                  # Sort by size (only show one level of the tree)
    --sort-by-type                  # Same as sort-by-type-dirs-first
    --sort-by-type-dirs-first       # Sort by type, directories first (only show one level of the tree)
    --sort-by-type-dirs-last        # Sort by type, directories last (only show one level of the tree)
    --trim-root(-t)                 # Trim the root too and don't show a scrollbar
    --no-trim-root(-T)              # Don't trim the root level, show a scrollbar
    --version(-V)                   # Print version information
    --whale-spotting(-w)            # Sort by size, show ignored and hidden files
    --write-default-conf: path      # Write default conf files in given directory
    file?: path                     # Root Directory
] {
    mut args = []
    if $cmd != null { $args = ($args | append $'--cmd=($cmd)') }
    if $color != null { $args = ($args | append $'--color=($color)') }
    if $conf != null { $args = ($args | append $'--conf=($conf)') }
    if $dates { $args = ($args | append $'--dates') }
    if $no_dates { $args = ($args | append $'--no-dates') }
    if $only_folders { $args = ($args | append $'--only-folders') }
    if $no_only_folders { $args = ($args | append $'--no-only-folders') }
    if $show_git_info { $args = ($args | append $'--show-git-info') }
    if $no_show_git_info { $args = ($args | append $'--no-show-git-info') }
    if $git_status { $args = ($args | append $'--git-status') }
    if $hidden { $args = ($args | append $'--hidden') }
    if $no_hidden { $args = ($args | append $'--no-hidden') }
    if $height != null { $args = ($args | append $'--height=($height)') }
    if $help { $args = ($args | append $'--help') }
    if $git_ignored { $args = ($args | append $'--git-ignored') }
    if $no_git_ignored { $args = ($args | append $'--no-git-ignored') }
    if $install { $args = ($args | append $'--install') }
    if $no_sort { $args = ($args | append $'--no-sort') }
    if $permissions { $args = ($args | append $'--permissions') }
    if $no_permissions { $args = ($args | append $'--no-permissions') }
    if $print_shell_function != null { $args = ($args | append $'--print-shell-function=($print_shell_function)') }
    if $sizes { $args = ($args | append $'--sizes') }
    if $no_sizes { $args = ($args | append $'--no-sizes') }
    if $set_install_state != null { $args = ($args | append $'--set-install-state=($set_install_state)') }
    if $show_root_fs { $args = ($args | append $'--show-root-fs') }
    if $sort_by_count { $args = ($args | append $'--sort-by-count') }
    if $sort_by_date { $args = ($args | append $'--sort-by-date') }
    if $sort_by_size { $args = ($args | append $'--sort-by-size') }
    if $sort_by_type { $args = ($args | append $'--sort-by-type') }
    if $sort_by_type_dirs_first { $args = ($args | append $'--sort-by-type-dirs-first') }
    if $sort_by_type_dirs_last { $args = ($args | append $'--sort-by-type-dirs-last') }
    if $trim_root { $args = ($args | append $'--trim-root') }
    if $no_trim_root { $args = ($args | append $'--no-trim-root') }
    if $version { $args = ($args | append $'--version') }
    if $whale_spotting { $args = ($args | append $'--whale-spotting') }
    if $write_default_conf != null { $args = ($args | append $'--write-default-conf=($write_default_conf)') }

    let cmd_file = ([ $nu.temp-path, $"broot-(random chars).tmp" ] | path join)
    touch $cmd_file
    if ($file == null) {
        ^broot --outcmd $cmd_file $args
    } else {
        ^broot --outcmd $cmd_file $args $file
    }
    let $cmd = (open $cmd_file)
    rm -p -f $cmd_file

    if (not ($cmd | lines | is-empty)) {
        cd ($cmd | parse -r `^cd\s+(?<quote>"|'|)(?<path>.+)\k<quote>[\s\r\n]*$` | get path | to text)
    }
}

export extern broot [
    --cmd(-c): string               # Semicolon separated commands to execute
    --color: string = "auto"        # Whether to have styles and colors (auto is default and usually OK) [possible values: auto, yes, no]
    --conf: string                  # Semicolon separated paths to specific config files"),
    --dates(-d)                     # Show the last modified date of files and directories"
    --no-dates(-D)                  # Don't show the last modified date"
    --only-folders(-f)              # Only show folders
    --no-only-folders(-F)           # Show folders and files alike
    --show-git-info(-g)             # Show git statuses on files and stats on repo
    --no-show-git-info(-G)          # Don't show git statuses on files and stats on repo
    --git-status                    # Only show files having an interesting git status, including hidden ones
    --hidden(-h)                    # Show hidden files
    --no-hidden(-H)                 # Don't show hidden files
    --height: int                   # Height (if you don't want to fill the screen or for file export)
    --help                          # Print help information
    --git-ignored(-i)               # Show git ignored files
    --no-git-ignored(-I)            # Don't show git ignored files
    --install                       # Install or reinstall the br shell function
    --no-sort                       # Don't sort
    --outcmd: path                  # Write cd command in given path
    --permissions(-p)               # Show permissions
    --no-permissions(-P)            # Don't show permissions
    --print-shell-function: string  # Print to stdout the br function for a given shell
    --sizes(-s)                     # Show the size of files and directories
    --no-sizes(-S)                  # Don't show sizes
    --set-install-state: path       # Where to write the produced cmd (if any) [possible values: undefined, refused, installed]
    --show-root-fs                  # Show filesystem info on top
    --sort-by-count                 # Sort by count (only show one level of the tree)
    --sort-by-date                  # Sort by date (only show one level of the tree)
    --sort-by-size                  # Sort by size (only show one level of the tree)
    --sort-by-type                  # Same as sort-by-type-dirs-first
    --sort-by-type-dirs-first       # Sort by type, directories first (only show one level of the tree)
    --sort-by-type-dirs-last        # Sort by type, directories last (only show one level of the tree)
    --trim-root(-t)                 # Trim the root too and don't show a scrollbar
    --no-trim-root(-T)              # Don't trim the root level, show a scrollbar
    --version(-V)                   # Print version information
    --whale-spotting(-w)            # Sort by size, show ignored and hidden files
    --write-default-conf: path      # Write default conf files in given directory
    file?: path                     # Root Directory
]
"#;

pub fn get_script() -> &'static str {
    NU_FUNC
}

/// return the path to the link to the function script
fn get_link_path() -> PathBuf {
    conf::dir().join("launcher").join(NAME).join("br")
}

/// return the root of
fn get_nushell_dir() -> Option<PathBuf> {
    BaseDirs::new()
        .map(|base_dirs| base_dirs.config_dir().join("nushell"))
        .filter(|dir| dir.exists())
}

/// return the path to the script containing the function.
///
/// In XDG_DATA_HOME (typically ~/.local/share on linux)
fn get_script_path() -> PathBuf {
    conf::app_dirs()
        .data_dir()
        .join("launcher")
        .join(NAME)
        .join(VERSION)
}

/// Check for nushell.
///
/// Check whether the shell function is installed, install
/// it if it wasn't refused before or if broot is launched
/// with --install.
pub fn install(si: &mut ShellInstall) -> Result<(), ShellInstallError> {
    info!("install {NAME}");
    let Some(nushell_dir) = get_nushell_dir() else {
        debug!("no nushell config directory. Assuming nushell isn't used.");
        return Ok(());
    };
    info!("nushell seems to be installed");
    let script_path = get_script_path();
    si.write_script(&script_path, NU_FUNC)?;
    let link_path = get_link_path();
    si.create_link(&link_path, &script_path)?;

    let escaped_path = link_path.to_string_lossy().replace(' ', "\\ ");
    let source_line = format!("source {}", &escaped_path);

    let sourcing_path = nushell_dir.join("config.nu");
    if !sourcing_path.exists() {
        warn!("Unexpected lack of config.nu file");
        return Ok(());
    }
    if sourcing_path.is_dir() {
        warn!("config.nu file");
        return Ok(());
    }
    let sourcing_path_str = sourcing_path.to_string_lossy();
    if util::file_contains_line(&sourcing_path, &source_line)? {
        mad_print_inline!(
            &si.skin,
            "`$0` already patched, no change made.\n",
            &sourcing_path_str,
        );
    } else {
        util::append_to_file(&sourcing_path, format!("\n{source_line}\n"))?;
        mad_print_inline!(
            &si.skin,
            "`$0` successfully patched, you can make the function immediately available with `source $0`\n",
            &sourcing_path_str,
        );
    }
    si.done = true;
    Ok(())
}
