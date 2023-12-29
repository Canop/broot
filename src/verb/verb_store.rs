use {
    super::{
        Internal,
        Verb,
        VerbId,
    },
    crate::{
        app::*,
        command::Sequence,
        conf::{Conf, VerbConf},
        errors::ConfError,
        keys::KEY_FORMAT,
        keys,
        verb::*,
    },
    crokey::*,
};

/// Provide access to the verbs:
/// - the built-in ones
/// - the user defined ones
/// A user defined verb can replace a built-in.
/// When the user types some keys, we select a verb
/// - if the input exactly matches a shortcut or the name
/// - if only one verb name starts with the input
pub struct VerbStore {
    verbs: Vec<Verb>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixSearchResult<'v, T> {
    NoMatch,
    Match(&'v str, T),
    Matches(Vec<&'v str>),
}

impl VerbStore {
    pub fn new(conf: &mut Conf) -> Result<Self, ConfError> {
        let mut store = Self { verbs: Vec::new() };
        for vc in &conf.verbs {
            store.add_from_conf(vc)?;
        }
        store.add_builtin_verbs(); // at the end so that we can override them
        Ok(store)
    }

    fn add_builtin_verbs(
        &mut self,
    ) {
        use super::{ExternalExecutionMode::*, Internal::*};
        self.add_internal(escape).with_key(key!(esc));

        // input actions, not visible in doc, but available for
        // example in remote control
        self.add_internal(input_clear).no_doc();
        self.add_internal(input_del_char_left).no_doc();
        self.add_internal(input_del_char_below).no_doc();
        self.add_internal(input_del_word_left).no_doc();
        self.add_internal(input_del_word_right).no_doc();
        self.add_internal(input_go_to_end).with_key(key!(end)).no_doc();
        self.add_internal(input_go_left).no_doc();
        self.add_internal(input_go_right).no_doc();
        self.add_internal(input_go_to_start).with_key(key!(home)).no_doc();
        self.add_internal(input_go_word_left).no_doc();
        self.add_internal(input_go_word_right).no_doc();

        // arrow keys bindings
        self.add_internal(back).with_key(key!(left));
        self.add_internal(open_stay).with_key(key!(right));
        self.add_internal(line_down).with_key(key!(down)).with_key(key!('j'));
        self.add_internal(line_up).with_key(key!(up)).with_key(key!('k'));

        self.add_internal(set_syntax_theme);

        // those two operations are mapped on ALT-ENTER, one
        // for directories and the other one for the other files
        self.add_internal(open_leave) // calls the system open
            .with_condition(FileTypeCondition::File)
            .with_key(key!(alt-enter))
            .with_shortcut("ol");
        self.add_external("cd", "cd {directory}", FromParentShell)
            .with_condition(FileTypeCondition::Directory)
            .with_key(key!(alt-enter))
            .with_shortcut("ol")
            .with_description("change directory and quit");

        #[cfg(unix)]
        self.add_external("chmod {args}", "chmod {args} {file}", StayInBroot)
            .with_condition(FileTypeCondition::File);
        #[cfg(unix)]
        self.add_external("chmod {args}", "chmod -R {args} {file}", StayInBroot)
            .with_condition(FileTypeCondition::Directory);
        self.add_internal(open_preview);
        self.add_internal(close_preview);
        self.add_internal(toggle_preview);
        self.add_internal(preview_image)
            .with_shortcut("img");
        self.add_internal(preview_text)
            .with_shortcut("txt");
        self.add_internal(preview_binary)
            .with_shortcut("hex");
        self.add_internal(close_panel_ok);
        self.add_internal(close_panel_cancel)
            .with_key(key!(ctrl-w));
        #[cfg(unix)]
        self.add_external(
            "copy {newpath}",
            "cp -r {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("cp");
        #[cfg(windows)]
        self.add_external(
            "copy {newpath}",
            "xcopy /Q /H /Y /I {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("cp");
        #[cfg(feature = "clipboard")]
        self.add_internal(copy_line)
            .with_key(key!(alt-c));
        #[cfg(feature = "clipboard")]
        self.add_internal(copy_path);
        #[cfg(unix)]
        self.add_external(
            "copy_to_panel",
            "cp -r {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("cpp");
        #[cfg(windows)]
        self.add_external(
            "copy_to_panel",
            "xcopy /Q /H /Y /I {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("cpp");
        #[cfg(unix)]
        self.add_internal(filesystems)
            .with_shortcut("fs");
        // :focus is also hardcoded on Enter on directories
        // but ctrl-f is useful for focusing on a file's parent
        // (and keep the filter)
        self.add_internal(focus)
            .with_key(key!(L))  // hum... why this one ?
            .with_key(key!(ctrl-f));
        self.add_internal(help)
            .with_key(key!(F1))
            .with_shortcut("?");
        #[cfg(feature="clipboard")]
        self.add_internal(input_paste)
            .with_key(key!(ctrl-v));
        #[cfg(unix)]
        self.add_external(
            "mkdir {subpath}",
            "mkdir -p {subpath:path-from-directory}",
            StayInBroot,
        )
            .with_shortcut("md");
        #[cfg(windows)]
        self.add_external(
            "mkdir {subpath}",
            "cmd /c mkdir {subpath:path-from-directory}",
            StayInBroot,
        )
            .with_shortcut("md");
        #[cfg(unix)]
        self.add_external(
            "move {newpath}",
            "mv {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("mv");
        #[cfg(windows)]
        self.add_external(
            "move {newpath}",
            "cmd /c move /Y {file} {newpath:path-from-parent}",
            StayInBroot,
        )
            .with_shortcut("mv");
        #[cfg(unix)]
        self.add_external(
            "move_to_panel",
            "mv {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("mvp");
        #[cfg(windows)]
        self.add_external(
            "move_to_panel",
            "cmd /c move /Y {file} {other-panel-directory}",
            StayInBroot,
        )
            .with_shortcut("mvp");
        #[cfg(unix)]
        self.add_external(
            "rename {new_filename:file-name}",
            "mv {file} {parent}/{new_filename}",
            StayInBroot,
        )
            .with_auto_exec(false)
            .with_key(key!(f2));
        #[cfg(windows)]
        self.add_external(
            "rename {new_filename:file-name}",
            "cmd /c move /Y {file} {parent}/{new_filename}",
            StayInBroot,
        )
            .with_auto_exec(false)
            .with_key(key!(f2));
        self.add_internal_bang(start_end_panel)
            .with_key(key!(ctrl-p));
        // the char keys for mode_input are handled differently as they're not
        // consumed by the command
        self.add_internal(mode_input)
            .with_key(key!(' '))
            .with_key(key!(':'))
            .with_key(key!('/'));
        self.add_internal(previous_match)
            .with_key(key!(shift-backtab))
            .with_key(key!(backtab));
        self.add_internal(next_match)
            .with_key(key!(tab));
        self.add_internal(no_sort)
            .with_shortcut("ns");
        self.add_internal(open_stay)
            .with_key(key!(enter))
            .with_shortcut("os");
        self.add_internal(open_stay_filter)
            .with_shortcut("osf");
        self.add_internal(parent)
            .with_key(key!(h))
            .with_shortcut("p");
        self.add_internal(page_down)
            .with_key(key!(ctrl-d))
            .with_key(key!(pagedown));
        self.add_internal(page_up)
            .with_key(key!(ctrl-u))
            .with_key(key!(pageup));
        self.add_internal(panel_left_no_open)
            .with_key(key!(ctrl-left));
        self.add_internal(panel_right)
            .with_key(key!(ctrl-right));
        self.add_internal(print_path).with_shortcut("pp");
        self.add_internal(print_relative_path).with_shortcut("prp");
        self.add_internal(print_tree).with_shortcut("pt");
        self.add_internal(quit)
            .with_key(key!(ctrl-c))
            .with_key(key!(ctrl-q))
            .with_shortcut("q");
        self.add_internal(refresh).with_key(key!(f5));
        self.add_internal(root_up)
            .with_key(key!(ctrl-up));
        self.add_internal(root_down)
            .with_key(key!(ctrl-down));
        self.add_internal(select_first);
        self.add_internal(select_last);
        self.add_internal(select);
        self.add_internal(clear_stage).with_shortcut("cls");
        self.add_internal(stage)
            .with_key(key!('+'));
        self.add_internal(unstage)
            .with_key(key!('-'));
        self.add_internal(stage_all_files)
            .with_key(key!(ctrl-a));
        self.add_internal(toggle_stage)
            .with_key(key!(ctrl-g));
        self.add_internal(open_staging_area).with_shortcut("osa");
        self.add_internal(close_staging_area).with_shortcut("csa");
        self.add_internal(toggle_staging_area).with_shortcut("tsa");
        self.add_internal(toggle_tree).with_shortcut("tree");
        self.add_internal(sort_by_count).with_shortcut("sc");
        self.add_internal(sort_by_date).with_shortcut("sd");
        self.add_internal(sort_by_size).with_shortcut("ss");
        self.add_internal(sort_by_type).with_shortcut("st");
        #[cfg(unix)]
        self.add_external("rm", "rm -rf {file}", StayInBroot);
        #[cfg(windows)]
        self.add_external("rm", "cmd /c rmdir /Q /S {file}", StayInBroot)
            .with_condition(FileTypeCondition::Directory);
        #[cfg(windows)]
        self.add_external("rm", "cmd /c del /Q {file}", StayInBroot)
            .with_condition(FileTypeCondition::File);
        self.add_internal(toggle_counts).with_shortcut("counts");
        self.add_internal(toggle_dates).with_shortcut("dates");
        self.add_internal(toggle_device_id).with_shortcut("dev");
        self.add_internal(toggle_files).with_shortcut("files");
        self.add_internal(toggle_git_ignore)
            .with_key(key!(alt-i))
            .with_shortcut("gi");
        self.add_internal(toggle_git_file_info).with_shortcut("gf");
        self.add_internal(toggle_git_status).with_shortcut("gs");
        self.add_internal(toggle_root_fs).with_shortcut("rfs");
        self.add_internal(toggle_hidden)
            .with_key(key!(alt-h))
            .with_shortcut("h");
        #[cfg(unix)]
        self.add_internal(toggle_perm).with_shortcut("perm");
        self.add_internal(toggle_sizes).with_shortcut("sizes");
        self.add_internal(toggle_trim_root);
        self.add_internal(trash);
        self.add_internal(total_search).with_key(key!(ctrl-s));
        self.add_internal(up_tree).with_shortcut("up");
    }

    fn build_add_internal(
        &mut self,
        internal: Internal,
        bang: bool,
    ) -> &mut Verb {
        let invocation = internal.invocation_pattern();
        let execution = VerbExecution::Internal(
            InternalExecution::from_internal_bang(internal, bang)
        );
        let description = VerbDescription::from_text(internal.description().to_string());
        self.add_verb(Some(invocation), execution, description).unwrap()
    }

    fn add_internal(
        &mut self,
        internal: Internal,
    ) -> &mut Verb {
        self.build_add_internal(internal, false)
    }

     fn add_internal_bang(
        &mut self,
        internal: Internal,
    ) -> &mut Verb {
        self.build_add_internal(internal, true)
    }

    fn add_external(
        &mut self,
        invocation_str: &str,
        execution_str: &str,
        exec_mode: ExternalExecutionMode,
    ) -> &mut Verb {
        let execution = VerbExecution::External(
            ExternalExecution::new(ExecPattern::from_string(execution_str), exec_mode)
        );
        self.add_verb(
            Some(invocation_str),
            execution,
            VerbDescription::from_code(execution_str.to_string()),
        ).unwrap()
    }

    pub fn add_verb(
        &mut self,
        invocation_str: Option<&str>,
        execution: VerbExecution,
        description: VerbDescription,
    ) -> Result<&mut Verb, ConfError> {
        let id = self.verbs.len();
        self.verbs.push(Verb::new(
            id,
            invocation_str,
            execution,
            description,
        )?);
        Ok(&mut self.verbs[id])
    }

    /// Create a verb from its configuration, adding it to its store
    pub fn add_from_conf(
        &mut self,
        vc: &VerbConf,
    ) -> Result<(), ConfError> {
        if vc.leave_broot == Some(false) && vc.from_shell == Some(true) {
            return Err(ConfError::InvalidVerbConf {
                details: "You can't simultaneously have leave_broot=false and from_shell=true".to_string(),
            });
        }
        let invocation = vc.invocation.clone().filter(|i| !i.is_empty());
        let internal = vc.internal.as_ref().filter(|i| !i.is_empty());
        let external = vc.external.as_ref().filter(|i| !i.is_empty());
        let cmd = vc.cmd.as_ref().filter(|i| !i.is_empty());
        let cmd_separator = vc.cmd_separator.as_ref().filter(|i| !i.is_empty());
        let execution = vc.execution.as_ref().filter(|i| !i.is_empty());
        let make_external_execution = |s| {
            let working_dir = match (vc.set_working_dir, &vc.working_dir) {
                (Some(false), _) => None,
                (_, Some(s)) => Some(s.clone()),
                (Some(true), None) => Some("{directory}".to_owned()),
                (None, None) => None,
            };
            let mut external_execution = ExternalExecution::new(
                s,
                ExternalExecutionMode::from_conf(vc.from_shell, vc.leave_broot),
            )
            .with_working_dir(working_dir);
            if let Some(b) = vc.switch_terminal {
                external_execution.switch_terminal = b;
            }
            external_execution
        };
        let execution = match (execution, internal, external, cmd) {
            // old definition with "execution": we guess whether it's an internal or
            // an external
            (Some(ep), None, None, None) => {
                if let Some(internal_pattern) = ep.as_internal_pattern() {
                    if let Some(previous_verb) = self.verbs.iter().find(|&v| v.has_name(internal_pattern)) {
                        previous_verb.execution.clone()
                    } else {
                        VerbExecution::Internal(InternalExecution::try_from(internal_pattern)?)
                    }
                } else {
                    VerbExecution::External(make_external_execution(ep.clone()))
                }
            }
            // "internal": the leading `:` or ` ` is optional
            (None, Some(s), None, None) => {
                VerbExecution::Internal(if s.starts_with(':') || s.starts_with(' ') {
                    InternalExecution::try_from(&s[1..])?
                } else {
                    InternalExecution::try_from(s)?
                })
            }
            // "external": it can be about any form
            (None, None, Some(ep), None) => {
                VerbExecution::External(make_external_execution(ep.clone()))
            }
            // "cmd": it's a sequence
            (None, None, None, Some(s)) => VerbExecution::Sequence(SequenceExecution {
                sequence: Sequence::new(s, cmd_separator),
            }),
            _ => {
                return Err(ConfError::InvalidVerbConf {
                    details: "You must define either internal, external or cmd".to_string(),
                });
            }
        };
        let description = vc
            .description
            .clone()
            .map(VerbDescription::from_text)
            .unwrap_or_else(|| VerbDescription::from_code(execution.to_string()));
        let verb = self.add_verb(
            invocation.as_deref(),
            execution,
            description,
        )?;
        // we accept both key and keys. We merge both here
        let mut unchecked_keys = vc.keys.clone();
        if let Some(key) = &vc.key {
            unchecked_keys.push(key.clone());
        }
        let mut checked_keys = Vec::new();
        for key in &unchecked_keys {
            let key = crokey::parse(key)?;
            if keys::is_reserved(key) {
                return Err(ConfError::ReservedKey {
                    key: keys::KEY_FORMAT.to_string(key)
                });
            }
            checked_keys.push(key);
        }
        for extension in &vc.extensions {
            verb.file_extensions.push(extension.clone());
        }
        if !checked_keys.is_empty() {
            verb.add_keys(checked_keys);
        }
        if let Some(shortcut) = &vc.shortcut {
            verb.names.push(shortcut.clone());
        }
        if vc.auto_exec == Some(false) {
            verb.auto_exec = false;
        }
        if !vc.panels.is_empty() {
            verb.panels = vc.panels.clone();
        }
        verb.selection_condition = vc.apply_to;
        Ok(())
    }

    pub fn search_sel_info<'v>(
        &'v self,
        prefix: &str,
        sel_info: SelInfo<'_>,
    ) -> PrefixSearchResult<'v, &Verb> {
        self.search(prefix, Some(sel_info))
    }

    pub fn search_prefix<'v>(
        &'v self,
        prefix: &str,
    ) -> PrefixSearchResult<'v, &Verb> {
        self.search(prefix, None)
    }

    /// Return either the only match, or None if there's not
    /// exactly one match
    pub fn search_sel_info_unique <'v>(
        &'v self,
        prefix: &str,
        sel_info: SelInfo<'_>,
    ) -> Option<&'v Verb> {
        match self.search_sel_info(prefix, sel_info) {
            PrefixSearchResult::Match(_, verb) => Some(verb),
            _ => None,
        }
    }

    pub fn search<'v>(
        &'v self,
        prefix: &str,
        sel_info: Option<SelInfo>,
    ) -> PrefixSearchResult<'v, &Verb> {
        let mut found_index = 0;
        let mut nb_found = 0;
        let mut completions: Vec<&str> = Vec::new();
        let extension = sel_info.as_ref().and_then(|si| si.extension());
        let sel_count = sel_info.map(|si| si.count_paths());
        for (index, verb) in self.verbs.iter().enumerate() {
            if let Some(sel_info) = sel_info {
                if !sel_info.is_accepted_by(verb.selection_condition) {
                    continue;
                }
            }
            if let Some(count) = sel_count {
                if count > 1 && verb.is_sequence() {
                    continue;
                }
                if count == 0 && verb.needs_selection {
                    continue;
                }
            }
            if !verb.accepts_extension(extension) {
                continue;
            }
            for name in &verb.names {
                if name.starts_with(prefix) {
                    if name == prefix {
                        return PrefixSearchResult::Match(name, verb);
                    }
                    found_index = index;
                    nb_found += 1;
                    completions.push(name);
                    continue;
                }
            }
        }
        match nb_found {
            0 => PrefixSearchResult::NoMatch,
            1 => PrefixSearchResult::Match(completions[0], &self.verbs[found_index]),
            _ => PrefixSearchResult::Matches(completions),
        }
    }

    pub fn key_desc_of_internal_stype(
        &self,
        internal: Internal,
        stype: SelectionType,
    ) -> Option<String> {
        for verb in &self.verbs {
            if verb.get_internal() == Some(internal) && verb.selection_condition.accepts_selection_type(stype) {
                return verb.keys.first().map(|&k| KEY_FORMAT.to_string(k));
            }
        }
        None
    }

    pub fn key_desc_of_internal(
        &self,
        internal: Internal,
    ) -> Option<String> {
        for verb in &self.verbs {
            if verb.get_internal() == Some(internal) {
                return verb.keys.first().map(|&k| KEY_FORMAT.to_string(k));
            }
        }
        None
    }

    pub fn verbs(&self) -> &[Verb] {
        &self.verbs
    }

    pub fn verb(&self, id: VerbId) -> &Verb {
        &self.verbs[id]
    }

}
