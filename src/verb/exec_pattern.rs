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
    std::{
        borrow::Cow,
        fmt,
    },
};

/// A pattern which can be expanded into an executable, built either
/// from a command line or from a list of arguments
#[derive(Debug, Clone)]
pub struct ExecPattern {
    /// the given arguments, or the ones of the command line without
    /// their wrapping quotes
    tokens: Vec<String>,
    /// the command line, when the pattern was built from one
    raw: Option<String>,
}

impl ExecPattern {
    pub fn from_string(s: &str) -> Self {
        Self {
            tokens: rejoin_split_groups(
                splitty::split_unquoted_whitespace(s)
                    .unwrap_quotes(true)
                    .map(String::from),
            ),
            raw: Some(s.to_string()),
        }
    }
    pub fn from_tokens(tokens: Vec<String>) -> Self {
        Self { tokens, raw: None }
    }
    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }
    /// Build the command line to run through a shell
    ///
    /// `expand_line` must replace the arg groups of a command line, with shell escaped
    /// values, and `expand_args` must build the arguments of a direct execution.
    pub fn shell_line(
        &self,
        expand_line: impl FnOnce(&str) -> String,
        expand_args: impl FnOnce(&[String]) -> Vec<String>,
    ) -> String {
        match &self.raw {
            Some(raw) => expand_line(raw),
            None => join_shell_words(&expand_args(&self.tokens)),
        }
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
        if let Some(raw) = &self.raw {
            return write!(f, "{raw}");
        }
        for (idx, s) in self.tokens.iter().enumerate() {
            if idx > 0 {
                write!(f, " ")?;
            }
            if s.contains(char::is_whitespace) {
                write!(f, "\"{s}\"")?;
            } else {
                write!(f, "{s}")?;
            }
        }
        Ok(())
    }
}

impl Serialize for ExecPattern {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match &self.raw {
            Some(raw) => raw.serialize(serializer),
            None => self.tokens.serialize(serializer),
        }
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

        Ok(match Raw::deserialize(deserializer)? {
            Raw::Single(s) => Self::from_string(&s),
            Raw::Multiple(v) => Self::from_tokens(v),
        })
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

/// Escape a string so that a shell reads it as a single word
pub fn shell_escape(s: &str) -> Cow<'_, str> {
    if s.is_empty() {
        return Cow::Borrowed("''");
    }
    // backslashes must be in this set: on Windows the string may go to
    // the launcher's outcmd file, whose eval would remove them
    if !regex_is_match!(r#"[\s"'\\`$*?\[\]{}()<>|;&!#]"#, s) {
        return Cow::Borrowed(s);
    }
    // the string is wrapped in single quotes, and each single quote is replaced
    // by `'"'"'` (close the single quote, add an escaped one, reopen)
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('\'');
    for c in s.chars() {
        if c == '\'' {
            escaped.push_str(r#"'"'"'"#);
        } else {
            escaped.push(c);
        }
    }
    escaped.push('\'');
    Cow::Owned(escaped)
}

/// Build a shell command line whose words are the given strings
fn join_shell_words(words: &[String]) -> String {
    let mut line = String::new();
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }
        line.push_str(&shell_escape(word));
    }
    line
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

    #[test]
    fn test_quotes_in_string_pattern() {
        let identity_line = |line: &str| line.to_string();
        let identity_args = |args: &[String]| args.to_vec();
        let ep: ExecPattern = deser_hjson::from_str(r#" "xterm -e \"nvim {file}\"" "#).unwrap();
        assert_eq!(ep.tokens(), &["xterm", "-e", "nvim {file}"]);
        assert_eq!(ep.shell_line(identity_line, identity_args), r#"xterm -e "nvim {file}""#);
        let ep: ExecPattern = deser_hjson::from_str(r#"["xterm", "-e", "nvim {file}"]"#).unwrap();
        assert_eq!(ep.tokens(), &["xterm", "-e", "nvim {file}"]);
        assert_eq!(ep.shell_line(identity_line, identity_args), "xterm -e 'nvim {file}'");
    }

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("/home/dys/dev"), "/home/dys/dev");
        assert_eq!(shell_escape("/home/dys/my dev"), "'/home/dys/my dev'");
        assert_eq!(shell_escape(r"C:\Users\dys\dev"), r"'C:\Users\dys\dev'");
        assert_eq!(shell_escape("/home/dys/it's dev"), r#"'/home/dys/it'"'"'s dev'"#);
        assert_eq!(shell_escape("/tmp/a[1]"), "'/tmp/a[1]'");
        assert_eq!(shell_escape("/tmp/$(x)"), "'/tmp/$(x)'");
        assert_eq!(shell_escape("/tmp/a*b"), "'/tmp/a*b'");
        assert_eq!(shell_escape("/tmp/é-ü_1.txt"), "/tmp/é-ü_1.txt");
        assert_eq!(shell_escape(""), "''");
    }
}
