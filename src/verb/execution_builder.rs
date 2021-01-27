use {
    super::*,
    crate::{
        app::Selection,
        path,
    },
    fnv::FnvHashMap,
    regex::Captures,
    std::path::{Path, PathBuf},
};

/// a temporary structure gathering selection and invocation
/// parameters and able to generate an executable string from
/// a verb's execution pattern
pub struct ExecutionStringBuilder<'b> {
    /// the current file selection
    pub sel: Selection<'b>,

    /// the selection in the other panel, when there exactly two
    other_file: Option<&'b PathBuf>,

    /// parsed arguments
    invocation_values: Option<FnvHashMap<String, String>>,
}

impl<'b> ExecutionStringBuilder<'b> {
    pub fn from_selection(
        sel: Selection<'b>,
    ) -> Self {
        Self {
            sel,
            other_file: None,
            invocation_values: None,
        }
    }
    pub fn from_invocation(
        invocation_parser: &Option<InvocationParser>,
        sel: Selection<'b>,
        other_file: &'b Option<PathBuf>,
        invocation_args: &Option<String>,
    ) -> Self {
        let invocation_values = invocation_parser
            .as_ref()
            .zip(invocation_args.as_ref())
            .and_then(|(parser, args)| parser.parse(args));
        Self {
            sel,
            other_file: other_file.as_ref(),
            invocation_values,
        }
    }
    fn get_file(&self) -> &Path {
        &self.sel.path
    }
    fn get_directory(&self) -> PathBuf {
        path::closest_dir(self.sel.path)
    }
    fn get_parent(&self) -> &Path {
        let file = &self.sel.path;
        file.parent().unwrap_or(file)
    }
    fn path_to_string(&self, path: &Path) -> String {
        path.to_string_lossy().to_string()
    }
    fn get_raw_capture_replacement(&self, ec: &Captures<'_>) -> Option<String> {
        let name = ec.get(1).unwrap().as_str();
        match name {
            "line" => Some(self.sel.line.to_string()),
            "file" => Some(self.path_to_string(self.get_file())),
            "directory" => Some(self.path_to_string(&self.get_directory())),
            "parent" => Some(self.path_to_string(self.get_parent())),
            "other-panel-file" => self.other_file.map(|p| self.path_to_string(p)),
            "other-panel-directory" => self
                .other_file
                .map(|p| path::closest_dir(p))
                .as_ref()
                .map(|p| self.path_to_string(p)),
            "other-panel-parent" => self
                .other_file
                .and_then(|p| p.parent())
                .map(|p| self.path_to_string(p)),
            _ => {
                // it's not one of the standard group names, so we'll look
                // into the ones provided by the invocation pattern
                self.invocation_values.as_ref()
                    .and_then(|map| map.get(name)
                        .map(|value| {
                            if let Some(fmt) = ec.get(2) {
                                match fmt.as_str() {
                                    "path-from-directory" => path::path_str_from(self.get_directory(), value),
                                    "path-from-parent" => path::path_str_from(self.get_parent(), value),
                                    _ => format!("invalid format: {:?}", fmt.as_str()),
                                }
                            } else {
                                value.to_string()
                            }
                        })
                    )
            }
        }
    }
    fn get_capture_replacement(&self, ec: &Captures<'_>) -> String {
        self.get_raw_capture_replacement(ec)
            .unwrap_or_else(|| ec[0].to_string())
    }
    /// build a shell compatible command, with escapings
    pub fn shell_exec_string(
        &self,
        exec_pattern: &ExecPattern,
    ) -> String {
        exec_pattern
            .apply(&|s| {
                GROUP.replace_all(
                    s,
                    |ec: &Captures<'_>| self.get_capture_replacement(ec),
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
    ) -> Vec<String> {
        exec_pattern
            .apply(&|s| {
                GROUP.replace_all(
                    s,
                    |ec: &Captures<'_>| self.get_capture_replacement(ec),
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
        crate::app::SelectionType,
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
        let mut builder = ExecutionStringBuilder::from_selection(sel);
        let mut map = FnvHashMap::default();
        for (k, v) in replacements {
            map.insert(k.to_owned(), v.to_owned());
        }
        builder.invocation_values = Some(map);
        for exec_pattern in exec_patterns {
            let exec_token = builder.exec_token(&exec_pattern);
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
