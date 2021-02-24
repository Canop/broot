

use {
    crate::{
        errors::ProgramError,
        shell_install::ShellInstallState,
    },
    clap::{self, ArgMatches},
    std::{
        env,
        str::FromStr,
    },
};


/// launch arguments related to installation
/// (not used by the application after the first step)
pub struct InstallLaunchArgs {
    pub install: Option<bool>,                        // installation is required
    pub set_install_state: Option<ShellInstallState>, // the state to set
    pub print_shell_function: Option<String>,         // shell function to print on stdout
}
impl InstallLaunchArgs {
    pub fn from(cli_args: &ArgMatches<'_>) -> Result<Self, ProgramError> {
        let mut install = None;
        if let Ok(s) = env::var("BR_INSTALL") {
            if s == "yes" {
                install = Some(true);
            } else if s == "no" {
                install = Some(false);
            } else {
                warn!("Unexpected value of BR_INSTALL: {:?}", s);
            }
        }
        // the cli arguments may override the env var value
        if cli_args.is_present("install") {
            install = Some(true);
        } else if cli_args.value_of("cmd-export-path").is_some() {
            install = Some(false);
        }
        let print_shell_function = cli_args
            .value_of("print-shell-function")
            .map(str::to_string);
        let set_install_state = cli_args
            .value_of("set-install-state")
            .map(ShellInstallState::from_str)
            .transpose()?;
        Ok(Self {
            install,
            set_install_state,
            print_shell_function,
        })
    }
}
