//! The goal of this mod is to ensure the launcher shell function
//! is available for PowerShell i.e. the `br` shell function can
//! be used to launch broot (and thus make it possible to execute
//! some commands, like `cd`, from the starting shell.
//!
//! In a correct installation, we have:
//! - a function declaration script in %APPDATA%/dystroy/broot/data/launcher/powershell/1
//! - a link to that script in %APPDATA%/dystroy/broot/config/launcher/powershell/br.ps1
//! - a line to source the link in %USERPROFILE%/Documents/WindowsPowerShell/Profile.ps1

use {
    super::{
        ShellInstall,
        util,
    },
    crate::{
        conf,
        errors::*,
    },
    directories::UserDirs,
    std::{
        fs,
        path::PathBuf,
    },
    termimad::mad_print_inline,
};

const NAME: &str = "powershell";
const VERSION: &str = "1";

const PS_FUNC: &str = r#"
# https://github.com/Canop/broot/issues/460#issuecomment-1303005689
Function br {
  $args = $args -join ' '
  $cmd_file = New-TemporaryFile

  $process = Start-Process -FilePath 'broot.exe' `
                           -ArgumentList "--outcmd $($cmd_file.FullName) $args" `
                           -NoNewWindow -PassThru -WorkingDirectory $PWD

  Wait-Process -InputObject $process #Faster than Start-Process -Wait
  If ($process.ExitCode -eq 0) {
    $cmd = Get-Content $cmd_file
    Remove-Item $cmd_file
    If ($cmd -ne $null) { Invoke-Expression -Command $cmd }
  } Else {
    Remove-Item $cmd_file
    Write-Host "`n" # Newline to tidy up broot unexpected termination
    Write-Error "broot.exe exited with error code $($process.ExitCode)"
  }
}
"#;

pub fn get_script() -> &'static str {
    PS_FUNC
}

/// return the path to the link to the function script
fn get_link_path() -> PathBuf {
    conf::dir().join("launcher").join(NAME).join("br.ps1")
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

/// Check whether the shell function is installed, install
/// it if it wasn't refused before or if broot is launched
/// with --install.
#[allow(unreachable_code, unused_variables)]
pub fn install(si: &mut ShellInstall) -> Result<(), ShellInstallError> {
    info!("install {NAME}");
    #[cfg(unix)]
    {
        debug!("Shell install not supported for PowerShell on unix-based systems.");
        return Ok(());
    }
    let Some(user_dir) = UserDirs::new() else {
        warn!("Could not find user directory.");
        return Ok(());
    };
    let Some(document_dir) = user_dir.document_dir() else {
        warn!("Could not find user documents directory.");
        return Ok(());
    };

    let script_path = get_script_path();
    si.write_script(&script_path, PS_FUNC)?;
    let link_path = get_link_path();
    si.create_link(&link_path, &script_path)?;

    let escaped_path = link_path.to_string_lossy().replace(' ', "\\ ");
    let source_line = format!(". {}", &escaped_path);

    let sourcing_path = document_dir.join("WindowsPowerShell").join("Profile.ps1");
    if !sourcing_path.exists() {
        debug!("Creating missing PowerShell profile file.");
        if let Some(parent) = sourcing_path.parent() {
            fs::create_dir_all(parent).context(&|| format!("creating {parent:?} directory"))?;
        }
        fs::File::create(&sourcing_path).context(&|| format!("creating {sourcing_path:?}"))?;
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
        mad_print_inline!(&si.skin, "`$0` successfully patched.\n", &sourcing_path_str,);
    }
    si.done = true;
    Ok(())
}
