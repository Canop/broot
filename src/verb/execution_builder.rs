use {
    super::*,
    crate::{
        app::*,
        command::*,
        path,
    },
    rustc_hash::FxHashMap,
    regex::Captures,
    std::path::{Path, PathBuf},
};

/// a temporary structure gathering selection and invocation
/// parameters and able to generate an executable string from
/// a verb's execution pattern
pub struct ExecutionStringBuilder<'b> {
    /// the current file selection
    pub sel_info: SelInfo<'b>,

    /// the current root of the app
    root: &'b Path,

    /// the selection in the other panel, when there are exactly two
    other_file: Option<&'b PathBuf>,

    /// parsed arguments
    invocation_values: Option<FxHashMap<String, String>>,

    /// whether to keep groups which can't be solved or remove them
    keep_groups: bool,
}

impl<'b> ExecutionStringBuilder<'b> {
    /// constructor to use when there's no invocation string
    /// (because we're in the process of building one, for example
    /// when a verb is triggered from a key shortcut)
    pub fn without_invocation(
         sel_info: SelInfo<'b>,
         app_state: &'b AppState,
    ) -> Self {
        Self {
            sel_info,
            root: &app_state.root,
            other_file: app_state.other_panel_path.as_ref(),
            invocation_values: None,
            keep_groups: false,
        }
    }
    pub fn with_invocation(
        invocation_parser: Option<&InvocationParser>,
        sel_info: SelInfo<'b>,
        app_state: &'b AppState,
        invocation_args: Option<&String>,
    ) -> Self {
        let invocation_values = invocation_parser
            .as_ref()
            .zip(invocation_args.as_ref())
            .and_then(|(parser, args)| parser.parse(args));
        Self {
            sel_info,
            root: &app_state.root,
            other_file: app_state.other_panel_path.as_ref(),
            invocation_values,
            keep_groups: false,
        }
    }
    fn get_raw_replacement<F>(
        &self,
        f: F
    ) -> Option<String>
    where
        F: Fn(Option<Selection<'_>>) -> Option<String>
    {
        match self.sel_info {
            SelInfo::None => f(None),
            SelInfo::One(sel) => f(Some(sel)),
            SelInfo::More(stage) => {
                let mut sels = stage.paths().iter()
                    .map(|path| Selection {
                        path,
                        line: 0,
                        stype: SelectionType::from(path),
                        is_exe: false,
                    });
                f(sels.next())
                    .filter(|first_rcr| {
                        for sel in sels {
                            let rcr = f(Some(sel));
                            if rcr.as_ref() != Some(first_rcr) {
                                return false;
                            }
                        }
                        true
                    })
            }
        }
    }
    fn get_raw_capture_replacement(
        &self,
        ec: &Captures<'_>,
        con: &AppContext,
    ) -> Option<String> {
        self.get_raw_replacement(|sel| {
            self.get_raw_sel_capture_replacement(ec, sel, con)
        })
    }
    /// return the standard replacement (ie not one from the invocation)
    fn get_raw_sel_name_standard_replacement(
        &self,
        name: &str,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Option<String> {
        debug!("repl name : {:?}", name);
        match name {
            "root" => Some(path_to_string(self.root)),
            "initial-root" => Some(path_to_string(&con.initial_root)),
            "line" => sel.map(|s| s.line.to_string()),
            "file" => sel.map(|s| s.path)
                .map(path_to_string),
            "file-name" => sel.map(|s| s.path)
                .and_then(|path| path.file_name())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string()),
            "file-stem" => sel.map(|s| s.path)
                .and_then(|path| path.file_stem())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string()),
            "file-extension" => {
                debug!("expending file extension");
                sel.map(|s| s.path)
                .and_then(|path| path.extension())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string())
            }
            "file-dot-extension" => {
                debug!("expending file dot extension");
                sel.map(|s| s.path)
                .and_then(|path| path.extension())
                .and_then(|oss| oss.to_str())
                .map(|ext| format!(".{ext}"))
                .or_else(|| Some("".to_string()))
            }
            "directory" => sel.map(|s| path::closest_dir(s.path))
                .map(path_to_string),
            "parent" => sel.and_then(|s| s.path.parent())
                .map(path_to_string),
            "other-panel-file" => self.other_file
                .map(path_to_string),
            "other-panel-filename" => self.other_file
                .and_then(|path| path.file_name())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string()),
            "other-panel-directory" => self
                .other_file
                .map(|p| path::closest_dir(p))
                .as_ref()
                .map(path_to_string),
            "other-panel-parent" => self
                .other_file
                .and_then(|p| p.parent())
                .map(path_to_string),
            "git-root" => { // path to git repo workdir
                debug!("finding git root");
                sel
                    .and_then(|s| git2::Repository::discover(s.path).ok())
                    .and_then(|repo| repo.workdir().map(path_to_string))
            }
            "git-name" => { // name of the git repo workdir
                sel
                    .and_then(|s| git2::Repository::discover(s.path).ok())
                    .and_then(|repo| repo.workdir().and_then(|path| {
                        path.file_name()
                        .and_then(|oss| oss.to_str())
                        .map(|s| s.to_string())
                    }))
            }
            "file-git-relative" => { // file path relative to git repo workdir
                let sel = sel?;
                let path = git2::Repository::discover(self.root).ok()
                    .and_then(|repo| repo.workdir().map(path_to_string))
                    .and_then(|gitroot| sel.path.strip_prefix(gitroot).ok())
                    .filter(|p| {
                        // it's empty when the file is both the tree root and the git root
                        !p.as_os_str().is_empty()
                    })
                    .unwrap_or(sel.path);
                Some(path_to_string(path))
            }
            _ => None,
        }
    }
    fn get_raw_sel_capture_replacement(
        &self,
        ec: &Captures<'_>,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Option<String> {
        let name = ec.get(1).unwrap().as_str();
        self.get_raw_sel_name_standard_replacement(name, sel, con)
            .or_else(||{
                // it's not one of the standard group names, so we'll look
                // into the ones provided by the invocation pattern
                self.invocation_values.as_ref()
                    .and_then(|map| map.get(name))
                    .and_then(|value| {
                        if let Some(fmt) = ec.get(2) {
                            match fmt.as_str() {
                                "path-from-directory" => {
                                    sel.map(|s| path::closest_dir(s.path))
                                        .map(|dir| path::path_str_from(dir, value))
                                }
                                "path-from-parent" => {
                                     sel.and_then(|s| s.path.parent())
                                        .map(|dir| path::path_str_from(dir, value))
                                }
                                _ => Some(format!("invalid format: {:?}", fmt.as_str())),
                            }
                        } else {
                            Some(value.to_string())
                        }
                    })
            })
    }
    #[inline]
    fn get_capture_replacement(
        &self,
        ec: &Captures<'_>,
        con: &AppContext,
    ) -> String {
        self.get_raw_capture_replacement(ec, con)
            .unwrap_or_else(||
                if self.keep_groups { ec[0].to_string() } else { "".to_string() }
            )
    }
    fn get_sel_capture_replacement(
        &self,
        ec: &Captures<'_>,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> String {
        self.get_raw_sel_capture_replacement(ec, sel, con)
            .unwrap_or_else(||
                if self.keep_groups { ec[0].to_string() } else { "".to_string() }
            )
    }
    /// fills groups having a default value (after the colon)
    ///
    /// This is used to fill the input in case on non auto_exec
    /// verb triggered with a key
    pub fn invocation_with_default(
        &self,
        verb_invocation: &VerbInvocation,
        con: &AppContext,
    ) -> VerbInvocation {
        VerbInvocation {
            name: verb_invocation.name.clone(),
            args: verb_invocation.args.as_ref().map(|a| {
                GROUP.replace_all(
                    a.as_str(),
                    |ec: &Captures<'_>| {
                        ec.get(2)
                            .map(|default_name| default_name.as_str())
                            .and_then(|default_name|
                                self.get_raw_replacement(|sel|
                                    self.get_raw_sel_name_standard_replacement(default_name, sel, con)
                                )
                            )
                            .unwrap_or_default()
                    },
                ).to_string()
            }),
            bang: verb_invocation.bang,
        }
    }

    fn base_dir(&self) -> &Path {
        self.sel_info
            .one_sel()
            .map_or(self.root, |sel| sel.path)
    }
    /// replace groups in a sequence
    ///
    /// Replacing escapes for the shell for externals, and without
    /// escaping for internals.
    ///
    /// Note that this is *before* asking the (local or remote) panel
    /// state the sequential execution of the different commands. In
    /// this secondary execution, new replacements are expected too,
    /// depending on the verbs.
    pub fn sequence(
        &self,
        sequence: &Sequence,
        verb_store: &VerbStore,
        con: &AppContext,
        panel_state_type: Option<PanelStateType>,
    ) -> Sequence {
        let mut inputs = Vec::new();
        for input in sequence.raw.split(&sequence.separator) {
            let raw_parts = CommandParts::from(input.to_string());
            let (_, verb_invocation) = raw_parts.split();
            let verb_is_external = verb_invocation
                .and_then(|vi| {
                    let command = Command::from_parts(vi, true);
                    if let Command::VerbInvocate(invocation) = &command {
                        let search = verb_store.search_prefix(&invocation.name, panel_state_type);
                        if let PrefixSearchResult::Match(_, verb) = search {
                            return Some(verb);
                        }
                    }
                    None
                })
                .map_or(false, |verb| verb.get_internal().is_none());
            let input = if verb_is_external {
                self.shell_exec_string(&ExecPattern::from_string(input), con)
            } else {
                self.string(input, con)
            };
            inputs.push(input);
        }
        Sequence {
            raw: inputs.join(&sequence.separator),
            separator: sequence.separator.clone(),
        }
    }
    /// build a raw string, without escapings
    pub fn string(
        &self,
        pattern: &str,
        con: &AppContext,
    ) -> String {
        GROUP
            .replace_all(
                pattern,
                |ec: &Captures<'_>| self.get_capture_replacement(ec, con),
            )
            .to_string()
    }
    /// build a path
    pub fn path(
        &self,
        pattern: &str,
        con: &AppContext,
    ) -> PathBuf {
        path::path_from(
            self.base_dir(),
            path::PathAnchor::Unspecified,
            &GROUP.replace_all(
                pattern,
                |ec: &Captures<'_>| self.get_capture_replacement(ec, con),
            )
        )
    }
    /// build a shell compatible command, with escapings
    pub fn shell_exec_string(
        &self,
        exec_pattern: &ExecPattern,
        con: &AppContext,
    ) -> String {
        exec_pattern
            .apply(&|s| {
                GROUP.replace_all(
                    s,
                    |ec: &Captures<'_>| self.get_capture_replacement(ec, con),
                ).to_string()
            })
            .fix_paths()
            .to_string()
    }
    /// build a shell compatible command, with escapings, for a specific
    /// selection (this is intended for execution on all selections of a
    /// stage)
    pub fn sel_shell_exec_string(
        &self,
        exec_pattern: &ExecPattern,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> String {
        exec_pattern
            .apply(&|s| {
                GROUP.replace_all(
                    s,
                    |ec: &Captures<'_>| self.get_sel_capture_replacement(ec, sel, con),
                ).to_string()
            })
            .fix_paths()
            .to_string()
    }
    /// build a vec of tokens which can be passed to Command to
    /// launch an executable
    pub fn exec_token(
        &self,
        exec_pattern: &ExecPattern,
        con: &AppContext,
    ) -> Vec<String> {
        exec_pattern
            .apply(&|s| {
                GROUP.replace_all(
                    s,
                    |ec: &Captures<'_>| self.get_capture_replacement(ec, con),
                ).to_string()
            })
            .fix_paths()
            .into_array()
    }
    /// build a vec of tokens which can be passed to Command to
    /// launch an executable
    pub fn sel_exec_token(
        &self,
        exec_pattern: &ExecPattern,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Vec<String> {
        exec_pattern
            .apply(&|s| {
                GROUP.replace_all(
                    s,
                    |ec: &Captures<'_>| self.get_sel_capture_replacement(ec, sel, con),
                ).to_string()
            })
            .fix_paths()
            .into_array()
    }
}

#[cfg(test)]
mod execution_builder_test {

    // allows writing vo!["a", "b"] to build a vec of strings
    macro_rules! vo {
        ($($item:literal),* $(,)?) => {{
            let mut vec = Vec::new();
            $(
                vec.push($item.to_owned());
            )*
            vec
        }}
    }


    use {
        super::*,
    };

    fn check_build_execution_from_sel(
        exec_patterns: Vec<ExecPattern>,
        path: &str,
        replacements: Vec<(&str, &str)>,
        chk_exec_token: Vec<&str>,
    ) {
        let path = PathBuf::from(path);
        let sel = Selection {
            path: &path,
            line: 0,
            stype: SelectionType::File,
            is_exe: false,
        };
        let app_state = AppState::new(PathBuf::from("/".to_owned()));
        let mut builder = ExecutionStringBuilder::without_invocation(
            SelInfo::One(sel),
            &app_state,
        );
        let mut map = FxHashMap::default();
        for (k, v) in replacements {
            map.insert(k.to_owned(), v.to_owned());
        }
        builder.invocation_values = Some(map);
        let con = AppContext::default();
        for exec_pattern in exec_patterns {
            let exec_token = builder.exec_token(&exec_pattern, &con);
            assert_eq!(exec_token, chk_exec_token);
        }
    }

    #[test]
    fn test_build_execution() {
        check_build_execution_from_sel(
            vec![ExecPattern::from_string("vi {file}")],
            "/home/dys/dev",
            vec![],
            vec!["vi", "/home/dys/dev"],
        );
        check_build_execution_from_sel(
            vec![
                ExecPattern::from_string("/bin/e.exe -a {arg} -e {file}"),
                ExecPattern::from_array(vo!["/bin/e.exe","-a", "{arg}", "-e", "{file}"]),
            ],
            "expérimental & 试验性",
            vec![("arg", "deux mots")],
            vec!["/bin/e.exe", "-a", "deux mots", "-e", "expérimental & 试验性"],
        );
        check_build_execution_from_sel(
            vec![
                ExecPattern::from_string("xterm -e \"kak {file}\""),
                ExecPattern::from_array(vo!["xterm", "-e", "kak {file}"]),
            ],
            "/path/to/file",
            vec![],
            vec!["xterm", "-e", "kak /path/to/file"],
        );
    }
}

fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}
