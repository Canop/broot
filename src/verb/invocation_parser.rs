use {
    super::*,
    crate::{
        errors::ConfError,
        path::PathAnchor,
    },
    regex::Regex,
    rustc_hash::FxHashMap,
    std::path::PathBuf,
};

/// Definition of how the user input should be checked
/// and maybe parsed to provide the arguments used
/// for execution or description.
#[derive(Debug)]
pub struct InvocationParser {
    /// pattern of how the command is supposed to be typed in the input
    pub invocation_pattern: VerbInvocation,

    /// a regex to read the arguments in the user input
    /// This regex declares named groups, with the name being the
    /// name of the replacement variable (this implies that an
    /// invocation name's characters are [_0-9a-zA-Z.\[\]])
    args_parser: Option<Regex>,

    pub arg_defs: Vec<VerbArgDef>,
}

impl InvocationParser {
    pub fn new(invocation_str: &str) -> Result<Self, ConfError> {
        let invocation_pattern = VerbInvocation::from(invocation_str);
        let mut args_parser = None;
        let mut arg_defs = Vec::new();
        if let Some(args) = &invocation_pattern.args {
            let spec = ARG_DEF_GROUP.replace_all(args, r"(?P<$1>.+)");
            let spec = format!("^{spec}$");
            args_parser = match Regex::new(&spec) {
                Ok(regex) => Some(regex),
                Err(_) => {
                    return Err(ConfError::InvalidVerbInvocation { invocation: spec });
                }
            };
            for group in ARG_DEF_GROUP.captures_iter(args) {
                arg_defs.push(VerbArgDef::from_capture(&group));
            }
        }
        Ok(Self {
            invocation_pattern,
            args_parser,
            arg_defs,
        })
    }

    pub fn name(&self) -> &str {
        &self.invocation_pattern.name
    }

    pub fn get_unique_arg_def(&self) -> Option<VerbArgDef> {
        (self.arg_defs.len() == 1).then(|| self.arg_defs[0].clone())
    }

    pub fn get_unique_arg_anchor(&self) -> PathAnchor {
        self.get_unique_arg_def()
            .map(|arg_def| arg_def.path_anchor())
            .unwrap_or_default()
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
                if regex.is_match(s) {
                    None
                } else {
                    Some(self.invocation_pattern.to_string_for_name(&invocation.name))
                }
            }
            (Some(_), None) => Some(format!("{} doesn't take arguments", invocation.name)),
        }
    }

    pub fn parse(
        &self,
        args: &str,
    ) -> Option<FxHashMap<String, String>> {
        self.args_parser.as_ref().map(|r| {
            let mut map = FxHashMap::default();
            if let Some(input_cap) = r.captures(args) {
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
