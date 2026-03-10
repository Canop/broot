use {
    crate::verb::*,
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
            tokens: splitty::split_unquoted_whitespace(s)
                .unwrap_quotes(true)
                .map(String::from)
                .collect(),
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
    pub fn has_selection_group(&self) -> bool {
        self.tokens.iter().any(|s| str_has_selection_group(s))
    }
    pub fn has_other_panel_group(&self) -> bool {
        self.tokens.iter().any(|s| str_has_other_panel_group(s))
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
        f: &mut dyn FnMut(&VerbArgDef),
    ) {
        for token in &self.tokens {
            for capture in ARG_DEF_GROUP.captures_iter(token) {
                let arg_def = VerbArgDef::from_capture(&capture);
                f(&arg_def);
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
            Raw::Single(s) => splitty::split_unquoted_whitespace(&s)
                .map(String::from)
                .collect(),
            Raw::Multiple(v) => v,
        };

        Ok(ExecPattern { tokens })
    }
}
