
use {
    super::{
        GROUP,
        InvocationParser,
    },
    crate::{
        app::{Selection},
        path,
    },
    regex::{Captures},
    fnv::FnvHashMap,
    std::{
        path::{Path, PathBuf},
    },
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
        invocation_args: & Option<String>,
    ) -> Self {
        let invocation_values = invocation_parser.as_ref().zip(invocation_args.as_ref())
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
    fn path_to_string(&self, path: &Path, escape: bool) -> String {
        if escape {
            path::escape_for_shell(path)
        } else {
            path.to_string_lossy().to_string()
        }
    }
    fn get_raw_capture_replacement(&self, ec: &Captures<'_>, escape: bool) -> Option<String> {
        let name = ec.get(1).unwrap().as_str();
        match name {
            "line" => Some(self.sel.line.to_string()),
            "file" => Some(self.path_to_string(self.get_file(), escape)),
            "directory" => Some(self.path_to_string(&self.get_directory(), escape)),
            "parent" => Some(self.path_to_string(self.get_parent(), escape)),
            "other-panel-file" => self.other_file.map(|p| self.path_to_string(p, escape)),
            "other-panel-directory" => {
                self.other_file
                    .map(|p| path::closest_dir(p))
                    .as_ref()
                    .map(|p| self.path_to_string(p, escape))
            }
            "other-panel-parent" => {
                self.other_file
                    .and_then(|p| p.parent())
                    .map(|p| self.path_to_string(p, escape))
            }
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
    fn get_capture_replacement(&self, ec: &Captures<'_>, escape: bool) -> String {
        self.get_raw_capture_replacement(ec, escape)
            .unwrap_or_else(|| ec[0].to_string())
    }
    /// build a shell compatible command, with escapings
    pub fn shell_exec_string(
        &self,
        exec_pattern: &str,
    ) -> String {
        GROUP
            .replace_all(
                exec_pattern,
                |ec: &Captures<'_>| self.get_capture_replacement(ec, true),
            )
            .to_string()
            .split_whitespace()
            .map(|token| {
                let path = Path::new(token);
                if path.exists() {
                    if let Some(path) = path.to_str() {
                        return path.to_string();
                    }
                }
                token.to_string()
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
    /// build a vec of tokens which can be passed to Command to
    /// launch an executable
    pub fn exec_token(
        &self,
        exec_pattern: &str,
    ) -> Vec<String> {
        exec_pattern
            .split_whitespace()
            .map(|token| {
                GROUP
                    .replace_all(
                        token,
                        |ec: &Captures<'_>| self.get_capture_replacement(ec, false),
                    )
                    .to_string()
            })
            .collect()
    }
}
