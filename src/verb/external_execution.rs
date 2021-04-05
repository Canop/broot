use {
    super::*,
    crate::{
        app::*,
        display::W,
        errors::ProgramError,
        launchable::Launchable,
        path,
    },
    std::{
        fs::OpenOptions,
        io::Write,
    },
};


/// Definition of how the user input should be interpreted
/// to be executed in an external command.
#[derive(Debug, Clone)]
pub struct ExternalExecution {
    /// the pattern which will result in an executable string when
    /// completed with the args.
    /// This pattern may include names coming from the invocation
    /// pattern (like {my-arg}) and special names automatically filled by
    /// broot from the selection and application state:
    /// * {file}
    /// * {directory}
    /// * {parent}
    /// * {other-panel-file}
    /// * {other-panel-directory}
    /// * {other-panel-parent}
    pub exec_pattern: ExecPattern,

    /// how the external process must be launched
    pub exec_mode: ExternalExecutionMode,

    /// whether the working dir of the external process must be set
    /// to the current directory
    pub set_working_dir: bool,
}

impl ExternalExecution {
    pub fn new(
        exec_pattern: ExecPattern,
        exec_mode: ExternalExecutionMode,
    ) -> Self {
        Self {
            exec_pattern,
            exec_mode,
            set_working_dir: false,
        }
    }

    pub fn with_set_working_dir(mut self, b: Option<bool>) -> Self {
        if let Some(b) = b {
            self.set_working_dir = b;
        }
        self
    }

    pub fn to_cmd_result(
        &self,
        w: &mut W,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if self.exec_mode.is_from_shell() {
            self.exec_from_shell_cmd_result(builder, con)
        } else {
            self.exec_cmd_result(w, builder, con)
        }
    }

    /// build the cmd result as an executable which will be called from shell
    fn exec_from_shell_cmd_result(
        &self,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if let Some(ref export_path) = con.launch_args.cmd_export_path {
            // Broot was probably launched as br.
            // the whole command is exported in the passed file
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", builder.shell_exec_string(&self.exec_pattern))?;
            Ok(CmdResult::Quit)
        } else if let Some(ref export_path) = con.launch_args.file_export_path {
            // old version of the br function: only the file is exported
            // in the passed file
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", builder.sel.path.to_string_lossy())?;
            Ok(CmdResult::Quit)
        } else {
            Ok(CmdResult::DisplayError(
                "this verb needs broot to be launched as `br`. Try `broot --install` if necessary."
                    .to_string(),
            ))
        }
    }

    /// build the cmd result as an executable which will be called in a process
    /// launched by broot
    fn exec_cmd_result(
        &self,
        w: &mut W,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        let launchable = Launchable::program(
            builder.exec_token(&self.exec_pattern),
            if self.set_working_dir {
                Some(path::closest_dir(builder.sel.path))
            } else {
                None
            },
            con,
        )?;
        if self.exec_mode.is_leave_broot() {
            Ok(CmdResult::from(launchable))
        } else {
            info!("Executing not leaving, launchable {:?}", launchable);
            let execution = launchable.execute(Some(w));
            match execution {
                Ok(()) => {
                    debug!("ok");
                    Ok(CmdResult::RefreshState { clear_cache: true })
                }
                Err(e) => {
                    warn!("launchable failed : {:?}", e);
                    Ok(CmdResult::DisplayError(e.to_string()))
                }
            }
        }
    }
}
