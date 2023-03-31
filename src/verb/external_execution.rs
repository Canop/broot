use {
    super::*,
    crate::{
        app::*,
        display::W,
        errors::ProgramError,
        launchable::Launchable,
    },
    std::{
        fs::OpenOptions,
        io::Write,
        path::PathBuf,
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

    /// the working directory of the new process, or none if we don't
    /// want to set it
    pub working_dir: Option<String>,

    /// whether we need to switch to the normal terminal for
    /// the duration of the execution of the process
    pub switch_terminal: bool,
}

impl ExternalExecution {
    pub fn new(
        exec_pattern: ExecPattern,
        exec_mode: ExternalExecutionMode,
    ) -> Self {
        Self {
            exec_pattern,
            exec_mode,
            working_dir: None,
            switch_terminal: true, // by default we switch
        }
    }

    pub fn with_working_dir(mut self, b: Option<String>) -> Self {
        self.working_dir = b;
        self
    }

    /// goes from the external execution command to the CmdResult:
    /// - by executing the command if it can be executed from a subprocess
    /// - by building a command to be executed in parent shell in other cases
    pub fn to_cmd_result(
        &self,
        w: &mut W,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        match self.exec_mode {
            ExternalExecutionMode::FromParentShell => self.cmd_result_exec_from_parent_shell(
                builder,
                con,
            ),
            ExternalExecutionMode::LeaveBroot => self.cmd_result_exec_leave_broot(
                builder,
                con,
            ),
            ExternalExecutionMode::StayInBroot => self.cmd_result_exec_stay_in_broot(
                w,
                builder,
                con,
            ),
        }
    }

    fn working_dir_path(
        &self,
        builder: &ExecutionStringBuilder<'_>,
    ) -> Option<PathBuf> {
        self.working_dir
            .as_ref()
            .map(|pattern| builder.path(pattern))
            .filter(|pb| {
                if pb.exists() {
                    true
                } else {
                    warn!("workding dir doesn't exist: {:?}", pb);
                    false
                }
            })
    }

    /// build the cmd result as an executable which will be called
    /// from the parent shell (meaning broot must quit)
    fn cmd_result_exec_from_parent_shell(
        &self,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if builder.sel_info.count_paths() > 1 {
            return Ok(CmdResult::error(
                "only verbs returning to broot on end can be executed on a multi-selection"
            ));
        }
        if let Some(ref export_path) = con.launch_args.outcmd {
            // Broot was probably launched as br.
            // the whole command is exported in the passed file
            let f = OpenOptions::new().append(true).open(export_path)?;
            writeln!(&f, "{}", builder.shell_exec_string(&self.exec_pattern))?;
            Ok(CmdResult::Quit)
        } else {
            Ok(CmdResult::error(
                "this verb needs broot to be launched as `br`. Try `broot --install` if necessary."
            ))
        }
    }

    /// build the cmd result as an executable which will be called in a process
    /// launched by broot at end of broot
    fn cmd_result_exec_leave_broot(
        &self,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        if builder.sel_info.count_paths() > 1 {
            return Ok(CmdResult::error(
                "only verbs returning to broot on end can be executed on a multi-selection"
            ));
        }
        let launchable = Launchable::program(
            builder.exec_token(&self.exec_pattern),
            self.working_dir_path(&builder),
            self.switch_terminal,
            con,
        )?;
        Ok(CmdResult::from(launchable))
    }

    /// build the cmd result as an executable which will be called in a process
    /// launched by broot
    fn cmd_result_exec_stay_in_broot(
        &self,
        w: &mut W,
        builder: ExecutionStringBuilder<'_>,
        con: &AppContext,
    ) -> Result<CmdResult, ProgramError> {
        let working_dir_path = self.working_dir_path(&builder);
        match &builder.sel_info {
            SelInfo::None | SelInfo::One(_) => {
                // zero or one selection -> only one execution
                let launchable = Launchable::program(
                    builder.exec_token(&self.exec_pattern),
                    working_dir_path,
                    self.switch_terminal,
                    con,
                )?;
                info!("Executing not leaving, launchable {:?}", launchable);
                if let Err(e) = launchable.execute(Some(w)) {
                    warn!("launchable failed : {:?}", e);
                    return Ok(CmdResult::error(e.to_string()));
                }
            }
            SelInfo::More(stage) => {
                // multiselection -> we must execute on all paths
                let sels = stage.paths().iter()
                    .map(|path| Selection {
                        path,
                        line: 0,
                        stype: SelectionType::from(path),
                        is_exe: false,
                    });
                for sel in sels {
                    let launchable = Launchable::program(
                        builder.sel_exec_token(&self.exec_pattern, Some(sel)),
                        working_dir_path.clone(),
                        self.switch_terminal,
                        con,
                    )?;
                    if let Err(e) = launchable.execute(Some(w)) {
                        warn!("launchable failed : {:?}", e);
                        return Ok(CmdResult::error(e.to_string()));
                    }
                }
            }
        }
        Ok(CmdResult::RefreshState { clear_cache: true })
    }
}
