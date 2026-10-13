use {
    super::*,
    crate::{
        app::*,
        command::*,
        git,
        path::{
            self,
            PathAnchor,
        },
    },
    regex::Captures,
    rustc_hash::FxHashMap,
    std::path::{
        Path,
        PathBuf,
    },
};

/// a temporary structure gathering selection and invocation
/// parameters and able to generate an executable string from
/// a verb's execution pattern
pub struct ExecutionBuilder<'b> {
    /// the current file selection
    pub sel_info: SelInfo<'b>,

    /// the current root of the app
    root: &'b Path,

    /// the selection in the other panel, when there are exactly two
    other_file: Option<&'b PathBuf>,

    /// the parser of the verb's invocation, defining the invocation arguments
    invocation_parser: Option<&'b InvocationParser>,

    /// parsed arguments
    invocation_values: Option<FxHashMap<String, String>>,
}

impl<'b> ExecutionBuilder<'b> {
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
            invocation_parser: None,
            invocation_values: None,
        }
    }
    pub fn with_invocation(
        invocation_parser: Option<&'b InvocationParser>,
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
            invocation_parser,
            invocation_values,
        }
    }

    /// Return the values of the argument for the whole sel_info: one per selection
    /// for a merging argument, at most one otherwise
    ///
    /// When you have a multiselection and no merging flag, don't call this function
    /// but get_sel_arg_replacement while building a command per selection.
    fn get_arg_values(
        &self,
        arg_def: &ExecArgDef,
        con: &AppContext,
    ) -> Vec<String> {
        match self.sel_info {
            SelInfo::None => self.get_sel_arg_replacement(arg_def, None, con).into_iter().collect(),
            SelInfo::One(sel) => self
                .get_sel_arg_replacement(arg_def, Some(sel), con)
                .into_iter()
                .collect(),
            SelInfo::More(stage) => {
                let sels = stage.to_selections();
                if arg_def.merging_flag().is_some() {
                    sels.into_iter()
                        .filter_map(|sel| self.get_sel_arg_replacement(arg_def, Some(sel), con))
                        .collect()
                } else {
                    // we're called with no specific selection and there's no merging
                    // strategy, this should probably not really happen, we'll take
                    // the first selection
                    let sel = sels.into_iter().next();
                    self.get_sel_arg_replacement(arg_def, sel, con).into_iter().collect()
                }
            }
        }
    }

    /// return the standard replacement (ie not one from the invocation)
    fn get_sel_standard_replacement(
        &self,
        standard_arg: StandardArg,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Option<String> {
        match standard_arg {
            StandardArg::Root => Some(path_to_string(self.root)),
            StandardArg::InitialRoot => Some(path_to_string(&con.initial_root)),
            StandardArg::Line => sel.map(|s| s.line.to_string()),
            StandardArg::File => sel.map(|s| s.path).map(path_to_string),
            StandardArg::FileName => sel
                .map(|s| s.path)
                .and_then(|path| path.file_name())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string()),
            StandardArg::FileStem => sel
                .map(|s| s.path)
                .and_then(|path| path.file_stem())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string()),
            StandardArg::FileExtension => {
                debug!("expending file extension");
                sel.map(|s| s.path)
                    .and_then(|path| path.extension())
                    .and_then(|oss| oss.to_str())
                    .map(|s| s.to_string())
            }
            StandardArg::FileDotExtension => {
                debug!("expending file dot extension");
                sel.map(|s| s.path)
                    .and_then(|path| path.extension())
                    .and_then(|oss| oss.to_str())
                    .map(|ext| format!(".{ext}"))
                    .or_else(|| Some("".to_string()))
            }
            StandardArg::Directory => sel.map(|s| path::closest_dir(s.path)).map(path_to_string),
            StandardArg::Parent => sel.and_then(|s| s.path.parent()).map(path_to_string),
            StandardArg::OtherPanelFile => self.other_file.map(path_to_string),
            StandardArg::OtherPanelFilename => self
                .other_file
                .and_then(|path| path.file_name())
                .and_then(|oss| oss.to_str())
                .map(|s| s.to_string()),
            StandardArg::OtherPanelDirectory => self
                .other_file
                .map(|p| path::closest_dir(p))
                .map(path_to_string),
            StandardArg::OtherPanelParent => {
                self.other_file.and_then(|p| p.parent()).map(path_to_string)
            }
            StandardArg::GitRoot => {
                // path to git repo workdir
                debug!("finding git root");
                sel.and_then(|s| git::workdir(s.path)).map(path_to_string)
            }
            StandardArg::GitName => {
                // name of the git repo workdir
                sel.and_then(|s| git::workdir(s.path)).and_then(|path| {
                    path.file_name()
                        .and_then(|oss| oss.to_str())
                        .map(|s| s.to_string())
                })
            }
            StandardArg::FileGitRelative => {
                // file path relative to git repo workdir
                let sel = sel?;
                let path = git::workdir(self.root)
                    .and_then(|gitroot| sel.path.strip_prefix(gitroot).ok())
                    .filter(|p| {
                        // it's empty when the file is both the tree root and the git root
                        !p.as_os_str().is_empty()
                    })
                    .unwrap_or(sel.path);
                Some(path_to_string(path))
            }
            StandardArg::FileRootRelative => {
                // file path relative to the tree root
                sel?.path.strip_prefix(self.root).ok().map(path_to_string)
            }
            StandardArg::ServerName => con.server_name.clone(),
        }
    }
    fn get_sel_arg_replacement(
        &self,
        arg_def: &ExecArgDef,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Option<String> {
        if let Some(standard_arg) = arg_def.standard_arg(self.invocation_parser) {
            return self.get_sel_standard_replacement(standard_arg, sel, con);
        }
        let value = self.invocation_values.as_ref()?.get(&arg_def.name)?;
        if arg_def.has_flag(ExecArgFlag::PathFromDirectory) {
            sel.map(|s| path::closest_dir(s.path))
                .map(|dir| path::path_from(dir, PathAnchor::Unspecified, value))
                .map(path_to_string)
        } else if arg_def.has_flag(ExecArgFlag::PathFromParent) {
            sel.map(|s| s.path.parent().unwrap_or(s.path))
                .map(|dir| path::path_from(dir, PathAnchor::Unspecified, value))
                .map(path_to_string)
        } else {
            Some(value.to_string())
        }
    }
    /// Replace the arg groups in a string, escaping each value for a shell
    /// when `escape` is true
    fn replace_args(
        &self,
        s: &str,
        escape: bool,
        values: &dyn Fn(&ExecArgDef) -> Vec<String>,
    ) -> String {
        ARG_DEF_GROUP
            .replace_all(s, |ec: &Captures<'_>| {
                let Ok(arg_def) = ExecArgDef::from_capture(ec) else {
                    return String::new();
                };
                let values = values(&arg_def);
                let separator = arg_def
                    .merging_flag()
                    .and_then(|flag| flag.separator())
                    .unwrap_or(" ");
                let mut replacement = String::new();
                if escape && values.is_empty() {
                    replacement.push_str("''");
                }
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        replacement.push_str(separator);
                    }
                    if escape {
                        replacement.push_str(&shell_escape(value));
                    } else {
                        replacement.push_str(value);
                    }
                }
                replacement
            })
            .to_string()
    }
    /// Build the invocation with the arguments replaced by their default
    /// values (or by nothing when they have none)
    ///
    /// This is used to fill the input in case on non auto_exec
    /// verb triggered with a key.
    pub fn invocation_with_default(
        &self,
        invocation_parser: &InvocationParser,
        con: &AppContext,
    ) -> VerbInvocation {
        let verb_invocation = &invocation_parser.invocation_pattern;
        VerbInvocation {
            name: verb_invocation.name.clone(),
            args: verb_invocation.args.as_ref().map(|a| {
                ARG_DEF_GROUP
                    .replace_all(a.as_str(), |ec: &Captures<'_>| {
                        invocation_parser
                            .get_arg_def(&ec[1])
                            .and_then(|arg_def| arg_def.default)
                            .and_then(|standard_arg| {
                                self.get_sel_standard_replacement(
                                    standard_arg,
                                    self.sel_info.first_sel(),
                                    con,
                                )
                            })
                            .unwrap_or_default()
                    })
                    .to_string()
            }),
            bang: verb_invocation.bang,
        }
    }

    fn base_dir(&self) -> &Path {
        self.sel_info.one_sel().map_or(self.root, |sel| sel.path)
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
                .is_some_and(|verb| verb.get_internal().is_none());
            let input = if verb_is_external {
                self.shell_exec_string(&ExecPattern::from_string(input), con)
            } else {
                self.replace_args(input, false, &|arg_def| self.get_arg_values(arg_def, con))
            };
            inputs.push(input);
        }
        Sequence {
            raw: inputs.join(&sequence.separator),
            separator: sequence.separator.clone(),
        }
    }

    /// build a path from a pattern (eg the `working_dir` parameter of a verb definition)
    pub fn path(
        &self,
        pattern: &str,
        con: &AppContext,
    ) -> PathBuf {
        path::path_from(
            self.base_dir(),
            path::PathAnchor::Unspecified,
            &self.replace_args(pattern, false, &|arg_def| self.get_arg_values(arg_def, con)),
        )
    }

    /// build a shell compatible command, with escapings
    pub fn shell_exec_string(
        &self,
        exec_pattern: &ExecPattern,
        con: &AppContext,
    ) -> String {
        exec_pattern.shell_line(
            |line| self.replace_args(line, true, &|arg_def| self.get_arg_values(arg_def, con)),
            |tokens| self.expand_tokens(tokens, con),
        )
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
        exec_pattern.shell_line(
            |line| {
                self.replace_args(line, true, &|arg_def| {
                    self.get_sel_arg_replacement(arg_def, sel, con).into_iter().collect()
                })
            },
            |tokens| self.sel_expand_tokens(tokens, sel, con),
        )
    }

    /// build a vec of tokens which can be passed to Command to
    /// launch an executable.
    pub fn exec_token(
        &self,
        exec_pattern: &ExecPattern,
        con: &AppContext,
    ) -> Vec<String> {
        self.expand_tokens(exec_pattern.tokens(), con)
    }

    /// build a vec of tokens which can be passed to Command to
    /// launch an executable.
    /// This is intended for execution on all selections of a stage
    /// when the exec pattern isn't merging.
    pub fn sel_exec_token(
        &self,
        exec_pattern: &ExecPattern,
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Vec<String> {
        self.sel_expand_tokens(exec_pattern.tokens(), sel, con)
    }

    fn expand_tokens(
        &self,
        tokens: &[String],
        con: &AppContext,
    ) -> Vec<String> {
        // When a token is a space-separated arg, and the selection is multiple,
        // we want to build several tokens so that it's received as several args by the
        // executed program, and not as a single arg with spaces.
        // This complex work is needed only when the selection is multiple and there's
        // a "space-separated" flag in the capture
        let mut output = Vec::new();
        for token in tokens {
            let arg_def = capture_if_total(&ARG_DEF_GROUP, token)
                .and_then(|ec| ExecArgDef::from_capture(&ec).ok());
            if let Some(arg_def) = arg_def {
                let space_separated = arg_def.has_flag(ExecArgFlag::SpaceSeparated);
                if space_separated {
                    if let SelInfo::More(stage) = &self.sel_info {
                        let sels = stage.to_selections();
                        for sel in sels {
                            if let Some(s) = self.get_sel_arg_replacement(&arg_def, Some(sel), con)
                            {
                                output.push(s);
                            }
                        }
                        continue; // we did the replacement
                    }
                }
            }
            // as we won't be able to build several tokens from this one, we do the
            // standard replacement
            let replaced =
                self.replace_args(token, false, &|arg_def| self.get_arg_values(arg_def, con));
            output.push(fix_token_path(replaced));
        }
        output
    }

    fn sel_expand_tokens(
        &self,
        tokens: &[String],
        sel: Option<Selection<'_>>,
        con: &AppContext,
    ) -> Vec<String> {
        tokens
            .iter()
            .map(|s| {
                self.replace_args(s, false, &|arg_def| {
                    self.get_sel_arg_replacement(arg_def, sel, con).into_iter().collect()
                })
            })
            .map(fix_token_path)
            .collect()
    }
}

fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

fn capture_if_total<'h>(
    regex: &Regex,
    s: &'h str,
) -> Option<Captures<'h>> {
    let captures = regex.captures(s)?;
    let overall_match = captures.get(0)?;
    if overall_match.start() == 0 && overall_match.end() == s.len() {
        Some(captures)
    } else {
        None
    }
}

fn fix_token_path<T: Into<String> + AsRef<str>>(token: T) -> String {
    let path = Path::new(token.as_ref());
    if path.exists() {
        if let Some(path) = path.to_str() {
            return path.to_string();
        }
    } else if path::TILDE_REGEX.is_match(token.as_ref()) {
        let path = path::untilde(token.as_ref());
        if path.exists() {
            if let Some(path) = path.to_str() {
                return path.to_string();
            }
        }
    }
    token.into()
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

    use super::*;

    fn check_build_execution_from_sel(
        exec_patterns: Vec<ExecPattern>,
        path: &str,
        replacements: Vec<(&str, &str)>,
        chk_exec_token: Vec<&str>,
    ) {
        let path = PathBuf::from(path);
        let sel = Selection::new(&path, SelectionType::File);
        let app_state = AppState::new(PathBuf::from("/".to_owned()));
        let mut invocation = "test".to_string();
        for (k, _) in &replacements {
            invocation.push_str(&format!(" {{{k}}}"));
        }
        let parser = InvocationParser::new(&invocation).unwrap();
        let mut builder = ExecutionBuilder::without_invocation(SelInfo::One(sel), &app_state);
        builder.invocation_parser = Some(&parser);
        let mut map = FxHashMap::default();
        for (k, v) in replacements {
            map.insert(k.to_owned(), v.to_owned());
        }
        builder.invocation_values = Some(map);
        let con = AppContext::default();
        for exec_pattern in exec_patterns {
            dbg!("checking pattern: {:#?}", &exec_pattern);
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
                ExecPattern::from_tokens(vo!["/bin/e.exe", "-a", "{arg}", "-e", "{file}"]),
            ],
            "expérimental & 试验性",
            vec![("arg", "deux mots")],
            vec![
                "/bin/e.exe",
                "-a",
                "deux mots",
                "-e",
                "expérimental & 试验性",
            ],
        );
        check_build_execution_from_sel(
            vec![
                ExecPattern::from_string("xterm -e \"kak {file}\""),
                ExecPattern::from_tokens(vo!["xterm", "-e", "kak {file}"]),
            ],
            "/path/to/file",
            vec![],
            vec!["xterm", "-e", "kak /path/to/file"],
        );
    }

    #[test]
    fn test_invocation_arg_masks_standard_arg() {
        check_build_execution_from_sel(
            vec![ExecPattern::from_string("echo {file} {parent}")],
            "/home/dys/dev",
            vec![("file", "typed")],
            vec!["echo", "typed", "/home/dys"],
        );
        // without a typed value, the masked standard argument isn't used either
        let path = PathBuf::from("/home/dys/dev");
        let sel = Selection::new(&path, SelectionType::File);
        let app_state = AppState::new(PathBuf::from("/"));
        let parser = InvocationParser::new("x {file}").unwrap();
        let builder =
            ExecutionBuilder::with_invocation(Some(&parser), SelInfo::One(sel), &app_state, None);
        let con = AppContext::default();
        assert_eq!(
            builder.exec_token(&ExecPattern::from_string("echo {file}"), &con),
            vec!["echo", ""],
        );
    }

    #[test]
    fn test_shell_exec_string_keeps_chained_command() {
        // a command with `&&` must survive tokenizing untouched so it can be
        // handed to `sh -c` when the verb is a `shell_command` (issue #1145)
        let app_state = AppState::new(PathBuf::from("/"));
        let builder = ExecutionBuilder::without_invocation(SelInfo::None, &app_state);
        let con = AppContext::default();
        let ep = ExecPattern::from_string("mkdir foo && touch foo/bar");
        assert_eq!(
            builder.shell_exec_string(&ep, &con),
            "mkdir foo && touch foo/bar",
        );
    }
}
