use {
    super::*,
    crate::{
        conf::ConfError,
        path::PathAnchor,
    },
    lazy_regex::*,
    std::{
        fmt,
        str::FromStr,
    },
};

/// A `{name}` or `{name:flags}` group in an execution pattern, where `flags` is a
/// comma-separated list of flags modifying how the value is built.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecArgDef {
    pub name: String,
    pub flags: Vec<ExecArgFlag>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecArgFlag {
    CommaSeparated,
    SpaceSeparated,
    PathFromDirectory,
    PathFromParent,
}

impl ExecArgFlag {
    pub fn is_merging(&self) -> bool {
        matches!(self, Self::CommaSeparated | Self::SpaceSeparated)
    }
    /// Return the separator of the merged values, for a merging flag
    pub fn separator(&self) -> Option<&'static str> {
        match self {
            Self::CommaSeparated => Some(","),
            Self::SpaceSeparated => Some(" "),
            _ => None,
        }
    }
    /// Return the anchor of the typed path, for a path flag
    pub fn path_anchor(&self) -> Option<PathAnchor> {
        match self {
            Self::PathFromDirectory => Some(PathAnchor::Directory),
            Self::PathFromParent => Some(PathAnchor::Parent),
            _ => None,
        }
    }
}

impl ExecArgDef {
    /// Parse a capture of the `ARG_DEF_GROUP` regex
    pub fn from_capture(capture: &Captures<'_>) -> Result<Self, ConfError> {
        let name = capture[1].to_string();
        let flags = match capture.get(2) {
            Some(flags) => flags
                .as_str()
                .split(',')
                .map(|s| s.trim().parse())
                .collect::<Result<_, _>>()?,
            None => Vec::new(),
        };
        let def = Self { name, flags };
        def.check_flags_compatibility()?;
        Ok(def)
    }
    /// Check there's at most one merging flag and at most one path flag
    fn check_flags_compatibility(&self) -> Result<(), ConfError> {
        for (i, &first) in self.flags.iter().enumerate() {
            for &second in &self.flags[i + 1..] {
                if first == second {
                    return Err(ConfError::RepeatedArgFlag {
                        name: self.name.clone(),
                        flag: first,
                    });
                }
                if (first.is_merging() && second.is_merging()) || (first.path_anchor().is_some() && second.path_anchor().is_some()) {
                    return Err(ConfError::ConflictingArgFlags {
                        name: self.name.clone(),
                        first,
                        second,
                    });
                }
            }
        }
        Ok(())
    }
    /// Return the standard argument this group refers to, if any (an argument
    /// of the invocation masks the standard argument of the same name)
    pub fn standard_arg(
        &self,
        invocation_parser: Option<&InvocationParser>,
    ) -> Option<StandardArg> {
        if invocation_parser.is_some_and(|parser| parser.has_arg(&self.name)) {
            return None;
        }
        self.name.parse().ok()
    }
    pub fn merging_flag(&self) -> Option<ExecArgFlag> {
        self.flags.iter().copied().find(ExecArgFlag::is_merging)
    }
    pub fn has_flag(
        &self,
        flag: ExecArgFlag,
    ) -> bool {
        self.flags.contains(&flag)
    }
    /// Check the group doesn't have a flag applying only to invocation arguments,
    /// assuming it's a standard argument
    pub fn check_standard_flags(&self) -> Result<(), ConfError> {
        match self.flags.iter().find(|flag| flag.path_anchor().is_some()) {
            Some(flag) => Err(ConfError::PathFlagOnStandardArg {
                name: self.name.clone(),
                flag: *flag,
            }),
            None => Ok(()),
        }
    }
    /// Return the anchor of the typed path, when the group has a path flag
    pub fn path_anchor(&self) -> Option<PathAnchor> {
        self.flags.iter().find_map(ExecArgFlag::path_anchor)
    }
}

/// Parse all the arg defs found in a string of an execution pattern
pub fn parse_exec_arg_defs(s: &str) -> Result<Vec<ExecArgDef>, ConfError> {
    ARG_DEF_GROUP
        .captures_iter(s)
        .map(|capture| ExecArgDef::from_capture(&capture))
        .collect()
}

impl FromStr for ExecArgFlag {
    type Err = ConfError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "comma-separated" => Ok(Self::CommaSeparated),
            "space-separated" => Ok(Self::SpaceSeparated),
            "path-from-directory" => Ok(Self::PathFromDirectory),
            "path-from-parent" => Ok(Self::PathFromParent),
            _ => Err(ConfError::UnknownVerbArgFlag {
                name: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for ExecArgFlag {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let s = match self {
            Self::CommaSeparated => "comma-separated",
            Self::SpaceSeparated => "space-separated",
            Self::PathFromDirectory => "path-from-directory",
            Self::PathFromParent => "path-from-parent",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod exec_arg_def_tests {
    use super::*;

    #[test]
    fn test_parse_exec_arg_defs() {
        let defs = parse_exec_arg_defs("zip -r {name:path-from-directory}.zip {file:space-separated, path-from-parent}").unwrap();
        assert_eq!(
            defs,
            vec![
                ExecArgDef {
                    name: "name".to_string(),
                    flags: vec![ExecArgFlag::PathFromDirectory],
                },
                ExecArgDef {
                    name: "file".to_string(),
                    flags: vec![ExecArgFlag::SpaceSeparated, ExecArgFlag::PathFromParent],
                },
            ]
        );
        assert_eq!(defs[1].standard_arg(None), Some(StandardArg::File));
        assert_eq!(defs[0].standard_arg(None), None);
        let parser = InvocationParser::new("x {file}").unwrap();
        assert_eq!(defs[1].standard_arg(Some(&parser)), None);
    }

    #[test]
    fn test_unknown_exec_arg_flag() {
        assert!(parse_exec_arg_defs("nvim {file:space_separated}").is_err());
        assert!(parse_exec_arg_defs("mv {file} {new_filename:file-name}").is_err());
        assert!(parse_exec_arg_defs(":set_syntax_theme {theme:theme}").is_err());
    }

    #[test]
    fn test_incompatible_exec_arg_flags() {
        assert!(parse_exec_arg_defs("mpv {file:space-separated,comma-separated}").is_err());
        assert!(parse_exec_arg_defs("mkdir {dir:path-from-parent,path-from-directory}").is_err());
        assert!(parse_exec_arg_defs("mpv {file:space-separated,space-separated}").is_err());
    }
}
