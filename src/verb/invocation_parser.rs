use {
    super::*,
    crate::{
        app::*,
        errors::ConfError,
        path::PathAnchor,
    },
    regex::Regex,
    ahash::AHashMap,
    std::{
        path::PathBuf,
    },
};


/// Definition of how the user input should be checked
/// and maybe parsed to provide the arguments used
/// for execution or description.
#[derive(Debug)]
pub struct InvocationParser {

    /// pattern of how the command is supposed to be typed in the input
    pub invocation_pattern: VerbInvocation,

    /// a regex to read the arguments in the user input
    args_parser: Option<Regex>,

    /// whether the path, when non absolute, should be interpreted
    /// as relative to the closest directory (which may be the selection)
    /// or to the parent of the selection
    pub arg_anchor: PathAnchor,

    /// contain the type of selection in case there's only one arg
    /// and it's a path (when it's not None, the user can type ctrl-P
    /// to select the argument in another panel)
    pub arg_selection_type: Option<SelectionType>,

}

impl InvocationParser {

    pub fn new(
        invocation_str: &str,
    ) -> Result<Self, ConfError> {
        let invocation_pattern = VerbInvocation::from(invocation_str);
        let mut args_parser = None;
        let mut arg_selection_type = None;
        let mut arg_anchor = PathAnchor::Unspecified;
        if let Some(args) = &invocation_pattern.args {
            let spec = GROUP.replace_all(args, r"(?P<$1>.+)");
            let spec = format!("^{}$", spec);
            args_parser = match Regex::new(&spec) {
                Ok(regex) => Some(regex),
                Err(_) => {
                    return Err(ConfError::InvalidVerbInvocation { invocation: spec });
                }
            };
            if let Some(group) = GROUP.find(args) {
                if group.start() == 0 && group.end() == args.len() {
                    // there's one group, covering the whole args
                    arg_selection_type = Some(SelectionType::Any);
                    let group_str = group.as_str();
                    if group_str.ends_with("path-from-parent}") {
                        arg_anchor = PathAnchor::Parent;
                    } else if group_str.ends_with("path-from-directory}") {
                        arg_anchor = PathAnchor::Directory;
                    }
                }
            }
        }
        Ok(Self {
            invocation_pattern,
            args_parser,
            arg_selection_type,
            arg_anchor,
        })
    }

    pub fn name(&self) -> &str {
        &self.invocation_pattern.name
    }

    /// Assuming the verb has been matched, check whether the arguments
    /// are OK according to the regex. Return none when there's no problem
    /// and return the error to display if arguments don't match
    pub fn check_args(
        &self,
        invocation: &VerbInvocation,
        _other_path: &Option<PathBuf>,
    ) -> Option<String> {
        match (&invocation.args, &self.args_parser) {
            (None, None) => None,
            (None, Some(ref regex)) => {
                if regex.is_match("") {
                    None
                } else {
                    Some(self.invocation_pattern.to_string_for_name(&invocation.name))
                }
            }
            (Some(ref s), Some(ref regex)) => {
                if regex.is_match(&s) {
                    None
                } else {
                    Some(self.invocation_pattern.to_string_for_name(&invocation.name))
                }
            }
            (Some(_), None) => Some(format!("{} doesn't take arguments", invocation.name)),
        }
    }

    pub fn parse(&self, args: &str) -> Option<AHashMap<String, String>> {
        self.args_parser.as_ref()
            .map(|r| {
                let mut map = AHashMap::default();
                if let Some(input_cap) = r.captures(&args) {
                    for name in r.capture_names().flatten() {
                        if let Some(c) = input_cap.name(name) {
                            map.insert(name.to_string(), c.as_str().to_string());
                        }
                    }
                }
                map
            })
    }

}
