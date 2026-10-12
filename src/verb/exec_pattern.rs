use {
    crate::{
        conf::ConfError,
        verb::*,
    },
    lazy_regex::regex_is_match,
    serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
    },
    std::fmt,
};

/// A pattern which can be expanded into an executable
#[derive(Debug, Clone)]
pub struct ExecPattern {
    tokens: Vec<String>,
}

impl ExecPattern {
    pub fn from_string(s: &str) -> Self {
        Self {
            tokens: rejoin_split_groups(
                splitty::split_unquoted_whitespace(s)
                    .unwrap_quotes(true)
                    .map(String::from),
            ),
        }
    }
    pub fn from_tokens(tokens: Vec<String>) -> Self {
        Self { tokens }
    }
    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }
    pub fn into_tokens(self) -> Vec<String> {
        self.tokens
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    /// Parse the arg defs of all tokens
    pub fn arg_defs(&self) -> Result<Vec<ExecArgDef>, ConfError> {
        let mut arg_defs = Vec::new();
        for token in &self.tokens {
            arg_defs.extend(parse_exec_arg_defs(token)?);
        }
        Ok(arg_defs)
    }
    pub fn to_internal_pattern(&self) -> Option<String> {
        let first_token = self.tokens.first()?;
        if first_token.starts_with(':') || first_token.starts_with(' ') {
            let mut ip = String::from(&first_token[1..]);
            for token in self.tokens.iter().skip(1) {
                ip.push(' ');
                ip.push_str(token);
            }
            Some(ip)
        } else {
            None
        }
    }

    pub fn visit_arg_defs(
        &self,
        f: &mut dyn FnMut(&ExecArgDef),
    ) {
        for token in &self.tokens {
            for capture in ARG_DEF_GROUP.captures_iter(token) {
                if let Ok(arg_def) = ExecArgDef::from_capture(&capture) {
                    f(&arg_def);
                }
            }
        }
    }

    /// Tell whether, in case of a multiple selection, the command should be executed once per
    /// selection or once for all selections together (meaning the selections will be merged).
    pub fn coarity(&self) -> CommandCoarity {
        let mut has_repeated = false;
        self.visit_arg_defs(&mut |arg_def| {
            for flag in &arg_def.flags {
                debug!("arg {} has flag {:?}", arg_def.name, flag);
                if flag.is_merging() {
                    has_repeated = true;
                }
            }
        });
        if has_repeated {
            CommandCoarity::Merged
        } else {
            CommandCoarity::PerSelection
        }
    }
}

// This implementation builds a string used for description (eg in help)
impl fmt::Display for ExecPattern {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for (idx, s) in self.tokens.iter().enumerate() {
            if idx > 0 {
                write!(f, " ")?;
            }
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

impl Serialize for ExecPattern {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        self.tokens.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ExecPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Single(String),
            Multiple(Vec<String>),
        }

        let tokens = match Raw::deserialize(deserializer)? {
            Raw::Single(s) => {
                rejoin_split_groups(splitty::split_unquoted_whitespace(&s).map(String::from))
            }
            Raw::Multiple(v) => v,
        };

        Ok(ExecPattern { tokens })
    }
}

/// Rejoin the tokens of a `{name:flags}` group which was split on a space
/// between flags (eg `{file:space-separated, path-from-parent}`)
fn rejoin_split_groups(tokens: impl Iterator<Item = String>) -> Vec<String> {
    let mut joined: Vec<String> = Vec::new();
    let mut in_group = false;
    for token in tokens {
        match joined.last_mut() {
            Some(last) if in_group => {
                last.push(' ');
                last.push_str(&token);
            }
            _ => joined.push(token),
        }
        in_group = joined
            .last()
            .is_some_and(|last| regex_is_match!(r"\{[^{}\s:]+:[^{}]*$", last));
    }
    joined
}

#[cfg(test)]
mod exec_pattern_tests {
    use super::*;

    #[test]
    fn test_split_keeps_groups() {
        let ep = ExecPattern::from_string("mpv {file:space-separated, path-from-parent} --x");
        assert_eq!(ep.tokens(), &["mpv", "{file:space-separated, path-from-parent}", "--x"]);
        let ep: ExecPattern =
            deser_hjson::from_str(r#""cp {file} {dest:path-from-parent,  space-separated}""#)
                .unwrap();
        assert_eq!(ep.tokens(), &["cp", "{file}", "{dest:path-from-parent, space-separated}"]);
        // braces which don't open a {name:flags} group are left alone
        let ep = ExecPattern::from_string("sh -c { a; b; } {file}");
        assert_eq!(ep.tokens(), &["sh", "-c", "{", "a;", "b;", "}", "{file}"]);
    }
}
