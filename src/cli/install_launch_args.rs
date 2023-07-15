use {
    crate::{
        errors::ProgramError,
        cli::{Args, CliShellInstallState},
    },
    std::{
        env,
    },
};


/// launch arguments related to installation
/// (not used by the application after the first step)
pub struct InstallLaunchArgs {
    pub install: Option<bool>,                           // installation is required
    pub set_install_state: Option<CliShellInstallState>, // the state to set
    pub print_shell_function: Option<String>,            // shell function to print on stdout
}
impl InstallLaunchArgs {
    pub fn from(args: &Args) -> Result<Self, ProgramError> {
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
        if args.install {
            install = Some(true);
        }
        let print_shell_function = args.print_shell_function.clone();
        let set_install_state = args.set_install_state;
        Ok(Self {
            install,
            set_install_state,
            print_shell_function,
        })
    }
}
