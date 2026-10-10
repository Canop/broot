use {
    crate::{
        errors::ConfError,
        path::PathAnchor,
    },
    lazy_regex::*,
    std::str::FromStr,
    std::fmt,
};

/// A `{name:flags}` group in a verb definition string, where `name` is the argument name and
/// `flags` is a comma-separated list of flags that modify how the argument is processed.
///
/// This pattern is also used slightly differently in verb invocations, where the flags part
/// can be used to specify a default value.
pub static ARG_DEF_GROUP: Lazy<Regex> = lazy_regex!(r"\{([^{}:]+)(?::([^{}:]+))?\}");

#[derive(Debug, Clone, PartialEq)]
pub struct VerbArgDef {
    pub name: String,
    pub flags: Vec<VerbArgFlag>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerbArgFlag {
    CommaSeparated,
    SpaceSeparated,
    PathFromDirectory,
    PathFromParent,
    Theme,
}

impl VerbArgFlag {
    pub fn is_merging(&self) -> bool {
        matches!(self, Self::CommaSeparated | Self::SpaceSeparated)
    }
    pub fn merge_values(
        &self,
        args: Vec<String>,
    ) -> Option<String> {
        if args.is_empty() {
            return None;
        }
        match self {
            Self::CommaSeparated => Some(args.join(",")),
            Self::SpaceSeparated => Some(args.join(" ")),
            _ => None,
        }
    }
    pub fn path_anchor(&self) -> PathAnchor {
        match self {
            Self::PathFromDirectory => PathAnchor::Directory,
            Self::PathFromParent => PathAnchor::Parent,
            _ => crate::path::PathAnchor::Unspecified,
        }
    }
}

impl VerbArgDef {
    /// Assuming a valid capture from the GROUP regex, parse the argument definition
    pub fn from_capture(capture: &Captures<'_>) -> VerbArgDef {
        let name = capture
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_else(|| {
                // internal error, the regex should guarantee this group exists
                error!("Invalid capture for argument definition");
                "???"
            })
            .to_string();
        let flags = capture
            .get(2)
            .map(|m| {
                m.as_str()
                    .split(',')
                    .map(str::trim)
                    .filter_map(|s| match s.parse() {
                        Ok(flag) => Some(flag),
                        Err(e) => {
                            warn!("Invalid flag '{}' in argument definition: {}", s, e);
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        VerbArgDef {
            name: name.as_str().to_string(),
            flags,
        }
    }
    pub fn merging_flag(&self) -> Option<VerbArgFlag> {
        for flag in &self.flags {
            if flag.is_merging() {
                return Some(*flag);
            }
        }
        None
    }
    pub fn has_flag(
        &self,
        flag: VerbArgFlag,
    ) -> bool {
        self.flags.contains(&flag)
    }
    pub fn path_anchor(&self) -> PathAnchor {
        for flag in &self.flags {
            let anchor = flag.path_anchor();
            if anchor != PathAnchor::Unspecified {
                return anchor;
            }
        }
        PathAnchor::Unspecified
    }
}

impl FromStr for VerbArgFlag {
    type Err = ConfError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "comma-separated" => Ok(Self::CommaSeparated),
            "space-separated" => Ok(Self::SpaceSeparated),
            "path-from-directory" => Ok(Self::PathFromDirectory),
            "path-from-parent" => Ok(Self::PathFromParent),
            "theme" => Ok(Self::Theme),
            _ => Err(ConfError::UnknownVerbArgFlag {
                name: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for VerbArgFlag {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let s = match self {
            Self::CommaSeparated => "comma-separated",
            Self::SpaceSeparated => "space-separated",
            Self::PathFromDirectory => "path-from-directory",
            Self::PathFromParent => "path-from-parent",
            Self::Theme => "theme",
        };
        write!(f, "{s}")
    }
}
