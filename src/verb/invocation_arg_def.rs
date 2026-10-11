use {
    super::*,
    crate::errors::ConfError,
    lazy_regex::*,
};

/// A `{name}` or `{name:flags}` group in an invocation pattern.
///
/// `flags` is a comma-separated list whose elements are either `theme`
/// or the name of a standard argument giving the default value.
#[derive(Debug, Clone, PartialEq)]
pub struct InvocationArgDef {
    pub name: String,
    pub kind: InvocationArgKind,
    /// The value proposed when the verb is triggered by a key without `auto_exec`
    pub default: Option<StandardArg>,
}

/// What the user is expected to type, which drives the tab completion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvocationArgKind {
    #[default]
    Path,
    Theme,
}

impl InvocationArgDef {
    /// Parse a capture of the `ARG_DEF_GROUP` regex
    pub fn from_capture(capture: &Captures<'_>) -> Result<Self, ConfError> {
        let mut def = Self {
            name: capture[1].to_string(),
            kind: InvocationArgKind::default(),
            default: None,
        };
        let Some(flags) = capture.get(2) else {
            return Ok(def);
        };
        for flag in flags.as_str().split(',').map(str::trim) {
            if flag == "theme" {
                def.kind = InvocationArgKind::Theme;
            } else if let Ok(standard_arg) = flag.parse() {
                if def.default.is_some() {
                    return Err(ConfError::InvalidVerbConf {
                        details: format!("several default values in {:?}", &capture[0]),
                    });
                }
                def.default = Some(standard_arg);
            } else {
                return Err(ConfError::UnknownInvocationArgFlag {
                    name: flag.to_string(),
                });
            }
        }
        Ok(def)
    }
}

#[cfg(test)]
mod invocation_arg_def_tests {
    use super::*;

    fn parse(s: &str) -> Result<InvocationArgDef, ConfError> {
        let capture = ARG_DEF_GROUP.captures(s).unwrap();
        InvocationArgDef::from_capture(&capture)
    }

    #[test]
    fn test_parse_invocation_arg_def() {
        assert_eq!(
            parse("{new_filename:file-name}").unwrap(),
            InvocationArgDef {
                name: "new_filename".to_string(),
                kind: InvocationArgKind::Path,
                default: Some(StandardArg::FileName),
            }
        );
        assert_eq!(
            parse("{theme:theme}").unwrap(),
            InvocationArgDef {
                name: "theme".to_string(),
                kind: InvocationArgKind::Theme,
                default: None,
            }
        );
        assert!(parse("{newpath:path-from-parent}").is_err());
        assert!(parse("{newpath:file,parent}").is_err());
    }
}
